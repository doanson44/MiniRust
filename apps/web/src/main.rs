use axum::{routing::get, Router};
use std::net::SocketAddr;

const DEFAULT_ADDR: &str = "127.0.0.1:3000";

fn app() -> Router {
    Router::new().route("/", get(root))
}

async fn root() -> &'static str {
    "MiniRust"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = DEFAULT_ADDR.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, app()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_root_returns_ok_with_body() {
        let response = app()
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "MiniRust");
    }
}
