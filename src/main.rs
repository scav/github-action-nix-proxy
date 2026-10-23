#![warn(clippy::pedantic)]
use std::net::SocketAddr;

use anyhow::{Context, Result};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio_rustls::rustls;
use tracing::{debug, error, info, warn};

use hyper::{server::conn::http1, service::service_fn};

use crate::config::Config;

use self::forward_proxy::send;

mod config;
mod forward_proxy;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    // Set crypto provider globally at startup
    let _ = rustls::crypto::CryptoProvider::install_default(
        rustls::crypto::aws_lc_rs::default_provider(),
    );

    let config = config::Config::new()?;
    debug!("Foward proxy running with {:?}", &config);

    serve(&config).await
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn serve(config: &Config) -> Result<(), anyhow::Error> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port)); // to support OCI we cant bind to 127
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("Unable to start proxy when binding to {addr}"))?;
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let mut signal = std::pin::pin!(shutdown_signal());

    info!("Forward proxy bound to {}", addr);

    let exec = TokioExecutor::new();

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_only()
        .enable_http1()
        .build();

    let client: Client<_, Incoming> =
        hyper_util::client::legacy::Builder::new(exec.clone()).build(https_connector);

    loop {
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let client = client.clone();
                let cache_url = config.cache_url.clone();
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(move |req| send(client.clone(),cache_url.clone(), req)))
                        .await
                    {
                        error!("Error serving connection: {}", err);
                    }
                });
            },

            () = &mut signal => {
                drop(listener);
                break;
            }
        }
    }
    tokio::select! {
        () = graceful.shutdown() => {
            info!("Shutdown complete");
        },
        () = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
            warn!("Shutdown failed");
        }
    }

    Ok(())
}
