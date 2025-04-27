mod config;
mod file_to_pdf;
mod filters;
mod image_path;
mod jpeg;
mod model;
mod pdf;


use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use askama::Template;
use clap::Parser;
use rocket::{FromForm, Responder, uri};
use rocket::form::Form;
use rocket::fs::{relative, FileServer, TempFile};
use rocket::http::{ContentType, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use sha3::Sha3_512;
use sha3::digest::{Digest, DynDigest};
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use tracing::error;

use crate::config::{CONFIG, CONFIG_PATH, load_config};
use crate::image_path::ImagePath;
use crate::model::{Annotation, Artifact, JpegImage, JpegImageInfo, Page};


static WEB_FILE: OnceLock<RwLock<crate::model::File>> = OnceLock::new();


#[derive(Parser)]
struct Opts {
    #[arg(default_value = "config.toml")]
    pub config_path: PathBuf,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Responder)]
#[response(content_type = "html")]
struct Html(String);
impl From<String> for Html {
    fn from(value: String) -> Self { Self(value) }
}

#[derive(Debug, Responder)]
enum HtmlOrRedirect {
    Html(Html),
    Redirect(rocket::response::Redirect),
}


#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    page_number: usize,
    page_count: usize,
    page: &'a Page,
}

#[derive(Template)]
#[template(path = "start.html")]
struct StartTemplate;


#[rocket::get("/")]
async fn index() -> HtmlOrRedirect {
    let page_count = {
        let file_guard = WEB_FILE
            .get().expect("WEB_FILE not set?!")
            .read().await;
        file_guard.pages.len()
    };
    if page_count == 0 {
        HtmlOrRedirect::Html(StartTemplate.render().unwrap().into())
    } else {
        HtmlOrRedirect::Redirect(Redirect::to("/page/0"))
    }
}

#[rocket::get("/page/<page_number>")]
async fn page_page(page_number: usize) -> Result<Html, (Status, Cow<'static, str>)> {
    let (page_count, page) = {
        let file_guard = WEB_FILE
            .get().expect("WEB_FILE not set?!")
            .read().await;
        let page_count = file_guard.pages.len();
        if page_number >= page_count {
            return Err((Status::NotFound, Cow::Owned(format!("page {page_number} does not exist"))));
        }
        (page_count, file_guard.pages[page_number].clone())
    };
    let page_template = PageTemplate {
        page_number,
        page_count,
        page: &page,
    };
    Ok(page_template.render().unwrap().into())
}

#[derive(FromForm)]
struct MakePageForm<'r> {
    #[field(name = "background-image")]
    pub background_image: TempFile<'r>,
}

async fn persist_state_file() -> Result<(), (Status, Cow<'static, str>)> {
    let file_path = {
        let config_guard = CONFIG
            .get().expect("CONFIG not set?!")
            .read().await;
        config_guard.state_file_path.clone()
    };
    let file_data = {
        let file_guard = WEB_FILE
            .get().expect("WEB_FILE not set?!")
            .read().await;
        let mut buf = Vec::new();
        if let Err(e) = ciborium::into_writer(&*file_guard, &mut buf) {
            error!("failed to encode state as CBOR: {}", e);
            return Err((Status::InternalServerError, Cow::Borrowed("failed to encode state as CBOR")));
        }
        buf
    };
    if let Err(e) = std::fs::write(&file_path, &file_data) {
        error!("failed to write state CBOR file {:?}: {}", file_path, e);
        return Err((Status::InternalServerError, Cow::Borrowed("failed to write state CBOR file")));
    }
    Ok(())
}


#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct SetAnnotationsData {
    pub annotations: Vec<Annotation>,
    pub artifacts: Vec<Artifact>,
}
impl SetAnnotationsData {
    pub fn into_inner(self) -> (Vec<Annotation>, Vec<Artifact>) {
        (self.annotations, self.artifacts)
    }
}


#[rocket::post("/page/<page>/annotations", data = "<set_annotations>")]
async fn set_page_annotations(page: usize, set_annotations: Json<SetAnnotationsData>) -> Result<Cow<'static, str>, (Status, Cow<'static, str>)> {
    {
        let mut file_guard = WEB_FILE
            .get().expect("WEB_FILE not set?!")
            .write().await;
        if page >= file_guard.pages.len() {
            return Err((Status::NotFound, Cow::Borrowed("no such page")));
        }
        let (annotations, artifacts) = set_annotations.into_inner().into_inner();
        file_guard.pages[page].annotations = annotations;
        file_guard.pages[page].artifacts = artifacts;
    }

    Ok(Cow::Borrowed("OK"))
}

#[rocket::post("/page", data = "<form>")]
async fn make_page(mut form: Form<MakePageForm<'_>>) -> Result<Redirect, (Status, Cow<'static, str>)> {
    use std::fmt::Write;

    // generate a name for the JPEG file out of its size and checksum
    let jpeg_size = form.background_image.len();

    let filename = {
        let mut file = match form.background_image.open().await {
            Ok(f) => f,
            Err(e) => {
                error!("error opening uploaded file {:?}: {}", form.background_image, e);
                return Err((Status::InternalServerError, Cow::Borrowed("failed to open uploaded file")));
            },
        };

        let mut sha = Sha3_512::new();
        let mut buf = vec![0u8; 4*1024*1024];
        loop {
            let bytes_read = match file.read(&mut buf).await {
                Ok(br) => br,
                Err(e) => {
                    error!("failed to read from uploaded file {:?}: {}", form.background_image, e);
                    return Err((Status::InternalServerError, Cow::Borrowed("failed to read from uploaded file")));
                },
            };
            if bytes_read == 0 {
                break;
            }
            Digest::update(&mut sha, &buf[..bytes_read]);
        }

        let mut digest = [0u8; 64];
        DynDigest::finalize_into(sha, &mut digest)
            .expect("failed to finalize SHA3-512");
        let mut filename = String::with_capacity(digest.len() * 2);
        for &b in &digest {
            write!(filename, "{:02x}", b).unwrap();
        }

        // append hyphen, length and extension
        write!(filename, "-{}.jpeg", jpeg_size).unwrap();

        filename
    };

    // join the file to the expected base path
    let base_path_string = {
        let config_guard = CONFIG
            .get().expect("CONFIG not set?!")
            .read().await;
        config_guard.image_dir.clone()
    };
    let image_path: ImagePath = match filename.parse() {
        Ok(ip) => ip,
        Err(e) => {
            error!("generated image path {:?} is invalid: {}", filename, e);
            return Err((Status::InternalServerError, Cow::Borrowed("generated invalid image name")));
        },
    };
    let base_path = PathBuf::from(base_path_string);
    let os_image_path = base_path.join(filename);

    // persist the image there
    if let Err(e) = form.background_image.persist_to(&os_image_path).await {
        error!("failed to persist uploaded file {:?} to {}: {}", form.background_image, os_image_path.display(), e);
        return Err((Status::InternalServerError, Cow::Borrowed("failed to persist uploaded file")));
    }

    // read the image
    let mut image_file = match File::open(&os_image_path) {
        Ok(i) => i,
        Err(e) => {
            error!("error opening persisted uploaded file {:?}: {}", os_image_path.display(), e);
            return Err((Status::InternalServerError, Cow::Borrowed("failed to open persisted uploaded file")));
        },
    };
    let mut jpeg_image = match crate::jpeg::Image::try_read(&mut image_file) {
        Ok(ji) => ji,
        Err(e) => {
            error!("error reading uploaded file {:?} as JPEG: {}", os_image_path.display(), e);
            return Err((Status::InternalServerError, Cow::Borrowed("failed to read persisted uploaded file as JPEG")));
        },
    };
    jpeg_image.image_data.clear();

    // assemble the initial page structure
    let color_space = match jpeg_image.color_space {
        crate::jpeg::ColorSpace::Grayscale => crate::model::ColorSpace::Grayscale,
        crate::jpeg::ColorSpace::Rgb => crate::model::ColorSpace::Rgb,
        crate::jpeg::ColorSpace::Cmyk => crate::model::ColorSpace::Cmyk,
        crate::jpeg::ColorSpace::Other(o) => {
            return Err((Status::BadRequest, Cow::Owned(format!("JPEG has unknown color space {}", o))));
        },
    };
    let density_unit = match jpeg_image.density_unit {
        crate::jpeg::DensityUnit::NoUnit => {
            return Err((Status::BadRequest, Cow::Borrowed("JPEG images without a density unit are not supported")));
        },
        crate::jpeg::DensityUnit::DotsPerInch => crate::model::DensityUnit::DotsPerInch,
        crate::jpeg::DensityUnit::DotsPerCentimeter => crate::model::DensityUnit::DotsPerCentimeter,
        crate::jpeg::DensityUnit::Other(o) => {
            return Err((Status::BadRequest, Cow::Owned(format!("JPEG has unknown density unit {}", o))));
        },
    };
    if jpeg_image.bit_depth == 0 {
        return Err((Status::BadRequest, Cow::Borrowed("JPEG image cannot have a bit depth of 0")));
    }
    if jpeg_image.width == 0 || jpeg_image.height == 0 {
        return Err((Status::BadRequest, Cow::Borrowed("JPEG image cannot have a width or height of 0")));
    }
    if jpeg_image.density_x == 0 || jpeg_image.density_y == 0 {
        return Err((Status::BadRequest, Cow::Borrowed("JPEG image cannot have a horizontal or vertical pixel density of 0")));
    }
    let page = Page::new(JpegImage {
        info: JpegImageInfo {
            bit_depth: jpeg_image.bit_depth,
            width: jpeg_image.width,
            height: jpeg_image.height,
            color_space,
            density_unit,
            density_x: jpeg_image.density_x,
            density_y: jpeg_image.density_y,
        },
        file_path: image_path,
    });

    // append it
    let new_page_index = {
        let mut file_guard = WEB_FILE
            .get().expect("WEB_FILE not set?!")
            .write().await;
        let new_page_index = file_guard.pages.len();
        file_guard.pages.push(page);
        new_page_index
    };

    // persist the state
    persist_state_file().await?;

    // redirect to the new page
    Ok(Redirect::to(uri!(page_page(new_page_index))))
}

#[rocket::get("/page/<page>/image")]
async fn page_image(page: usize) -> Result<(ContentType, File), (Status, Cow<'static, str>)> {
    let page_path = {
        let file_guard = WEB_FILE
            .get().expect("WEB_FILE not set?!")
            .read().await;

        if page > file_guard.pages.len() {
            return Err((Status::NotFound, Cow::Borrowed("no such page")));
        }
        file_guard.pages[page].scanned_image.file_path.clone()
    };

    let base_path = {
        let config_guard = CONFIG
            .get().expect("CONFIG not set?!")
            .read().await;

        PathBuf::from(config_guard.image_dir.as_str())
    };

    let page_os_path = page_path.to_os_path(&base_path);
    let page_os_file = match File::open(&page_os_path) {
        Ok(pof) => pof,
        Err(e) => {
            error!("page file {:?} not found on system: {}", page_os_path.display(), e);
            return Err((Status::InternalServerError, Cow::Borrowed("file should exist but not found on server")));
        },
    };

    Ok((ContentType::JPEG, page_os_file))
}


#[rocket::launch]
fn launch_rocket() -> _ {
    // set up tracing
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // get arguments
    let opts = Opts::parse();

    let _ = CONFIG_PATH.set(opts.config_path);
    let config = load_config()
        .expect("failed to load config");
    CONFIG.set(RwLock::new(config.clone()))
        .expect("CONFIG already set?!");

    // read the initial file if it exists
    let initial_file = match std::fs::metadata(&config.state_file_path) {
        Ok(m) => {
            if !m.is_file() {
                panic!("state file {:?} exists and is not a file", config.state_file_path);
            }

            let initial_state_file = match File::open(&config.state_file_path) {
                Ok(ifc) => ifc,
                Err(e) => panic!("failed to open state file {:?}: {}", config.state_file_path, e),
            };
            match ciborium::from_reader(&initial_state_file) {
                Ok(is) => is,
                Err(e) => panic!("failed to parse state file {:?} as CBOR: {}", config.state_file_path, e),
            }
        },
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                crate::model::File::default()
            } else {
                panic!("could not open state file {:?}: {}", config.state_file_path, e);
            }
        },
    };
    WEB_FILE
        .set(RwLock::new(initial_file))
        .expect("WEB_FILE already set?!");

    // now, let's get down to brass tacks

    rocket::build()
        .mount("/", rocket::routes![
            index,
            page_image,
            page_page,
            make_page,
            set_page_annotations,
        ])
        .mount("/static", FileServer::from(relative!("static")).rank(2))
        .mount("/static/js", FileServer::from(relative!("ts/dist")).rank(1))
}
