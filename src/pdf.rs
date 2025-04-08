//! Structures and procedures relevant to the Portable Document Format.


use std::collections::BTreeMap;
use std::io::{self, Write};


/// The ID of a PDF object.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PdfId(pub u64);


/// A document in Portable Document Format.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Document {
    /// A mapping from IDs to objects.
    ///
    /// Generation numbers are always 0 in this simplified implementation.
    pub objects: BTreeMap<u64, Content>,
}

/// The contents of a PDF object.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Content {
    Catalog(Catalog),
    Pages(Pages),
    Page(Page),
    PageContents(PageContents),
    ImageXObject(ImageXObject),
}

/// A Catalog PDF object, the topmost object in the hierarchical structure.
///
/// A catalog contains references to the root node of the page tree, to the tree of bookmarks, etc.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Catalog {
    pub root_pages_id: PdfId,
}

/// A Pages PDF object, a branch in the page tree.
///
/// The structure of the page tree is independent of the structure of the tree of bookmarks. The
/// simplest page tree consists of one [`Pages`] object with all [`Page`] objects as its direct
/// children, but the PDF standard recommends that the page tree be balanced.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pages {
    pub children: Vec<PdfId>,
}

/// A Page PDF object, a leaf in the page tree.
///
/// The structure of the page tree is independent of the structure of the tree of bookmarks. The
/// simplest page tree consists of one [`Pages`] object with all [`Page`] objects as its direct
/// children, but the PDF standard recommends that the page tree be balanced.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Page {
    /// The ID of the [`Pages`] object that is the immediate parent of this page.
    pub parent: PdfId,

    /// The width of the page, in points (1/72 in).
    pub width_pt: u64,

    /// The height of the page, in points (1/72 in).
    pub height_pt: u64,

    /// The ID of the content stream describing the contents of this page.
    pub contents: Option<PdfId>,

    /// Mapping of names to XObjects referenced by this page.
    pub xobject_refs: BTreeMap<String, PdfId>,

    /// Mapping of names to fonts referenced by this page.
    pub font_refs: BTreeMap<String, PdfId>,
}

/// A stream describing the contents of a PDF page.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PageContents {
    /// The drawing commands, in PDF's postfix operator notation.
    pub commands: String,
}

/// An external object (XObject) which is an image.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageXObject {
    /// The width of the image, in pixels.
    pub width: u64,

    /// The height of the image, in pixels.
    pub height: u64,

    /// The PDF name of the color space of the image.
    pub color_space: &'static str,

    /// The number of bits used to encode one color component of one pixel.
    pub bits_per_component: u8,

    /// Recommend that the image be interpolated when scaled.
    ///
    /// PDF viewers are free to ignore the value of this attribute.
    pub interpolate: bool,

    /// A list of PDF names of the filters applied to the image, in order.
    pub data_filters: Vec<String>,

    /// The binary data of the image.
    pub data: Vec<u8>,
}

/// One of the standard 14 fonts.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StandardFont {
    /// The name of the font.
    pub name: String,
}

/// Writes out a textual string in PDF format.
///
/// The string is wrapped in parentheses (`(` and `)`), encoded in UTF-16BE with BOM, and all
/// backslashes and parentheses are escaped with a preceding backslash.
pub fn write_pdf_string<W: Write>(string: &str, mut writer: W) -> Result<(), io::Error> {
    const ESCAPE_US: [u16; 3] = [
        b'(' as u16,
        b')' as u16,
        b'\\' as u16,
    ];

    writer.write_all(b"(\xFE\xFF")?;
    for word in string.encode_utf16() {
        if ESCAPE_US.contains(&word) {
            // precede with a backslash
            writer.write_all(b"\x00\x5C")?;
        }
        let word_be_bytes = word.to_be_bytes();
        writer.write_all(&word_be_bytes)?;
    }
    writer.write_all(b")")?;
    Ok(())
}
