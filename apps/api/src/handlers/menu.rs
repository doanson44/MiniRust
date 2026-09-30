use axum::extract::{rejection::JsonRejection, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use minirust_services::cqrs::{AsyncCommandHandler, AsyncQueryHandler};
use minirust_services::{Menu, MenuAccess, MenuCommand, MenuQuery, UpdateMenu};
use serde::{Deserialize, Serialize};

use crate::handlers::auth::{authorize_admin, current_authenticated_user};
use crate::response::{ApiResponse, Locale, ProblemDetails};
use crate::{json_rejection_response, parse_user_id, AppState};

#[derive(Deserialize)]
pub(crate) struct MenuRequest {
    parent_id: Option<String>,
    name: String,
    path: String,
    icon: Option<String>,
    allow_user: bool,
    allow_premium: bool,
    sort_order: i32,
    is_active: bool,
}

#[derive(Serialize)]
pub(crate) struct MenuResponse {
    id: String,
    parent_id: Option<String>,
    name: String,
    path: String,
    icon: Option<String>,
    allow_user: bool,
    allow_premium: bool,
    sort_order: i32,
    is_active: bool,
}

#[derive(Serialize)]
pub(crate) struct MenuListResponse {
    menus: Vec<MenuResponse>,
}

pub async fn list_for_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let actor = match current_authenticated_user(&state, &jar, locale).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };

    match state
        .menu_queries
        .handle(MenuQuery::ListForUser { actor })
        .await
    {
        Ok(menus) => (
            StatusCode::OK,
            Json(ApiResponse::new(MenuListResponse {
                menus: menus.into_iter().map(menu_response).collect(),
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
    }
}

pub async fn list_admin(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let actor = match authorize_admin(&state, &jar, locale).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };

    match state.menu_queries.handle(MenuQuery::List { actor }).await {
        Ok(menus) => (
            StatusCode::OK,
            Json(ApiResponse::new(MenuListResponse {
                menus: menus.into_iter().map(menu_response).collect(),
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
    }
}

pub async fn update(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(menu_id_value): Path<String>,
    body: Result<Json<MenuRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let actor = match authorize_admin(&state, &jar, locale).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let menu_id = match parse_user_id(&menu_id_value, locale) {
        Ok(menu_id) => menu_id,
        Err(error) => return error.into_response(),
    };
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    let input = match parse_request(body, locale).await {
        Ok(input) => input,
        Err(response) => return *response,
    };

    match state
        .menu_commands
        .handle(MenuCommand::Update {
            actor,
            menu_id,
            input,
        })
        .await
    {
        Ok(menu) => (StatusCode::OK, Json(ApiResponse::new(menu_response(menu)))).into_response(),
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
    }
}

async fn parse_request(
    body: MenuRequest,
    locale: Locale,
) -> Result<UpdateMenu, Box<axum::response::Response>> {
    let parent_id = match body.parent_id.as_deref() {
        Some(value) => match parse_user_id(value, locale) {
            Ok(id) => Some(id),
            Err(error) => return Err(Box::new(error.into_response())),
        },
        None => None,
    };

    Ok(UpdateMenu {
        parent_id,
        name: body.name,
        path: body.path,
        icon: body.icon,
        access: MenuAccess {
            user: body.allow_user,
            premium: body.allow_premium,
        },
        sort_order: body.sort_order,
        is_active: body.is_active,
    })
}

fn menu_response(menu: Menu) -> MenuResponse {
    MenuResponse {
        id: menu.id.as_uuid().to_string(),
        parent_id: menu.parent_id.map(|id| id.as_uuid().to_string()),
        name: menu.name,
        path: menu.path,
        icon: menu.icon,
        allow_user: menu.access.user,
        allow_premium: menu.access.premium,
        sort_order: menu.sort_order,
        is_active: menu.is_active,
    }
}
