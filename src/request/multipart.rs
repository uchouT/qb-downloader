use std::{borrow::Cow, path::Path};
use mime_guess::Mime;
use nyquest::{PartBody, r#async::Part};

use crate::request::RequestError;

/// A multipart builder, which contains infomation to build a multipart
#[derive(Clone)]
pub struct MultipartBuilder {
    inner: Vec<PartSpec>,
}

impl MultipartBuilder {
    pub fn new() -> Self {
        MultipartBuilder { inner: vec![] }
    }

    /// Add text part
    pub fn text(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.inner.push(PartSpec {
            name: name.into(),
            kind: PartKind::Text(value.into()),
        });
        self
    }

    /// Add file part by bytes
    pub fn bytes(
        mut self,
        name: impl Into<Cow<'static, str>>,
        bytes: impl Into<Cow<'static, [u8]>>,
        filename: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.inner.push(PartSpec {
            name: name.into(),
            kind: PartKind::Bytes {
                value: bytes.into(),
                filename: filename.into(),
            },
        });
        self
    }

    /// Add file part by path
    pub fn path(
        mut self,
        name: impl Into<Cow<'static, str>>,
        path: impl Into<Cow<'static, Path>>,
    ) -> Self {
        self.inner.push(PartSpec {
            name: name.into(),
            kind: PartKind::Path(path.into()),
        });
        self
    }

    pub(super) async fn into_multipart(self) -> Result<Multipart, RequestError> {
        let mut parts = Vec::with_capacity(self.inner.len());
        for part in self.inner.into_iter() {
            if let PartKind::Path(_) = part.kind {
                parts.push(part.into_part_async().await?);
            } else {
                parts.push(part.into_part());
            }
        }

        Ok(Multipart { parts })
    }
}

pub(super) struct Multipart {
    parts: Vec<Part>,
}

impl IntoIterator for Multipart {
    type Item = Part;
    type IntoIter = std::vec::IntoIter<Part>;

    fn into_iter(self) -> Self::IntoIter {
        self.parts.into_iter()
    }
}

/// A file part for multipart/form-data, which is used to build [`Part`]
struct FilePart {
    bytes: Cow<'static, [u8]>,
    mime: Mime,
    filename: Cow<'static, str>,
}

impl FilePart {
    /// Create a file part from bytes and filename, the mime type is guessed from the filename
    fn bytes(bytes: impl Into<Cow<'static, [u8]>>, filename: impl Into<Cow<'static, str>>) -> Self {
        let filename = filename.into();
        let mime = mime_guess::from_path(Path::new(filename.as_ref())).first_or_octet_stream();
        Self {
            bytes: bytes.into(),
            mime,
            filename,
        }
    }

    /// Create a file part from a file path, the mime type is guessed from the file extension
    async fn path(path: &Path) -> Result<Self, RequestError> {
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

    /// turn into [`Part`]
    fn into_part(self, name: impl Into<Cow<'static, str>>) -> Part {
        Part::new_with_content_type(
            name.into(),
            self.mime.to_string(),
            PartBody::bytes(self.bytes),
        )
        .with_filename(self.filename)
    }
}

/// Contains infomation to build a [`Part`]
#[derive(Clone)]
struct PartSpec {
    name: Cow<'static, str>,
    kind: PartKind,
}

#[derive(Clone)]
enum PartKind {
    Text(Cow<'static, str>),
    Bytes {
        value: Cow<'static, [u8]>,
        filename: Cow<'static, str>,
    },
    Path(Cow<'static, Path>),
}

impl PartSpec {
    /// Turn into [`Part`]
    /// # Precondition
    /// `self.kind` is not [`PartKind::Path`]
    fn into_part(self) -> Part {
        match self.kind {
            PartKind::Text(value) => {
                Part::new_with_content_type(self.name, "text/plain", PartBody::text(value))
            }
            PartKind::Bytes { value, filename } => {
                FilePart::bytes(value, filename).into_part(self.name)
            }
            _ => panic!("Path should be handled in into_part_async"),
        }
    }

    async fn into_part_async(self) -> Result<Part, RequestError> {
        let part = match self.kind {
            PartKind::Path(path) => FilePart::path(&path).await?.into_part(self.name),
            _ => self.into_part(),
        };
        Ok(part)
    }
}
