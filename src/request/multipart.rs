use std::{borrow::Cow, path::Path};

use futures_util::future::try_join_all;
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

/// A file part for multipart/form-data, which is used to build [`Part`]
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

#[derive(Clone)]
pub struct MultipartBuilder {
    inner: Vec<PartBuilder>,
}

#[derive(Clone)]
pub struct PartBuilder {
    name: Cow<'static, str>,
    kind: PartKind,
}

#[derive(Clone)]
pub enum PartKind {
    Text(Cow<'static, str>),
    File(FilePartKind),
}

#[derive(Clone)]
pub enum FilePartKind {
    Bytes {
        value: Cow<'static, [u8]>,
        filename: Cow<'static, str>,
    },
    Path(Cow<'static, Path>),
}

impl FilePartKind {
    async fn into_part(self, name: impl Into<Cow<'static, str>>) -> Result<Part, RequestError> {
        let part = match self {
            Self::Bytes { value, filename } => FilePart::bytes(value, filename),
            Self::Path(path) => FilePart::path(&path).await?,
        };
        let file_part = Part::new_with_content_type(
            name.into(),
            part.mime.to_string(),
            PartBody::bytes(part.bytes),
        )
        .with_filename(part.filename);
        Ok(file_part)
    }
}

impl PartBuilder {
    pub async fn into_part(self) -> Result<Part, RequestError> {
        let part = match self.kind {
            PartKind::Text(value) => {
                Part::new_with_content_type(self.name, "text/plain", PartBody::text(value))
            }
            PartKind::File(file_part) => file_part.into_part(self.name).await?,
        };
        Ok(part)
    }
}

impl MultipartBuilder {
    pub fn new() -> Self {
        MultipartBuilder { inner: vec![] }
    }

    pub fn text(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.inner.push(PartBuilder {
            name: name.into(),
            kind: PartKind::Text(value.into()),
        });
        self
    }

    pub fn bytes(
        mut self,
        name: impl Into<Cow<'static, str>>,
        bytes: impl Into<Cow<'static, [u8]>>,
        filename: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.inner.push(PartBuilder {
            name: name.into(),
            kind: PartKind::File(FilePartKind::Bytes {
                value: bytes.into(),
                filename: filename.into(),
            }),
        });
        self
    }

    pub fn path(
        mut self,
        name: impl Into<Cow<'static, str>>,
        path: impl Into<Cow<'static, Path>>,
    ) -> Self {
        self.inner.push(PartBuilder {
            name: name.into(),
            kind: PartKind::File(FilePartKind::Path(path.into())),
        });
        self
    }

    pub async fn into_multipart(self) -> Result<Multipart, RequestError> {
        let s = self
            .inner
            .into_iter()
            .map(|part_builder| part_builder.into_part());
        let parts = try_join_all(s).await?;
        Ok(Multipart { parts })
    }
}
