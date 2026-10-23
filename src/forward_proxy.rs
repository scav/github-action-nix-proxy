use anyhow::Result;
use std::sync::Arc;
use thiserror::Error;

use hyper::{Request, Response, body::Incoming};
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use tracing::debug;

type HttpClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, Incoming>;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Could not create proxy request url: {message:?})")]
    UrlError { message: String },

    #[error("Could not forward proxy request: {message:?})")]
    ForwardError { message: String },
}

pub async fn send(
    client: HttpClient,
    cache_url: Arc<str>,
    req: Request<Incoming>,
) -> Result<Response<Incoming>, ProxyError> {
    let req = send_inner::<Incoming>(&cache_url, req)?;
    debug!("Created request {:?}", &req);

    client
        .request(req)
        .await
        .map_err(|e| ProxyError::ForwardError {
            message: e.to_string(),
        })
}

fn send_inner<B>(cache_url: &str, mut req: Request<B>) -> Result<Request<B>, ProxyError> {
    let url = req.uri_mut();

    if let Some(f) = url.path_and_query() {
        *url = format!(
            "{cache_url}{}?{}x-use-new-ingress=1",
            f.path(),
            f.query()
                .filter(|q| !q.is_empty())
                .map(|q| format!("{q}&"))
                .unwrap_or_default()
        )
        .parse()
        .map_err(|_| ProxyError::UrlError {
            message: "Could not create url".to_string(),
        })?;
    }

    req.headers_mut().extend([
        (
            hyper::header::HOST,
            hyper::header::HeaderValue::from_static(
                "ip-nix-cache-test.s3.eu-north-1.amazonaws.com",
            ),
        ),
        (
            hyper::header::FORWARDED,
            hyper::header::HeaderValue::from_static("by=github-actions-nix-proxy;for=nix"),
        ),
    ]);

    Ok(req)
}

#[cfg(test)]
mod test {
    use crate::forward_proxy::send_inner;
    use http::Request;

    #[tokio::test]
    async fn test_send_inner() {
        let cache_url = "https://example.org";
        let req = Request::builder()
            .method("GET")
            .uri("http://127.0.0.1:8300?abc=123&123=abc")
            .body(())
            .unwrap();
        let modified_request = send_inner::<()>(cache_url, req);
        assert!(modified_request.is_ok_and(|req| {
            req.uri()
                .eq("https://example.org/?abc=123&123=abc&x-use-new-ingress=1")
        }));
    }
}
