// TODO: test qb and uploader is ok
//! check if is authenticated

use super::{Action, BoxBody, Req, ServerResult};
use crate::server::{ResultResponse, full};
use hyper::{Response, StatusCode};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct Asset;
const ROOT: &str = "index.html";
pub struct AssetAPI;
impl Action for AssetAPI {
    fn needs_auth(&self) -> bool {
        false
    }
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        let path = req.uri().path().trim_start_matches('/');

        let (file, mime_type) = if path.is_empty() {
            (Asset::get("index.html"), mime_guess::mime::TEXT_HTML_UTF_8)
        } else {
            (
                Asset::get(path).or_else(|| Asset::get(ROOT)),
                from_path(path).first_or_octet_stream(),
            )
        };

        match file {
            Some(content) => {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime_type.as_ref())
                    .body(full(content.data.into_owned()))
                    .unwrap())
            }
            None => Ok(ResultResponse::error_with_code(StatusCode::NOT_FOUND)),
        }
    }
}
