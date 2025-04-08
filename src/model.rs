//! Structures representing data within pdfmcr.


/// A pdfmcr file: a list of pages with annotations.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct File {
    /// The pages and their annotations.
    pub pages: Vec<Page>,

    /// The default language for this document, as a BCP 47 language tag.
    pub default_language: Option<String>,
}


/// A single page with annotations.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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


/// A JPEG image, represented as bytes in JFIF or Exif format.
///
/// JFIF and Exif are the most common representations of JPEG files.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JpegImage(Vec<u8>);
impl AsRef<[u8]> for JpegImage {
    fn as_ref(&self) -> &[u8] { &self.0 }
}
impl From<JpegImage> for Vec<u8> {
    fn from(value: JpegImage) -> Self { value.0 }
}


/// A single cohesive annotation on the page that represents actual content.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Annotation {
    /// The horizontal coordinate of the annotation, from the left edge of the page.
    pub left: u64,

    /// The vertical coordinate of the annotation, from the bottom edge of the page.
    pub bottom: u64,

    /// The elements of the annotation.
    pub elements: Vec<TextChunk>,
}

/// A single cohesive annotation on the page that represents a non-content element.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Artifact {
    /// The type of artifact represented by this object.
    ///
    /// Preferfined PDF values are `Pagination`
    pub kind: String,

    /// The artifact represented as an annotation.
    pub annotation: Annotation,
}

/// The type of non-content element represented by an [`Artifact`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

/// A chunk of text.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextChunk {
    /// The text itself.
    pub text: String,

    /// Whether the text is typeset in bold font.
    pub bold: bool,

    /// Whether the text is typeset in italic font.
    pub italic: bool,

    /// The size of the font, in points (72ths of an inch).
    pub font_size: u64,

    /// Character spacing.
    pub character_spacing: u64,

    /// Word spacing.
    pub word_spacing: u64,

    /// The language of this chunk, as a BCP 47 language tag, if it differs from the default
    /// document language.
    pub language: Option<String>,

    /// Alternate text describing this element.
    pub alternate_text: Option<String>,

    /// TODO: describe
    pub actual_text: Option<String>,

    // TODO
}
