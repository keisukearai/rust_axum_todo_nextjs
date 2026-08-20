use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::Router;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

use super::model::{CreateTodo, Todo, UpdateTodo, normalize_title};
use super::repo;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/todos", get(list).post(create))
        .route("/todos/{id}", patch(update).delete(delete))
}

async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<Todo>>> {
    Ok(Json(repo::list(&state.pool).await?))
}

async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> AppResult<(StatusCode, Json<Todo>)> {
    let title = normalize_title(&payload.title).map_err(AppError::Validation)?;
    let todo = repo::create(&state.pool, &title).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodo>,
) -> AppResult<Json<Todo>> {
    if payload.title.is_none() && payload.completed.is_none() {
        return Err(AppError::Validation(
            "title か completed のどちらかを指定してください".to_string(),
        ));
    }

    let title = payload
        .title
        .as_deref()
        .map(normalize_title)
        .transpose()
        .map_err(AppError::Validation)?;

    repo::update(&state.pool, id, title.as_deref(), payload.completed)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<StatusCode> {
    if repo::delete(&state.pool, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
