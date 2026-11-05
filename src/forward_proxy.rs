use anyhow::Result;
use thiserror::Error;

use hyper::{Request, Response, body::Incoming};
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use tracing::debug;

use crate::config::Config;

type HttpClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, Incoming>;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Could not create request url: {message:?})")]
    Url { message: String },

    #[error("Could not forward request: {message:?})")]
    Forward { message: String },

    #[error("Could not create header: {message:?})")]
    HeaderError { message: String },
}

pub async fn send(
    config: Config,
    client: HttpClient,
    req: Request<Incoming>,
) -> Result<Response<Incoming>, ProxyError> {
    let req = send_inner::<Incoming>(&config.cache_url, &config.auth_token, &config.host, req)?;

    debug!("Request {:?}", &req);

    client.request(req).await.map_err(|e| ProxyError::Forward {
        message: e.to_string(),
    })
}

fn send_inner<B>(
    cache_url: &str,
    token: &str,
    host: &str,
    mut req: Request<B>,
) -> Result<Request<B>, ProxyError> {
    let url = req.uri_mut();

    if let Some(f) = url.path_and_query() {
        *url = format!(
            "{cache_url}{}{}",
            f.path(),
            f.query()
                .filter(|q| !q.is_empty())
                .map(|q| format!("?{q}"))
                .unwrap_or_default()
        )
        .parse()
        .map_err(|_| ProxyError::Url {
            message: "Could not create url".to_string(),
        })?;
    }

    req.headers_mut().insert(
        hyper::header::HOST,
        hyper::header::HeaderValue::from_str(host).map_err(|e| ProxyError::HeaderError {
            message: format!("Error in HOST header: {}", e),
        })?,
    );
    req.headers_mut().insert(
        hyper::header::FORWARDED,
        hyper::header::HeaderValue::from_static("by=github-actions-nix-proxy;for=nix"),
    );
    req.headers_mut().insert(
        hyper::header::AUTHORIZATION,
        hyper::header::HeaderValue::from_str(token).map_err(|e| ProxyError::HeaderError {
            message: format!("Error in AUTHORIZATION header: {}", e),
        })?,
    );

    Ok(req)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::config::Config;
    use crate::forward_proxy::send_inner;
    use http::Request;

    #[test]
    fn test_send_inner() {
        let cache_url = Arc::from("https://example.org");
        let port = 8300;
        let auth_token = String::from("na");
        let host = Arc::from("example.org");

        let config = Config::with_token("Bearer", port, cache_url, host, auth_token)
            .expect("config should be created");

        {
            let req = Request::builder()
                .method("GET")
                .uri("http://127.0.0.1:8300?abc=123&123=abc")
                .body(())
                .unwrap();
            let modified_request =
                send_inner::<()>(&config.cache_url, &config.auth_token, &config.host, req);
            assert!(
                modified_request
                    .is_ok_and(|req| { req.uri().eq("https://example.org/?abc=123&123=abc") })
            );
        }
        {
            let req = Request::builder()
                .method("GET")
                .uri("http://127.0.0.1:8300/foo/bar?abc=123&123=abc")
                .body(())
                .unwrap();
            let modified_request =
                send_inner::<()>(&config.cache_url, &config.auth_token, &config.host, req);
            assert!(
                modified_request.is_ok_and(|req| {
                    req.uri().eq("https://example.org/foo/bar?abc=123&123=abc")
                })
            );
        }
        {
            let req = Request::builder()
                .method("GET")
                .uri("http://127.0.0.1:8300")
                .body(())
                .unwrap();
            let modified_request =
                send_inner::<()>(&config.cache_url, &config.auth_token, &config.host, req);
            assert!(modified_request.is_ok_and(|req| { req.uri().eq("https://example.org") }));
        }
    }
}
