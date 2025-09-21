use std::{borrow::Cow, path::Path};

use mime_guess::Mime;
use nyquest::{PartBody, r#async::Part};

use crate::request::RequestError;
pub struct Multipart {
    parts: Vec<Part>,
}

impl IntoIterator for Multipart {
    type Item = Part;
    type IntoIter = std::vec::IntoIter<Part>;

    fn into_iter(self) -> Self::IntoIter {
        self.parts.into_iter()
    }
}

impl Multipart {
    /// create a new empty multipart
    pub fn new() -> Self {
        Self { parts: vec![] }
    }

    /// add a text part
    pub fn text(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        let part = Part::new_with_content_type(name, "text/plain", PartBody::text(value));
        self.parts.push(part);
        self
    }

    /// add a file part
    pub fn file(mut self, name: impl Into<Cow<'static, str>>, file: FilePart) -> Self {
        let content_type = file.mime.to_string();
        let part = Part::new_with_content_type(name, content_type, PartBody::bytes(file.bytes))
            .with_filename(file.filename);
        self.parts.push(part);
        self
    }
}

/// A file part for multipart/form-data
pub struct FilePart {
    bytes: Cow<'static, [u8]>,
    mime: Mime,
    filename: Cow<'static, str>,
}

impl FilePart {
    /// Create a file part from bytes and filename, the mime type is guessed from the filename
    pub fn bytes(
        bytes: impl Into<Cow<'static, [u8]>>,
        filename: impl Into<Cow<'static, str>>,
    ) -> Self {
        let filename = filename.into();
        let mime = mime_guess::from_path(Path::new(filename.as_ref())).first_or_octet_stream();
        Self {
            bytes: bytes.into(),
            mime,
            filename,
        }
    }

    /// Create a file part from a file path, the mime type is guessed from the file extension
    pub async fn path(path: &Path) -> Result<Self, RequestError> {
        let bytes = tokio::fs::read(path).await?;
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        Ok(Self {
            bytes: bytes.into(),
            mime,
            filename: filename.into(),
        })
    }
}
