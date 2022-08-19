use std::{net::SocketAddr, str::FromStr, time::Duration};

use anyhow::Result;
use axum::{
    body::{Body, Bytes},
    error_handling::HandleErrorLayer,
    extract::Extension,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Router,
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

async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::log::init_tracing();

    let args = Args::parse();

    let app = Router::new()
        .route("/ding/text", post(ding::api::ding_text))
        .route("/ding/markdown", post(ding::api::ding_markdown))
        .layer(
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
                .layer(middleware::from_fn(print_request_response))
                .layer(Extension((args.title, args.ding_url)))
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
