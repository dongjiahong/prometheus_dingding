use std::{net::SocketAddr, str::FromStr, time::Duration};

use anyhow::Result;
use axum::{
    error_handling::HandleErrorLayer, extract::Extension, http::StatusCode, routing::post, Router,
};
use clap::Parser;
use ding::utils;
use tokio::signal;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::info;
use validator::Validate;

#[derive(Parser, Validate)]
#[clap(author = "jiahong", version = utils::version::get_version())]
struct Args {
    /// the web port, eg: 9080
    #[clap(short, long)]
    #[validate(length(min = 1))]
    port: String,
    #[clap(short, long)]
    #[validate(length(min = 1))]
    ding_url: String,
    #[clap(short, long)]
    #[validate(length(min = 1))]
    title: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::log::init_tracing();

    let args = Args::parse();

    let app = Router::new().route("/warn", post(ding::api::ding)).layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|error: BoxError| async move {
                if error.is::<tower::timeout::error::Elapsed>() {
                    Ok(StatusCode::REQUEST_TIMEOUT)
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    ))
                }
            }))
            .timeout(Duration::from_secs(30))
            .layer(TraceLayer::new_for_http())
            .layer(Extension(args.ding_url))
            .layer(Extension(args.title))
            .into_inner(),
    );

    let addr = SocketAddr::from_str(format!("0.0.0.0:{}", args.port).as_str())?;
    info!("listen on {:?}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c=> {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
