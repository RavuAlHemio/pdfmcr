use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error as _;


/// An error pertaining to an image path.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Image path is empty.
    Empty,

    /// Image path contains a colon (`:`) character.
    ///
    /// Path components in an image path must be separated by a forward slash.
    ContainsColon,

    /// Image path contains a backslash (`\\`) character.
    ///
    /// Path components in an image path must be separated by a forward slash.
    ContainsBackslash,

    /// Image path contains an empty component.
    ///
    /// This is mostly due to a leading, trailing or double slash.
    ContainsEmptyComponent,

    /// Image path contains a `..` component.
    ///
    /// To prevent traversal out of the designated folder, such components are forbidden.
    ContainsDotDotComponent,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Empty
                => write!(f, "image path is empty"),
            Error::ContainsColon
                => write!(f, "image path contains colon"),
            Error::ContainsBackslash
                => write!(f, "image path contains backslash"),
            Error::ContainsEmptyComponent
                => write!(f, "image path contains an empty component"),
            Error::ContainsDotDotComponent
                => write!(f, "image path contains a \"..\" component"),
        }
    }
}
impl std::error::Error for Error {
}

/// A path to an image.
///
/// An image path is a string that complies with these rules:
/// * It is not empty.
/// * It contains neither a colon (`:`, U+003A) nor a backslash (`\`, U+005C).
/// * When split at slash (`/`, U+002F) characters, none of the components is empty.
/// * When split at slash characters, none of the components equals `..` (the sequence of twice the
///   character U+002E).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImagePath(String);
impl ImagePath {
    pub fn as_str(&self) -> &str { self.0.as_str() }

    pub fn to_os_path(&self, base_path: &Path) -> PathBuf {
        let relative_path = self.to_relative_os_path();
        base_path.join(&relative_path)
    }

    pub fn to_relative_os_path(&self) -> String {
        self.as_str().replace("/", std::path::MAIN_SEPARATOR_STR)
    }
}
impl fmt::Display for ImagePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl FromStr for ImagePath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 {
            return Err(Error::Empty);
        }
        if s.contains(':') {
            return Err(Error::ContainsColon);
        }
        if s.contains('\\') {
            return Err(Error::ContainsBackslash);
        }

        if s.split('/').any(|component| component.len() == 0) {
            return Err(Error::ContainsEmptyComponent);
        }
        if s.split('/').any(|component| component == "..") {
            return Err(Error::ContainsDotDotComponent);
        }

        // good enough
        Ok(ImagePath(s.to_owned()))
    }
}
impl AsRef<str> for ImagePath {
    fn as_ref(&self) -> &str { self.as_str() }
}
impl<'de> Deserialize<'de> for ImagePath {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string)
            .map_err(|e| D::Error::custom(e))
    }
}
impl Serialize for ImagePath {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let str = self.as_str();
        str.serialize(serializer)
    }
}
