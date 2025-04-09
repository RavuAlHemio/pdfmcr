//! Structures and procedures relevant to the Portable Document Format.


use std::collections::BTreeMap;
use std::io::{self, Seek, Write};


/// The ID of a PDF object.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PdfId(pub u64);


/// A document in Portable Document Format.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Document {
    /// A mapping from IDs to objects.
    ///
    /// Generation numbers are always 0 in this simplified implementation.
    pub objects: BTreeMap<PdfId, Content>,
}
impl Document {
    pub fn write_pdf<W: Write + Seek>(&self, writer: &mut W) -> Result<(), io::Error> {
        let pdf_start_pos = writer.stream_position()?;

        // header (magic and binary detection comment line)
        writer.write_all(b"%PDF-1.5\n%\xE2\xE3\xCF\xD3\n\n")?;

        let mut xref_offsets = BTreeMap::new();
        for (&id, data) in &self.objects {
            let object_start_abs = writer.stream_position()?;
            xref_offsets.insert(id, object_start_abs - pdf_start_pos);
            write!(writer, "{} 0 obj\n", id.0)?;
            data.write_content(writer)?;
            writer.write_all(b"\nendobj\n")?;
        }

        let max_obj_id = self.objects.keys()
            .map(|id| id.0)
            .max()
            .expect("no objects");

        let xref_abs = writer.stream_position()?;
        writer.write_all(b"xref\n")?;
        write!(writer, "0 {}\n", max_obj_id + 1)?;
        let mut cur_obj_id = 0;
        for (&id, &xref_offset) in &xref_offsets {
            while cur_obj_id < id.0 {
                write!(writer, "{:010} 65535 f\r\n", xref_offset)?;
                cur_obj_id += 1;
            }
            write!(writer, "{:010} 00000 n\r\n", xref_offset)?;
            cur_obj_id += 1;
        }

        let root_obj_id = self.objects.iter()
            .filter(|(_id, data)| matches!(data, Content::Catalog(_)))
            .map(|(id, _data)| *id)
            .nth(0)
            .expect("no catalog object found");

        writer.write_all(b"trailer\n")?;
        write!(writer, "<</Size {}/Root {} 0 R>>\n", max_obj_id + 1, root_obj_id.0)?;
        write!(writer, "startxref\n{}\n%%EOF\n", xref_abs - pdf_start_pos)?;
        Ok(())
    }
}

/// A PDF object whose content can be written to a byte stream.
pub trait Object {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error>;
}

/// The contents of a PDF object.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Content {
    Catalog(Catalog),
    Pages(Pages),
    Page(Page),
    PageContents(PageContents),
    ImageXObject(ImageXObject),
    StandardFont(StandardFont),
}
impl Object for Content {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        match self {
            Self::Catalog(catalog) => catalog.write_content(writer),
            Self::Pages(pages) => pages.write_content(writer),
            Self::Page(page) => page.write_content(writer),
            Self::PageContents(page_contents) => page_contents.write_content(writer),
            Self::ImageXObject(image_xobject) => image_xobject.write_content(writer),
            Self::StandardFont(font) => font.write_content(writer),
        }
    }
}

/// A Catalog PDF object, the topmost object in the hierarchical structure.
///
/// A catalog contains references to the root node of the page tree, to the tree of bookmarks, etc.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Catalog {
    pub root_pages_id: PdfId,
    pub lang: Option<String>,
}
impl Object for Catalog {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(b"<</Type/Catalog")?;
        write!(writer, "/Pages {} 0 R", self.root_pages_id.0)?;
        if let Some(lang) = self.lang.as_ref() {
            writer.write_all(b"/Lang")?;
            write_pdf_string(&lang, writer)?;
        }
        writer.write_all(b">>")?;
        Ok(())
    }
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
impl Object for Pages {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(b"<</Type/Pages")?;

        writer.write_all(b"/Kids[")?;
        let mut first_child = true;
        for child_id in &self.children {
            if first_child {
                first_child = false;
            } else {
                writer.write_all(b" ")?;
            }
            write!(writer, "{} 0 R", child_id.0)?;
        }
        writer.write_all(b"]")?;

        write!(writer, "/Count {}", self.children.len())?;

        writer.write_all(b">>")?;
        Ok(())
    }
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
impl Object for Page {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(b"<</Type/Page")?;
        write!(writer, "/Parent {} 0 R", self.parent.0)?;

        writer.write_all(b"/Resources<</ProcSet[/PDF/Text/ImageB/ImageC/ImageI]")?;
        if self.xobject_refs.len() > 0 {
            writer.write_all(b"/XObject<<")?;
            for (name, id) in &self.xobject_refs {
                write_pdf_name(name, writer)?;
                write!(writer, " {} 0 R", id.0)?;
            }
            writer.write_all(b">>")?;
        }
        if self.font_refs.len() > 0 {
            writer.write_all(b"/Font<<")?;
            for (name, id) in &self.xobject_refs {
                write_pdf_name(name, writer)?;
                write!(writer, " {} 0 R", id.0)?;
            }
            writer.write_all(b">>")?;
        }
        writer.write_all(b">>")?;

        write!(writer, "/MediaBox[0 0 {} {}]", self.width_pt, self.height_pt)?;
        if let Some(contents) = self.contents {
            write!(writer, "/Contents {} 0 R", contents.0)?;
        }

        writer.write_all(b">>")?;
        Ok(())
    }
}

/// A stream describing the contents of a PDF page.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PageContents {
    /// The drawing commands, in PDF's postfix operator notation.
    ///
    /// Since we are using inline UTF-16 strings, it's better to consider this a binary string.
    pub commands: Vec<u8>,
}
impl Object for PageContents {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(b"<<")?;
        write!(writer, "/Length {}", self.commands.len())?;
        writer.write_all(b">>")?;

        write_pdf_stream(&self.commands, writer)?;
        Ok(())
    }
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
impl Object for ImageXObject {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(b"<</Type/XObject/Subtype/Image")?;
        write!(writer, "/Width {}", self.width)?;
        write!(writer, "/Height {}", self.height)?;

        writer.write_all(b"/ColorSpace")?;
        write_pdf_name(self.color_space, writer)?;

        write!(writer, "/BitsPerComponent {}", self.bits_per_component)?;
        write!(writer, "/Interpolate {}", if self.interpolate { "true" } else { "false" })?;

        if self.data_filters.len() > 0 {
            writer.write_all(b"/Filter[")?;
            for data_filter in &self.data_filters {
                write_pdf_name(data_filter, writer)?;
            }
            writer.write_all(b"]")?;
        }

        write!(writer, "/Length {}", self.data.len())?;

        writer.write_all(b">>")?;

        write_pdf_stream(&self.data, writer)?;
        Ok(())
    }
}

/// One of the standard 14 fonts.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StandardFont {
    /// The PDF name of the font.
    pub name: String,
}
impl Object for StandardFont {
    fn write_content<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(b"<</Type/Font/Subtype/Type1")?;
        writer.write_all(b"/BaseFont")?;
        write_pdf_name(&self.name, writer)?;
        writer.write_all(b">>")?;
        Ok(())
    }
}

/// Writes out a textual string in PDF format.
///
/// The string is wrapped in parentheses (`(` and `)`), encoded in UTF-16BE with BOM, and all
/// backslashes and parentheses are escaped with a preceding backslash.
pub fn write_pdf_string<W: Write>(string: &str, writer: &mut W) -> Result<(), io::Error> {
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

/// Writes out a PDF name.
///
/// The string starts with a slash (`/`). The number sign (`#`) as well as regular characters
/// outside the range of `!` to `~` are encoded as a hex escape: the number sign and two uppercase
/// hexadecimal digits.
///
/// Regular characters are those that are neither white-space nor delimiter characters. White-space
/// characters are NUL (U+0000), tab (U+0009), line feed (U+000A), form feed (U+000C), carriage
/// return (U+000D) and space (U+0020). Delimiter characters are all types of brackets (`()<>[]{}`),
/// the slash (`/`) and the percent sign (`%`).
pub fn write_pdf_name<W: Write>(name: &str, writer: &mut W) -> Result<(), io::Error> {
    const SORTED_BYTES_TO_ESCAPE: [u8; 17] = [
        // white space characters
        0x00, 0x09, 0x0A, 0x0C, 0x0D, 0x20,
        // the escape character
        b'#',
        // delimiter characters
        b'%', b'(', b')', b'/', b'<', b'>', b'[', b']', b'{', b'}',
    ];
    writer.write_all(b"/")?;
    for &b in name.as_bytes() {
        if SORTED_BYTES_TO_ESCAPE.binary_search(&b).is_ok() {
            write!(writer, "#{:02X}", b)?;
        } else {
            writer.write_all(&[b])?;
        }
    }
    Ok(())
}

/// Writes out a delimited PDF stream.
pub fn write_pdf_stream<W: Write>(data: &[u8], writer: &mut W) -> Result<(), io::Error> {
    writer.write_all(b"\nstream\n")?;
    writer.write_all(data)?;
    writer.write_all(b"\nendstream")?;
    Ok(())
}
