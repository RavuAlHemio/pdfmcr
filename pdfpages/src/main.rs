use std::path::PathBuf;

use clap::Parser;
use lopdf::Document;


#[derive(Parser)]
struct Opts {
    pub pdf_file: PathBuf,
}


const POINTS_PER_INCH: f32 = 72.0;
const POINTS_PER_CM: f32 = 3600.0 / 127.0;


fn main() {
    let opts = Opts::parse();

    let doc = Document::load(&opts.pdf_file)
        .expect("failed to load PDF document");
    for (page_index, page_obj_id) in doc.page_iter().enumerate() {
        let page_number = page_index + 1;
        let page_dict = match doc.get_dictionary(page_obj_id) {
            Ok(pd) => pd,
            Err(e) => {
                eprintln!(
                    "failed to obtain dictionary for page {} (object ID {:?}): {}",
                    page_index,
                    page_obj_id,
                    e,
                );
                continue;
            },
        };
        let media_box = match page_dict.get(b"MediaBox") {
            Ok(mb) => mb,
            Err(_) => {
                eprintln!("page {} unknown media box", page_number);
                continue;
            },
        };
        let media_box_array = match media_box.as_array() {
            Ok(mba) => mba,
            Err(_) => {
                eprintln!("page {} media box not an array", page_number);
                continue;
            },
        };
        if media_box_array.len() != 4 {
            eprintln!("page {} media box has {} elements instead of 4", page_number, media_box_array.len());
            continue;
        }

        let mut dimensions = [0f32; 4];
        let mut dimensions_ok = true;
        for (elem_index, elem) in media_box_array.iter().enumerate() {
            match elem {
                lopdf::Object::Integer(i) => {
                    dimensions[elem_index] = *i as f32;
                },
                lopdf::Object::Real(r) => {
                    dimensions[elem_index] = *r;
                },
                other => {
                    eprintln!("page {} media box element {} is not a float but {:?}", page_number, elem_index, other);
                    dimensions_ok = false;
                    break;
                },
            }
        }
        if !dimensions_ok {
            continue;
        }

        if dimensions[0] != 0.0 || dimensions[1] != 0.0 {
            eprintln!("page {} media box is not anchored at (0, 0) but at ({}, {})", page_number, dimensions[0], dimensions[1]);
            continue;
        }

        let width_pt = dimensions[2];
        let height_pt = dimensions[3];

        let width_in = width_pt / POINTS_PER_INCH;
        let height_in = height_pt / POINTS_PER_INCH;

        let width_cm = width_pt / POINTS_PER_CM;
        let height_cm = height_pt / POINTS_PER_CM;

        println!(
            "page {}: {:.3} x {:.3} pt   {:.3} x {:.3} in   {:.3} x {:.3} cm",
            page_number,
            width_pt, height_pt,
            width_in, height_in,
            width_cm, height_cm,
        );
    }
}
