//! Structures representing data within pdfmcr.


/// A pdfmcr file: a list of pages with annotations.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct File {
    pub pages: Vec<Page>,
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
    /// The elements of the annotation.
    pub elements: Vec<AnnotationElement>,
}

/// A single cohesive annotation on the page that represents a non-content element.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Artifact {
    /// The type of artifact represented by this object.
    ///
    /// Preferfined PDF values are `Pagination`
    pub kind: String,

    /// The elements of the annotation.
    pub elements: Vec<AnnotationElement>,
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

/// A single element of an annotation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AnnotationElement {

}
