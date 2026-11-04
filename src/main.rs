#![warn(clippy::pedantic)]
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio_rustls::rustls;

use hyper::{server::conn::http1, service::service_fn};
use tokio_util::task::TaskTracker;
use tracing::{error, info, warn};

use crate::config::Config;

use self::forward_proxy::send;

mod config;
mod forward_proxy;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let _ = rustls::crypto::CryptoProvider::install_default(
        rustls::crypto::aws_lc_rs::default_provider(),
    );

    let config = config::Config::new()?;
    info!("Forward proxy running with {}", &config);

    serve(config).await
}

async fn serve(config: Config) -> Result<(), anyhow::Error> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("Unable to start proxy when binding to {addr}"))?;
    info!("Forward proxy bound to {}", addr);

    let tracker = TaskTracker::new();

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_only()
        .enable_http1()
        .build();

    let exec = TokioExecutor::new();
    let client: Client<_, Incoming> =
        hyper_util::client::legacy::Builder::new(exec.clone()).build(https_connector);

    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                info!("SIGTERM received. Stopping new connections");
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C received. Stopping new connections");
                break;
            }

            res = listener.accept() => {
                let (stream, _) = match res {
                    Ok(conn) => conn,
                    Err(err) => {
                        error!("Accept failed: {err}");
                        continue;
                    }
                };

                let io = TokioIo::new(stream);
                let (client, config) = (client.clone(), config.clone());

                tracker.spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(move |req| send(config.clone(), client.clone(), req)))
                        .await
                    {
                        warn!("Connection closed: {err}");
                    }
                });
            }
        }
    }

    drop(listener);

    tracker.close();
    let connections = tracker.len();

    tokio::select! {
        () = tracker.wait() => {
            info!("Connections drained ({connections})");
        }
        () = tokio::time::sleep(Duration::from_secs(5)) => {
            warn!("Connection drain timeout. Forcing exit of {connections} connections");
        }
    }

    Ok(())
}
