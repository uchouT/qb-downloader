//! qb request, which will attach cookie
//! should never be called without login success

use nyquest::header;

use crate::{
    qb::{self, QB, QbError},
    request::{self, MyRequest, MyRequestBuilder, RequestError, Res},
};

pub(super) struct QbRequest;

pub(super) struct QbRequestBuilder {
    inner: crate::request::MyRequestBuilderImpl,
}

impl MyRequestBuilder for QbRequestBuilder {
    type Err = QbError;
    fn basic_auth(self, username: &str, password: &str) -> Self {
        Self {
            inner: self.inner.basic_auth(username, password),
        }
    }

    fn form<F, K, V>(self, fields: F) -> Self
    where
        F: IntoIterator<Item = (K, V)>,
        K: Into<std::borrow::Cow<'static, str>>,
        V: Into<std::borrow::Cow<'static, str>>,
    {
        Self {
            inner: self.inner.form(fields),
        }
    }

    fn json<T: serde::Serialize>(self, value: T) -> Self {
        Self {
            inner: self.inner.json(value),
        }
    }

    fn header(
        self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        value: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self {
            inner: self.inner.header(name, value),
        }
    }

    fn multipart(self, parts: crate::request::multipart::MultipartBuilder) -> Self {
        Self {
            inner: self.inner.multipart(parts),
        }
    }

    fn query<T: serde::Serialize>(self, input: T) -> Self {
        Self {
            inner: self.inner.query(input),
        }
    }

    async fn send(self) -> Result<Res, QbError> {
        let sender = self.inner.clone();
        let cookie = QB.get().unwrap().load().cookie.clone().unwrap();

        match self.inner.header(header::COOKIE, cookie).send().await {
            Err(e) => {
                let err = RequestError::from(e);
                if let RequestError::Response(code) = err
                    && code == 403
                {
                    qb::login().await;
                    let cookie = QB.get().unwrap().load().cookie.clone().unwrap();
                    sender
                        .header(header::COOKIE, cookie)
                        .send()
                        .await
                        .map_err(|e| {
                            if let RequestError::Response(code) = e
                                && code == 403
                            {
                                QbError::NotLogin
                            } else {
                                QbError::from(e)
                            }
                        })
                } else {
                    Err(QbError::from(err))
                }
            }
            Ok(res) => Ok(res),
        }
    }
}
impl MyRequest for QbRequest {
    type RequestBuilder = QbRequestBuilder;
    fn get(url: impl Into<std::borrow::Cow<'static, str>>) -> Self::RequestBuilder {
        Self::RequestBuilder {
            inner: request::get(url),
        }
    }

    fn post(url: impl Into<std::borrow::Cow<'static, str>>) -> Self::RequestBuilder {
        Self::RequestBuilder {
            inner: request::post(url),
        }
    }
}
