use axum::extract::{rejection::JsonRejection, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use minirust_services::cqrs::{AsyncCommandHandler, AsyncQueryHandler};
use minirust_services::{
    CreateMenu, Menu, MenuCommand, MenuCommandResult, MenuQuery, MenuQueryResult, MenuRole,
    UpdateMenu,
};
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
    required_role: String,
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
    required_role: &'static str,
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
        Ok(MenuQueryResult::Menus(menus)) => (
            StatusCode::OK,
            Json(ApiResponse::new(MenuListResponse {
                menus: menus.into_iter().map(menu_response).collect(),
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
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
        Ok(MenuQueryResult::Menus(menus)) => (
            StatusCode::OK,
            Json(ApiResponse::new(MenuListResponse {
                menus: menus.into_iter().map(menu_response).collect(),
            })),
        )
            .into_response(),
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

pub async fn get(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(menu_id_value): Path<String>,
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

    match state
        .menu_queries
        .handle(MenuQuery::Get { actor, menu_id })
        .await
    {
        Ok(MenuQueryResult::Menu(menu)) => {
            (StatusCode::OK, Json(ApiResponse::new(menu_response(menu)))).into_response()
        }
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

pub async fn create(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    body: Result<Json<MenuRequest>, JsonRejection>,
) -> impl IntoResponse {
    let locale = Locale::from_accept_language(&headers);
    let actor = match authorize_admin(&state, &jar, locale).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => return json_rejection_response(rejection, locale).into_response(),
    };

    let input = match parse_request(body.0, locale).await {
        Ok(input) => input,
        Err(response) => return response,
    };

    match state
        .menu_commands
        .handle(MenuCommand::Create {
            actor,
            input: CreateMenu {
                parent_id: input.parent_id,
                name: input.name,
                path: input.path,
                icon: input.icon,
                required_role: input.required_role,
                sort_order: input.sort_order,
                is_active: input.is_active,
            },
        })
        .await
    {
        Ok(MenuCommandResult::Menu(menu)) => (
            StatusCode::CREATED,
            Json(ApiResponse::new(menu_response(menu))),
        )
            .into_response(),
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
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

    let input = match parse_request(body.0, locale).await {
        Ok(input) => input,
        Err(response) => return response,
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
        Ok(MenuCommandResult::Menu(menu)) => {
            (StatusCode::OK, Json(ApiResponse::new(menu_response(menu)))).into_response()
        }
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

pub async fn delete_menu(
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    Path(menu_id_value): Path<String>,
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

    match state
        .menu_commands
        .handle(MenuCommand::Delete { actor, menu_id })
        .await
    {
        Ok(MenuCommandResult::Deleted) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => ProblemDetails::menu(&error, locale).into_response(),
        Ok(_) => ProblemDetails::internal(locale).into_response(),
    }
}

async fn parse_request(
    body: MenuRequest,
    locale: Locale,
) -> Result<UpdateMenu, axum::response::Response> {
    let parent_id = match body.parent_id.as_deref() {
        Some(value) => match parse_user_id(value, locale) {
            Ok(id) => Some(id),
            Err(error) => return Err(error.into_response()),
        },
        None => None,
    };
    let required_role = match MenuRole::parse(&body.required_role) {
        Ok(role) => role,
        Err(error) => return Err(ProblemDetails::menu(&error, locale).into_response()),
    };

    Ok(UpdateMenu {
        parent_id,
        name: body.name,
        path: body.path,
        icon: body.icon,
        required_role,
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
        required_role: menu.required_role.as_str(),
        sort_order: menu.sort_order,
        is_active: menu.is_active,
    }
}

