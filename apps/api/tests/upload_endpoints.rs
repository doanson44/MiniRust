mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use minirust_services::MAX_UPLOAD_BYTES;
use support::{admin_cookie, temp_upload_directory, test_app, test_app_with};
use tower::ServiceExt;

const BOUNDARY: &str = "minirust-boundary";

fn multipart_body(file_name: Option<&str>, content: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());

    match file_name {
        Some(file_name) => body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
                 Content-Type: application/octet-stream\r\n\r\n"
            )
            .as_bytes(),
        ),
        None => body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"note\"\r\n\r\nno file here",
        ),
    }

    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    body
}

fn upload_request(file_name: Option<&str>, content: &[u8]) -> Request<Body> {
    Request::post("/api/v1/uploads")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .body(Body::from(multipart_body(file_name, content)))
        .unwrap()
}

#[tokio::test]
async fn upload_requires_authentication() {
    let app = test_app().await;

    let response = app
        .router()
        .oneshot(upload_request(Some("note.txt"), b"hello"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn authenticated_user_can_upload_a_file() {
    let directory = temp_upload_directory();
    let app = test_app_with(&directory).await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/uploads")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={BOUNDARY}"),
                )
                .header("cookie", &cookie)
                .body(Body::from(multipart_body(Some("Report.PDF"), b"%PDF-1.4")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let name = body["data"]["name"].as_str().unwrap();

    assert!(name.ends_with(".pdf"));
    assert_eq!(body["data"]["original_name"], "Report.PDF");
    assert_eq!(body["data"]["size"], 8);
    assert_eq!(
        std::fs::read(directory.join(name)).unwrap(),
        b"%PDF-1.4".to_vec()
    );
}

#[tokio::test]
async fn upload_rejects_a_request_without_a_file_field() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/uploads")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={BOUNDARY}"),
                )
                .header("cookie", &cookie)
                .body(Body::from(multipart_body(None, b"")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upload_rejects_a_file_over_the_limit() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;
    let content = vec![0u8; MAX_UPLOAD_BYTES + 1];

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/uploads")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={BOUNDARY}"),
                )
                .header("cookie", &cookie)
                .body(Body::from(multipart_body(Some("big.bin"), &content)))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["code"], "UPLOAD_FILE_TOO_LARGE");
}

fn avatar_request(file_name: &str, content: &[u8]) -> Request<Body> {
    Request::post("/api/v1/users/me/avatar")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .body(Body::from(multipart_body(Some(file_name), content)))
        .unwrap()
}

fn authenticated_avatar_request(cookie: &str, file_name: &str, content: &[u8]) -> Request<Body> {
    Request::post("/api/v1/users/me/avatar")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .header("cookie", cookie)
        .body(Body::from(multipart_body(Some(file_name), content)))
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn avatar_upload_requires_authentication() {
    let app = test_app().await;

    let response = app
        .router()
        .oneshot(avatar_request("portrait.png", b"image"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn authenticated_user_can_upload_and_read_an_avatar() {
    let directory = temp_upload_directory();
    let app = test_app_with(&directory).await;
    let (cookie, admin_id) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(authenticated_avatar_request(
            &cookie,
            "Portrait.PNG",
            b"image-bytes",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let avatar_url = body["data"]["avatar_url"].as_str().unwrap();
    assert!(avatar_url.starts_with("/api/v1/users/me/avatar/png?v="));

    assert_eq!(
        std::fs::read(directory.join(&admin_id).join("avatar.png")).unwrap(),
        b"image-bytes".to_vec()
    );

    let path = avatar_url.split('?').next().unwrap();
    let response = app
        .router()
        .oneshot(
            Request::get(path)
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("content-type").unwrap(), "image/png");
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), b"image-bytes");
}

#[tokio::test]
async fn avatar_upload_rejects_a_non_image_file() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(authenticated_avatar_request(&cookie, "notes.txt", b"text"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = json_body(response).await;
    assert_eq!(body["code"], "UNSUPPORTED_AVATAR_FORMAT");
}

#[tokio::test]
async fn avatar_read_reports_a_missing_file() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(
            Request::get("/api/v1/users/me/avatar/png")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn avatar_read_rejects_an_unsupported_extension() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(
            Request::get("/api/v1/users/me/avatar/svg")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn patch_profile(
    app: &support::TestApp,
    cookie: &str,
    avatar_url: &str,
) -> axum::response::Response {
    app.router()
        .oneshot(
            Request::patch("/api/v1/users/me")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(
                    serde_json::json!({
                        "full_name": "MiniRust Admin",
                        "avatar_url": avatar_url
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn avatar_url_returned_by_the_upload_is_accepted_by_the_profile() {
    let directory = temp_upload_directory();
    let app = test_app_with(&directory).await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = app
        .router()
        .oneshot(authenticated_avatar_request(
            &cookie,
            "portrait.png",
            b"image-bytes",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let avatar_url = json_body(response).await["data"]["avatar_url"]
        .as_str()
        .unwrap()
        .to_owned();

    let response = patch_profile(&app, &cookie, &avatar_url).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["data"]["avatar_url"], avatar_url);
}

#[tokio::test]
async fn profile_update_rejects_a_protocol_relative_avatar_url() {
    let app = test_app().await;
    let (cookie, _admin_id) = admin_cookie(&app).await;

    let response = patch_profile(&app, &cookie, "//evil.example/avatar.png").await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["code"], "INVALID_AVATAR_URL");
}
