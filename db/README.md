# db

TODO アプリのデータベース。アプリとは独立したプロセスとして動かします。

## セットアップ

```sh
# PostgreSQL を起動（Homebrew）
brew services start postgresql@16

# データベースを作る
createdb todo_next
```

## マイグレーション

`migrations/` の SQL は backend の起動時に自動適用されます（`sqlx::migrate!`）。
適用済みかどうかは `_sqlx_migrations` テーブルで判定されるため、二重適用はされません。

手動で当てる場合:

```sh
psql -d todo_next -f migrations/20260821000000_create_todos.sql
```

## 接続先

backend の `.env` の `DATABASE_URL` が唯一の接続点です。

```
DATABASE_URL=postgres://<user>@localhost:5432/todo_next
```

## スキーマ

| テーブル | 列 | 型 | 内容 |
|---|---|---|---|
| `todos` | `id` | `BIGSERIAL` | 主キー |
| | `title` | `TEXT NOT NULL` | 本文 |
| | `completed` | `BOOLEAN NOT NULL` | 完了フラグ |
| | `created_at` | `TIMESTAMPTZ NOT NULL` | 作成日時 |
| | `updated_at` | `TIMESTAMPTZ NOT NULL` | 更新日時 |
