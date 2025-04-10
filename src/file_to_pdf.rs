//! Conversion from pdfmcr files to PDF files.


use std::collections::BTreeMap;
use std::io::Write;

use crate::model::File;
use crate::pdf::{
    Catalog, Content, Document, ImageXObject, Page, PageContents, Pages, PdfId, StandardFont,
};


/// Converts a pdfmcr file to PDF.
pub(crate) fn file_to_pdf(file: &File) -> Document {
    // we'll go for the following structure:
    // 1 = catalog
    // 2 = page tree root with all pages
    // 3 = Times Regular font
    // 4 = Times Italic font
    // 5 = Times Bold font
    // 6 = Times BoldItalic font
    // 7+3i = page
    // 7+3i+1 = page content
    // 7+3i+2 = scanned page background image

    const COMMON_IDS: u64 = 6;
    const IDS_PER_PAGE: u64 = 3;

    let mut document = Document {
        objects: BTreeMap::new(),
    };

    let catalog = Catalog {
        root_pages_id: PdfId(2),
        lang: file.default_language.clone(),
    };
    document.objects.insert(PdfId(1), Content::Catalog(catalog));

    let pages_children = (0..file.pages.len()).into_iter()
        .map(|page_index| u64::try_from(page_index).unwrap())
        .map(|page_index| PdfId(1 + COMMON_IDS + IDS_PER_PAGE*page_index))
        .collect();
    let root_pages = Pages {
        children: pages_children,
    };
    document.objects.insert(PdfId(2), Content::Pages(root_pages));

    let times_regular = StandardFont { name: "Times-Regular".to_owned() };
    let times_italic = StandardFont { name: "Times-Italic".to_owned() };
    let times_bold = StandardFont { name: "Times-Bold".to_owned() };
    let times_bold_italic = StandardFont { name: "Times-BoldItalic".to_owned() };
    document.objects.insert(PdfId(3), Content::StandardFont(times_regular));
    document.objects.insert(PdfId(4), Content::StandardFont(times_italic));
    document.objects.insert(PdfId(5), Content::StandardFont(times_bold));
    document.objects.insert(PdfId(6), Content::StandardFont(times_bold_italic));

    let mut font_refs = BTreeMap::new();
    font_refs.insert("F0".to_owned(), PdfId(3));
    font_refs.insert("F1".to_owned(), PdfId(4));
    font_refs.insert("F2".to_owned(), PdfId(5));
    font_refs.insert("F3".to_owned(), PdfId(6));

    for (page_index_usize, page) in file.pages.iter().enumerate() {
        let page_index: u64 = page_index_usize.try_into().unwrap();
        let page_pdf_id = 1 + COMMON_IDS + IDS_PER_PAGE*page_index;

        let width_pt = page.scanned_image.info.density_unit.try_to_points(
            page.scanned_image.info.width,
            page.scanned_image.info.density_x,
        ).unwrap();
        let height_pt = page.scanned_image.info.density_unit.try_to_points(
            page.scanned_image.info.height,
            page.scanned_image.info.density_y,
        ).unwrap();

        let mut xobject_refs = BTreeMap::new();
        xobject_refs.insert(
            "Im0".to_owned(),
            PdfId(page_pdf_id + 2),
        );

        let pdf_page = Page {
            parent: PdfId(2),
            width_pt,
            height_pt,
            contents: Some(PdfId(page_pdf_id + 1)),
            xobject_refs,
            font_refs: font_refs.clone(),
        };
        document.objects.insert(
            PdfId(page_pdf_id),
            Content::Page(pdf_page),
        );

        let mut commands = Vec::new();
        // place the image, then the annotations, then the artifacts
        write!(commands, "q {} 0 0 {} 0 0 cm/Im0 Do Q", width_pt, height_pt).unwrap();
        for annotation in &page.annotations {
            annotation.write_drawing_commands(&mut commands).unwrap();
        }
        for artifact in &page.artifacts {
            artifact.write_drawing_commands(&mut commands).unwrap();
        }
        let content = PageContents {
            commands,
        };
        document.objects.insert(
            PdfId(page_pdf_id + 1),
            Content::PageContents(content),
        );

        let image_data = page.scanned_image.data.read()
            .expect("failed to read image data")
            .into_owned();
        let image = ImageXObject {
            width: page.scanned_image.info.width.into(),
            height: page.scanned_image.info.height.into(),
            color_space: page.scanned_image.info.color_space.as_pdf_name(),
            bits_per_component: page.scanned_image.info.bit_depth,
            interpolate: true,
            data_filters: vec!["DCTDecode".to_owned()],
            data: image_data,
        };
        document.objects.insert(
            PdfId(page_pdf_id + 1),
            Content::ImageXObject(image),
        );
    }

    document
}
