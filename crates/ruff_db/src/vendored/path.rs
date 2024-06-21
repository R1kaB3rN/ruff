use std::fmt;
use std::ops::Deref;
use std::path;

use itertools::Itertools;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct VendoredPath(str);

impl VendoredPath {
    pub fn to_path_buf(&self) -> VendoredPathBuf {
        VendoredPathBuf(self.0.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_std_path(&self) -> &path::Path {
        path::Path::new(&self.0)
    }

    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

#[derive(Debug)]
pub struct UnsupportedComponentError(String);

impl fmt::Display for UnsupportedComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported component in a vendored path: {:?}", self.0)
    }
}

impl std::error::Error for UnsupportedComponentError {}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Default)]
pub struct VendoredPathBuf(String);

impl VendoredPathBuf {
    pub fn new(path: &camino::Utf8Path) -> Result<Self, UnsupportedComponentError> {
        let mut normalized_parts = camino::Utf8PathBuf::new();

        // Allow the `RootDir` component, but only if it is at the very start of the string.
        let mut components = path.components().peekable();
        if let Some(camino::Utf8Component::RootDir) = components.peek() {
            components.next();
        }

        for component in components {
            match component {
                camino::Utf8Component::Normal(part) => normalized_parts.push(part),
                camino::Utf8Component::CurDir => continue,
                camino::Utf8Component::ParentDir => {
                    normalized_parts.pop();
                }
                unsupported => return Err(UnsupportedComponentError(unsupported.to_string())),
            }
        }
        Ok(Self(normalized_parts.into_iter().join("/")))
    }

    pub fn as_path(&self) -> &VendoredPath {
        let path = self.0.as_str();
        // SAFETY: VendoredPath is marked as #[repr(transparent)] so the conversion from a
        // *const str to a *const VendoredPath is valid.
        unsafe { &*(path as *const str as *const VendoredPath) }
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<VendoredPath> for VendoredPathBuf {
    fn as_ref(&self) -> &VendoredPath {
        self.as_path()
    }
}

impl AsRef<VendoredPath> for VendoredPath {
    #[inline]
    fn as_ref(&self) -> &VendoredPath {
        self
    }
}

impl AsRef<path::Path> for VendoredPath {
    #[inline]
    fn as_ref(&self) -> &path::Path {
        path::Path::new(&self.0)
    }
}

impl Deref for VendoredPathBuf {
    type Target = VendoredPath;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl<'a> TryFrom<&'a camino::Utf8Path> for VendoredPathBuf {
    type Error = UnsupportedComponentError;

    fn try_from(value: &'a camino::Utf8Path) -> Result<Self, Self::Error> {
        VendoredPathBuf::new(value)
    }
}

impl<'a> TryFrom<&'a str> for VendoredPathBuf {
    type Error = UnsupportedComponentError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        VendoredPathBuf::new(camino::Utf8Path::new(value))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VendoredPathConstructionError<'a> {
    #[error("Could not convert {0} to a UTF-8 string")]
    InvalidUTF8(&'a path::Path),
    #[error("{0}")]
    UnsupporteComponent(#[from] UnsupportedComponentError),
}

impl<'a> TryFrom<&'a path::Path> for VendoredPathBuf {
    type Error = VendoredPathConstructionError<'a>;

    fn try_from(value: &'a path::Path) -> Result<Self, Self::Error> {
        let Some(path_str) = value.to_str() else {
            return Err(VendoredPathConstructionError::InvalidUTF8(value));
        };
        Ok(VendoredPathBuf::new(camino::Utf8Path::new(path_str))?)
    }
}
