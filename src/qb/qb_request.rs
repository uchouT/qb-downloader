//! qb request, which will attach cookie
//! should never be called without login success

use nyquest::header;

use crate::{
    qb::{QB, QbError},
    request::{
        MyRequest, MyRequestBuilder, MyRequestBuilderAccessor, MyRequestImpl, RequestError, Res,
    },
};

pub(super) struct QbRequest;

pub(super) struct QbRequestBuilder {
    inner: crate::request::MyRequestBuilderImpl,
}

impl MyRequestBuilderAccessor for QbRequestBuilder {
    fn url_mut(&mut self) -> std::borrow::Cow<'static, str> {
        self.inner.url_mut()
    }

    fn method(&self) -> crate::request::Method {
        self.inner.method()
    }

    fn headers_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<
        std::borrow::Cow<'static, str>,
        std::borrow::Cow<'static, str>,
    > {
        self.inner.headers_mut()
    }

    fn take_body(&mut self) -> Option<crate::request::MyBody> {
        self.inner.take_body()
    }

    fn body_mut(&mut self) -> &mut Option<crate::request::MyBody> {
        self.inner.body_mut()
    }
}

impl MyRequestBuilder for QbRequestBuilder {
    type Err = QbError;
    async fn send(&mut self) -> Result<Res, QbError> {
        self.inner.send().await.map_err(|e| {
            let err = RequestError::from(e);
            if let RequestError::Response(code) = err
                && code == 403
            {
                QbError::NotLogin
            } else {
                QbError::from(err)
            }
        })
    }
}
impl MyRequest for QbRequest {
    type RequestBuilder = QbRequestBuilder;
    fn get(url: impl Into<std::borrow::Cow<'static, str>>) -> Self::RequestBuilder {
        let cookie = QB.get().unwrap().load().cookie.clone().unwrap();
        let mut inner = MyRequestImpl::get(url);
        inner.header(header::COOKIE, cookie);
        Self::RequestBuilder { inner }
    }

    fn post(url: impl Into<std::borrow::Cow<'static, str>>) -> Self::RequestBuilder {
        let cookie = QB.get().unwrap().load().cookie.clone().unwrap();
        let mut inner = MyRequestImpl::post(url);
        inner.header(header::COOKIE, cookie);
        Self::RequestBuilder { inner }
    }
}
