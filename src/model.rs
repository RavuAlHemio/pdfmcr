//! Structures representing data within pdfmcr.


use std::io::{self, Write};

use from_to_repr::FromToRepr;
use serde::{Deserialize, Serialize};

use crate::pdf::write_pdf_string;


/// A pdfmcr file: a list of pages with annotations.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct File {
    /// The pages and their annotations.
    pub pages: Vec<Page>,

    /// The default language for this document, as a BCP 47 language tag.
    pub default_language: Option<String>,
}


/// A single page with annotations.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Page {
    /// The scanned image of the page, in JPEG format.
    pub scanned_image: JpegImage,

    /// The annotations on the page, in reading order.
    ///
    /// Annotations represent the actual content.
    pub annotations: Vec<Annotation>,

    /// The artifacts on the page, in reading order.
    ///
    /// Artifacts represent elements that are printed on the page but which are not the actual page
    /// content, e.g. page numbers.
    pub artifacts: Vec<Artifact>,
}


/// A JPEG image.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct JpegImage {
    /// The bit depth of the image.
    pub bit_depth: u8,

    /// The width of the image, in pixels.
    pub width: u16,

    /// The height of the image, in pixels.
    pub height: u16,

    /// The color space in which the image is stored.
    pub color_space: ColorSpace,

    /// The unit in which the pixel density is specified.
    pub density_unit: DensityUnit,

    /// The pixel density in the horizontal direction (across the width).
    pub density_x: u16,

    /// The pixel density in the vertical direction (across the height).
    pub density_y: u16,

    /// The actual full data of the image, in JFIF or Exif formats.
    ///
    /// JFIF and Exif are the most common representations of JPEG files.
    pub data: Vec<u8>,
}
impl JpegImage {
    pub fn write_object_body<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(b"<</Type/XObject/Subtype/Image")?;
        write!(writer, "/Width {}", self.width)?;
        write!(writer, "/Height {}", self.height)?;
        write!(writer, "/ColorSpace{}", self.color_space.as_pdf_name())?;
        write!(writer, "/BitsPerComponent {}", self.color_space.as_pdf_name())?;
        writer.write_all(b"/Filter[/DCTDecode]")?;
        write!(writer, "/Length {}", self.data.len())?;
        writer.write_all(b">>\nstream\n")?;
        writer.write_all(&self.data)?;
        writer.write_all(b">>\nendstream\n")?;
        Ok(())
    }
}

/// The color space of an image or graphics system.
#[derive(Clone, Copy, Debug, Deserialize, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[repr(u8)]
pub enum ColorSpace {
    Grayscale = 1,
    Rgb = 3,
    Cmyk = 4,
}
impl ColorSpace {
    pub fn as_pdf_name(&self) -> &'static str {
        match self {
            Self::Grayscale => "/DeviceGray",
            Self::Rgb => "/DeviceRGB",
            Self::Cmyk => "/DeviceCMYK",
        }
    }
}

/// The unit in which pixel (dot) density is specified.
#[derive(Clone, Copy, Debug, Deserialize, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[repr(u8)]
pub enum DensityUnit {
    NoUnit = 0,
    DotsPerInch = 1,
    DotsPerCentimeter = 2,
}
impl DensityUnit {
    /// Uses the density unit to convert the pixel count and density value into points (1/72 in).
    ///
    /// Returns `None` for [`DensityUnit::NoUnit`].
    pub fn try_to_points(&self, pixel_count: u16, density: u16) -> Option<u64> {
        match self {
            Self::NoUnit => None,
            Self::DotsPerInch => Some(u64::from(pixel_count) / (72 * u64::from(density))),
            Self::DotsPerCentimeter => Some(25 * u64::from(pixel_count) / (4572 * u64::from(density))),
        }
    }
}


/// A single cohesive annotation on the page that represents actual content.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Annotation {
    /// The horizontal coordinate of the annotation, from the left edge of the page.
    pub left: u64,

    /// The vertical coordinate of the annotation, from the bottom edge of the page.
    pub bottom: u64,

    /// The elements of the annotation.
    pub elements: Vec<TextChunk>,
}
impl Annotation {
    pub fn write_drawing_commands<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(b" BT")?;
        write!(writer, " 1 0 0 1 {} {} Tm", self.left, self.bottom)?;
        for element in &self.elements {
            element.write_drawing_commands(&mut writer)?;
        }
        writer.write_all(b" ET")?;
        Ok(())
    }
}


/// A single cohesive annotation on the page that represents a non-content element.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Artifact {
    /// The type of artifact represented by this object.
    pub kind: ArtifactKind,

    /// The artifact represented as an annotation.
    pub annotation: Annotation,
}
impl Artifact {
    pub fn write_drawing_commands<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        write!(writer, "/Artifact<</Type{}>>BDC", self.kind.as_pdf_name())?;
        self.annotation.write_drawing_commands(&mut writer)?;
        writer.write_all(b" EDC")?;
        Ok(())
    }
}

/// The type of non-content element represented by an [`Artifact`].
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ArtifactKind {
    /// Running heads, folios (page numbers), Bates numbering, etc.
    Pagination,

    /// Design elements such as footnote rules or decorative ornaments.
    Layout,

    /// Cut marks, color bars, etc.
    Page,

    /// Background elements repeated on every page.
    Background,
}
impl ArtifactKind {
    pub const fn as_pdf_name(&self) -> &'static str {
        match self {
            Self::Pagination => "/Pagination",
            Self::Layout => "/Layout",
            Self::Page => "/Page",
            Self::Background => "/Background",
        }
    }
}

/// A chunk of text.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TextChunk {
    /// The text itself.
    pub text: String,

    /// The font variant to use.
    pub font_variant: FontVariant,

    /// The size of the font, in points (72ths of an inch).
    pub font_size: u64,

    /// Character spacing.
    pub character_spacing: u64,

    /// Word spacing.
    pub word_spacing: u64,

    /// Leading (additional line spacing).
    ///
    /// Pronunciation tip: "leading" is derived from the chemical element lead (Pb) and pronounced
    /// accordingly.
    pub leading: i64,

    /// The language of this chunk, as a BCP 47 language tag, if it differs from the default
    /// document language.
    pub language: Option<String>,

    /// Alternate text describing this chunk.
    ///
    /// It may seem absurd to provide alternate text for textual elements, but this can be done to
    /// provide a verbal explanation if the chunk's layout, not just its textual content, carries
    /// meaning.
    pub alternate_text: Option<String>,

    /// The actual text, free of typographical specialities, represented by this string.
    ///
    /// This may be useful for reflowing text or for screen readers. For example, older German
    /// spelling rules prescribe that "ck" be hyphenated "k-k" (e.g. "Nacken" becomes "Nak-ken") and
    /// the text "k-⏎k" can be annotated with "ck" as the actual text.
    pub actual_text: Option<String>,

    /// The expanded phrase if the text is an abbreviation.
    ///
    /// This is useful for screen readers since the meaning of some abbreviations is
    /// context-specific (e.g. "Dr." for "Doctor" in front of a person's name and "Drive" in the
    /// name of a street).
    pub expansion: Option<String>,
}
impl TextChunk {
    pub fn write_drawing_commands<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        let need_span =
            self.language.is_some()
            || self.alternate_text.is_some()
            || self.actual_text.is_some()
            || self.expansion.is_some();

        // pick the correct font
        write!(writer, "/F{} {} Tf", self.font_variant.as_index(), self.font_size)?;

        // set some spacing settings
        if self.character_spacing > 0 {
            write!(writer, " {} Tc", self.character_spacing)?;
        }
        if self.word_spacing > 0 {
            write!(writer, " {} Tw", self.word_spacing)?;
        }
        if self.leading > 0 {
            write!(writer, " {} TL", self.leading)?;
        }

        // do not actually output the characters
        // (neither fill nor stroke nor influence the clipping path)
        write!(writer, " 3 Tr")?;

        if need_span {
            writer.write_all(b"/Span<<")?;
            if let Some(language) = self.language.as_ref() {
                writer.write_all(b"/Lang")?;
                write_pdf_string(language, &mut writer)?;
            }
            if let Some(alt_text) = self.alternate_text.as_ref() {
                writer.write_all(b"/Alt")?;
                write_pdf_string(alt_text, &mut writer)?;
            }
            if let Some(actual) = self.actual_text.as_ref() {
                writer.write_all(b"/ActualText")?;
                write_pdf_string(actual, &mut writer)?;
            }
            if let Some(expansion) = self.expansion.as_ref() {
                writer.write_all(b"/E")?;
                write_pdf_string(expansion, &mut writer)?;
            }
            writer.write_all(b">>BDC")?;
        }

        write_pdf_string(&self.text, &mut writer)?;
        writer.write_all(b"Tj")?;

        if need_span {
            writer.write_all(b" EMC")?;
        }
        Ok(())
    }
}

/// The variant of a font.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[repr(u8)]
pub enum FontVariant {
    Regular,
    Italic,
    Bold,
    BoldItalic,
}
impl FontVariant {
    pub const fn as_index(&self) -> u8 {
        match self {
            Self::Regular => 0b00,
            Self::Italic => 0b01,
            Self::Bold => 0b10,
            Self::BoldItalic => 0b11,
        }
    }
}
