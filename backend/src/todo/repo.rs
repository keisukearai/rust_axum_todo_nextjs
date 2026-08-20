use sqlx::PgPool;

use super::model::Todo;

const COLUMNS: &str = "id, title, completed, created_at, updated_at";

pub async fn list(pool: &PgPool) -> Result<Vec<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>(&format!(
        "SELECT {COLUMNS} FROM todos ORDER BY created_at DESC, id DESC"
    ))
    .fetch_all(pool)
    .await
}

pub async fn create(pool: &PgPool, title: &str) -> Result<Todo, sqlx::Error> {
    sqlx::query_as::<_, Todo>(&format!(
        "INSERT INTO todos (title) VALUES ($1) RETURNING {COLUMNS}"
    ))
    .bind(title)
    .fetch_one(pool)
    .await
}

/// 部分更新。None の項目は現在値を残す。
pub async fn update(
    pool: &PgPool,
    id: i64,
    title: Option<&str>,
    completed: Option<bool>,
) -> Result<Option<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>(&format!(
        "UPDATE todos
            SET title = COALESCE($2, title),
                completed = COALESCE($3, completed),
                updated_at = now()
          WHERE id = $1
      RETURNING {COLUMNS}"
    ))
    .bind(id)
    .bind(title)
    .bind(completed)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
