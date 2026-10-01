use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

pub async fn openapi() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "openapi": "3.0.3",
            "info": { "title": "MiniRust API", "version": "0.1.0" },
            "paths": {
                "/health": { "get": { "summary": "Health and MariaDB connectivity", "responses": { "200": { "description": "Application and database are healthy" }, "503": { "description": "Database is unavailable" } } } },
                "/api/v1/hello": { "get": { "summary": "Hello query", "responses": { "200": { "description": "Greeting" } } } },
                "/api/v1/echo": { "post": { "summary": "Echo command", "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object", "required": ["message"], "properties": { "message": { "type": "string" } } } } } }, "responses": { "200": { "description": "Echo response" }, "400": { "description": "Malformed JSON or invalid content type" }, "422": { "description": "Validation error" }, "500": { "description": "Unexpected server failure" } } } },
                "/api/v1/auth/register/request-verification": { "post": { "summary": "Request registration verification link", "responses": { "200": { "description": "Request accepted" } } } },
                "/api/v1/auth/register/verify": { "post": { "summary": "Verify registration link and create session", "responses": { "200": { "description": "Authenticated session" } } } },
                "/api/v1/auth/login/request-code": { "post": { "summary": "Request login verification code", "responses": { "200": { "description": "Request accepted" } } } },
                "/api/v1/auth/login/verify-code": { "post": { "summary": "Verify login code and create session", "responses": { "200": { "description": "Authenticated session" } } } },
                "/api/v1/auth/logout": { "post": { "summary": "Revoke current session", "responses": { "200": { "description": "Session revoked" } } } },
                "/api/v1/auth/me": { "get": { "summary": "Get current authenticated user", "responses": { "200": { "description": "Current user" }, "401": { "description": "Invalid or expired session" } } } },
                "/api/v1/menus": { "get": { "summary": "List active menus the current account may open", "responses": { "200": { "description": "Access-filtered menus" }, "401": { "description": "Authentication required" } } } },
                "/api/v1/admin/menus": { "get": { "summary": "List all system menus (admin only)", "responses": { "200": { "description": "Menus" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" } } } },
                "/api/v1/admin/menus/{menu_id}": { "patch": { "summary": "Update menu access flags by id (admin only)", "responses": { "200": { "description": "Menu updated" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" }, "404": { "description": "Menu not found" }, "422": { "description": "Invalid menu data" } } } },
                "/api/v1/users/me": { "patch": { "summary": "Update current user profile", "responses": { "200": { "description": "Profile updated" }, "401": { "description": "Authentication required" }, "422": { "description": "Invalid profile data" } } }, "delete": { "summary": "Delete current user account", "responses": { "200": { "description": "Account deleted" }, "401": { "description": "Authentication required" } } } },
                "/api/v1/users/me/lock": { "post": { "summary": "Lock current user account", "responses": { "200": { "description": "Account locked" }, "401": { "description": "Authentication required" } } } },
                "/api/v1/admin/users": { "get": { "summary": "List users (admin only)", "responses": { "200": { "description": "Users" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" } } }, "post": { "summary": "Create user by email (admin only)", "responses": { "201": { "description": "User created" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" } } } },
                "/api/v1/admin/users/{user_id}": { "get": { "summary": "Get user by id (admin only)", "responses": { "200": { "description": "User" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } }, "patch": { "summary": "Update user email by user id (admin only)", "responses": { "200": { "description": "User updated" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" }, "409": { "description": "Email already exists or protected user" } } }, "delete": { "summary": "Delete user by user id (admin only)", "responses": { "204": { "description": "User deleted" }, "401": { "description": "Authentication required" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } } },
                "/api/v1/admin/users/{user_id}/unlock": { "post": { "summary": "Unlock user account by user id (admin only)", "responses": { "200": { "description": "User unlocked" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } } },
                "/api/v1/admin/users/{user_id}/role": { "put": { "summary": "Assign or remove admin role by user id (admin only)", "responses": { "200": { "description": "User role updated" }, "403": { "description": "Admin role required" }, "422": { "description": "Invalid role" } } } },
                "/api/v1/admin/users/{user_id}/entitlements/premium": { "get": { "summary": "Get premium entitlement by user id (admin only)", "responses": { "200": { "description": "Premium entitlement" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } }, "put": { "summary": "Assign premium entitlement by user id (admin only)", "responses": { "200": { "description": "Premium entitlement updated" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" }, "422": { "description": "Invalid premium expiry" } } }, "delete": { "summary": "Revoke premium entitlement by user id (admin only)", "responses": { "200": { "description": "Premium entitlement revoked" }, "403": { "description": "Admin role required" }, "404": { "description": "User not found" } } } }
            }
        })),
    )
}

pub async fn swagger_ui() -> impl IntoResponse {
    let html = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MiniRust API — OpenAPI</title>
<style>
body{margin:0;font:15px/1.5 system-ui,sans-serif;background:#0f172a;color:#e2e8f0}
main{max-width:1100px;margin:auto;padding:32px 20px}h1{margin:0 0 8px;color:#fff}p{color:#94a3b8}
.card{margin:14px 0;padding:16px;border:1px solid #334155;border-radius:12px;background:#111827}
.method{display:inline-block;padding:3px 8px;border-radius:6px;background:#22d3ee;color:#082f49;font-weight:700;margin-right:10px}
.path{font-family:ui-monospace,monospace;color:#fff}.summary{margin:8px 0;color:#94a3b8}
pre{white-space:pre-wrap;background:#020617;padding:16px;border-radius:10px;overflow:auto}
a{color:#67e8f9}
</style>
</head>
<body><main><h1>MiniRust API</h1><p>OpenAPI 3.0.3 documentation. The viewer is bundled with the application; no external CDN is required.</p><div id="docs">Loading…</div></main>
<script>
fetch('/api/v1/openapi.json').then(r=>r.json()).then(spec=>{
 const root=document.querySelector('#docs'); const paths=spec.paths||{};
 root.innerHTML=Object.entries(paths).flatMap(([path,item])=>Object.entries(item).map(([method,op])=>'<section class="card"><div><span class="method">'+method.toUpperCase()+'</span><span class="path">'+path+'</span></div><div class="summary">'+(op.summary||'')+'</div></section>')).join('')+'<section class="card"><details><summary>Raw OpenAPI document</summary><pre>'+JSON.stringify(spec,null,2).replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]))+'</pre></details></section>';
}).catch(()=>{document.querySelector('#docs').textContent='Unable to load OpenAPI document.'});
</script></body></html>"#;
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
}
