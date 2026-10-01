#[cfg(feature = "hydrate")]
use crate::types::{ApiProblem, ApiResponse, ApiResponseWithMeta};
#[cfg(feature = "hydrate")]
use serde::Deserialize;

#[cfg(feature = "hydrate")]
pub async fn api_request(
    method: gloo_net::http::Method,
    path: &str,
    body: Option<String>,
) -> Result<gloo_net::http::Response, String> {
    let request = match method {
        gloo_net::http::Method::GET => gloo_net::http::Request::get(path),
        gloo_net::http::Method::POST => gloo_net::http::Request::post(path),
        gloo_net::http::Method::PATCH => gloo_net::http::Request::patch(path),
        gloo_net::http::Method::PUT => gloo_net::http::Request::put(path),
        gloo_net::http::Method::DELETE => gloo_net::http::Request::delete(path),
        _ => return Err("Unsupported HTTP method.".to_owned()),
    };
    let request = if let Some(body) = body {
        request
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|error| error.to_string())?
    } else {
        request.build().map_err(|error| error.to_string())?
    };
    request.send().await.map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn api_error(response: gloo_net::http::Response) -> String {
    response
        .json::<ApiProblem>()
        .await
        .map(|problem| problem.detail)
        .unwrap_or_else(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn api_json<T: for<'de> Deserialize<'de>>(
    method: gloo_net::http::Method,
    path: &str,
    body: Option<String>,
) -> Result<T, String> {
    let response = api_request(method, path, body).await?;
    if !response.ok() {
        return Err(api_error(response).await);
    }
    response
        .json::<ApiResponse<T>>()
        .await
        .map(|envelope| envelope.data)
        .map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn api_json_with_meta<T: for<'de> Deserialize<'de>, M: for<'de> Deserialize<'de>>(
    method: gloo_net::http::Method,
    path: &str,
    body: Option<String>,
) -> Result<ApiResponseWithMeta<T, M>, String> {
    let response = api_request(method, path, body).await?;
    if !response.ok() {
        return Err(api_error(response).await);
    }
    response
        .json::<ApiResponseWithMeta<T, M>>()
        .await
        .map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn api_empty(
    method: gloo_net::http::Method,
    path: &str,
    body: Option<String>,
) -> Result<(), String> {
    let response = api_request(method, path, body).await?;
    if response.ok() || response.status() == 204 {
        Ok(())
    } else {
        Err(api_error(response).await)
    }
}
