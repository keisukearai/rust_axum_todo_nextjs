//! API の統合テスト。
//!
//! `#[sqlx::test]` がテストごとに使い捨ての DB を作り、`../db/migrations` を適用し、
//! 終了後に落とす。テスト間で状態が混ざらないので順序に依存しない。
//!
//! 実行には CREATE DATABASE 権限のある `DATABASE_URL` が要る。

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use todo_backend::{build_app, state::AppState};
use tower::ServiceExt;

const ORIGIN: &str = "http://localhost:3000";

fn build(pool: PgPool) -> Router {
    build_app(AppState::new(pool), ORIGIN).expect("CORS オリジンが不正")
}

/// リクエストを1本投げて (ステータス, JSON ボディ) を返す。ボディが空なら Null。
async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);

    let request = match body {
        Some(value) => builder
            .header("content-type", "application/json")
            .body(Body::from(value.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();

    // /health や axum のリジェクションは JSON ではなく素のテキストを返す
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };

    (status, json)
}

async fn create(app: &Router, title: &str) -> Value {
    let (status, body) = call(app, "POST", "/api/todos", Some(json!({ "title": title }))).await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    body
}

fn id_of(todo: &Value) -> i64 {
    todo["id"].as_i64().expect("id が数値でない")
}

// --- ヘルスチェック ---------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn health_returns_ok(pool: PgPool) {
    let app = build(pool);
    let (status, body) = call(&app, "GET", "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}

// --- 一覧 -------------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn list_is_empty_initially(pool: PgPool) {
    let app = build(pool);
    let (status, body) = call(&app, "GET", "/api/todos", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_returns_newest_first(pool: PgPool) {
    let app = build(pool);
    let first = create(&app, "1番目").await;
    let second = create(&app, "2番目").await;

    let (status, body) = call(&app, "GET", "/api/todos", None).await;

    assert_eq!(status, StatusCode::OK);
    let ids: Vec<i64> = body
        .as_array()
        .unwrap()
        .iter()
        .map(id_of)
        .collect();
    assert_eq!(ids, vec![id_of(&second), id_of(&first)]);
}

// --- 作成 -------------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn create_returns_created_todo(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "牛乳を買う").await;

    assert_eq!(todo["title"], "牛乳を買う");
    assert_eq!(todo["completed"], false);
    assert!(todo["created_at"].is_string());
    assert!(todo["updated_at"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn create_trims_surrounding_whitespace(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "  前後に空白  ").await;

    assert_eq!(todo["title"], "前後に空白");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn create_rejects_blank_title(pool: PgPool) {
    let app = build(pool);

    for blank in ["", "   ", "\n\t"] {
        let (status, body) =
            call(&app, "POST", "/api/todos", Some(json!({ "title": blank }))).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "title: {blank:?}");
        assert!(body["error"].is_string(), "body: {body}");
    }
}

#[sqlx::test(migrations = "../db/migrations")]
async fn create_rejects_too_long_title(pool: PgPool) {
    let app = build(pool);
    let over = "あ".repeat(501);

    let (status, _) = call(&app, "POST", "/api/todos", Some(json!({ "title": over }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 500 文字ちょうどは通る（文字数で数える。バイト数ではない）
    let ok = "あ".repeat(500);
    let (status, _) = call(&app, "POST", "/api/todos", Some(json!({ "title": ok }))).await;
    assert_eq!(status, StatusCode::CREATED);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn create_rejects_missing_title_field(pool: PgPool) {
    let app = build(pool);

    let (status, _) = call(&app, "POST", "/api/todos", Some(json!({}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// --- 更新 -------------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn patch_toggles_completed_and_keeps_title(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "完了にする").await;
    let uri = format!("/api/todos/{}", id_of(&todo));

    let (status, updated) = call(&app, "PATCH", &uri, Some(json!({ "completed": true }))).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["completed"], true);
    assert_eq!(updated["title"], "完了にする");
    assert_eq!(updated["created_at"], todo["created_at"]);
    assert!(
        updated["updated_at"].as_str() > todo["updated_at"].as_str(),
        "updated_at が進んでいない"
    );
}

#[sqlx::test(migrations = "../db/migrations")]
async fn patch_renames_and_keeps_completed(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "改題前").await;
    let uri = format!("/api/todos/{}", id_of(&todo));
    call(&app, "PATCH", &uri, Some(json!({ "completed": true }))).await;

    let (status, updated) = call(&app, "PATCH", &uri, Some(json!({ "title": "  改題後  " }))).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["title"], "改題後");
    assert_eq!(updated["completed"], true);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn patch_rejects_empty_body(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "何も更新しない").await;
    let uri = format!("/api/todos/{}", id_of(&todo));

    let (status, body) = call(&app, "PATCH", &uri, Some(json!({}))).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].is_string(), "body: {body}");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn patch_rejects_blank_title(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "元のまま").await;
    let uri = format!("/api/todos/{}", id_of(&todo));

    let (status, _) = call(&app, "PATCH", &uri, Some(json!({ "title": "   " }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 失敗した更新で元の値が壊れていないこと
    let (_, list) = call(&app, "GET", "/api/todos", None).await;
    assert_eq!(list[0]["title"], "元のまま");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn patch_unknown_id_returns_not_found(pool: PgPool) {
    let app = build(pool);

    let (status, body) = call(
        &app,
        "PATCH",
        "/api/todos/999999",
        Some(json!({ "completed": true })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].is_string(), "body: {body}");
}

// --- 削除 -------------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn delete_removes_todo(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "消す").await;
    let uri = format!("/api/todos/{}", id_of(&todo));

    let (status, _) = call(&app, "DELETE", &uri, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = call(&app, "GET", "/api/todos", None).await;
    assert_eq!(list, json!([]));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delete_twice_returns_not_found(pool: PgPool) {
    let app = build(pool);
    let todo = create(&app, "二重削除").await;
    let uri = format!("/api/todos/{}", id_of(&todo));

    call(&app, "DELETE", &uri, None).await;
    let (status, _) = call(&app, "DELETE", &uri, None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// --- ルーティングと CORS ----------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn trailing_slash_is_not_absorbed(pool: PgPool) {
    let app = build(pool);

    let (status, _) = call(&app, "GET", "/api/todos/", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn preflight_allows_configured_origin(pool: PgPool) {
    let app = build(pool);

    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/todos")
        .header("origin", ORIGIN)
        .header("access-control-request-method", "PATCH")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let headers = response.headers();

    assert_eq!(headers["access-control-allow-origin"], ORIGIN);
    assert_eq!(
        headers["access-control-allow-methods"],
        "GET,POST,PATCH,DELETE"
    );
    assert_eq!(headers["access-control-allow-headers"], "content-type");
}

/// `allow_origin` に固定値を渡しているので、リクエストの Origin が何であれ
/// 返る allow-origin は常に設定値。許可外オリジンはこの不一致でブラウザが弾く。
/// ここでは「許可外のオリジンがエコーバックされないこと」を確かめる。
#[sqlx::test(migrations = "../db/migrations")]
async fn other_origins_are_not_echoed_back(pool: PgPool) {
    let app = build(pool);
    let other = "http://evil.example";

    let request = Request::builder()
        .method("GET")
        .uri("/api/todos")
        .header("origin", other)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let allow_origin = &response.headers()["access-control-allow-origin"];

    assert_ne!(allow_origin, other, "許可外のオリジンがエコーバックされている");
    assert_eq!(allow_origin, ORIGIN);
}
