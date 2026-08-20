# todo-next

TODO アプリ。フロントエンド・バックエンド・データベースを別々に動かす3層構成。

```
todo-next/
├── backend/    Rust + Axum + sqlx  → http://127.0.0.1:8080  (JSON API のみ)
├── frontend/   Next.js (App Router, TypeScript) → http://localhost:3000
└── db/         PostgreSQL のマイグレーションと手順（独立プロセス）
```

データの流れは `Next.js → Rust API → PostgreSQL` の一方向。Next.js から DB へは直接つながず、
DB に触るのは backend だけ。

## 必要なもの

- Rust（stable）
- Node.js 20 以上
- PostgreSQL 16 以上（Homebrew 等で独立して起動）

## セットアップ

```sh
# 1. DB
brew services start postgresql@16
createdb todo_next

# 2. backend
cd backend
cp .env.example .env
$EDITOR .env          # DATABASE_URL を自分のユーザー名に合わせる
cargo run             # migrations/ は起動時に自動適用される

# 3. frontend（別ターミナル）
cd frontend
cp .env.example .env.local
npm install
npm run dev
```

http://localhost:3000 を開く。

## API

すべて `http://127.0.0.1:8080` 配下。

| メソッド | パス | 内容 | 成功時 |
|---|---|---|---|
| `GET` | `/health` | ヘルスチェック | `200 ok` |
| `GET` | `/api/todos` | 一覧（作成日時の新しい順） | `200` |
| `POST` | `/api/todos` | 作成 `{ "title": "..." }` | `201` |
| `PATCH` | `/api/todos/:id` | 部分更新 `{ "title"?, "completed"? }` | `200` |
| `DELETE` | `/api/todos/:id` | 削除 | `204` |

エラーは `{ "error": "..." }` を返す。`400`（title が空・500文字超・更新項目なし）、
`404`（該当 TODO なし）、`500`（DB エラー）。

## 設定

### backend（`backend/.env`）

| 変数 | 既定値 | 内容 |
|---|---|---|
| `DATABASE_URL` | **必須** | PostgreSQL の接続先 |
| `HOST` | `127.0.0.1` | 待ち受けアドレス |
| `PORT` | `8080` | 待ち受けポート |
| `DATABASE_MAX_CONNECTIONS` | `5` | コネクションプール上限 |
| `CORS_ORIGIN` | `http://localhost:3000` | 許可するオリジン（1つ） |
| `RUST_LOG` | `todo_backend=debug,tower_http=info,sqlx=warn` | ログレベル |

### frontend（`frontend/.env.local`）

| 変数 | 既定値 | 内容 |
|---|---|---|
| `NEXT_PUBLIC_API_BASE_URL` | `http://localhost:8080` | Rust API の場所。ブラウザからも参照する |

## 実装メモ

- 一覧の初期表示は Server Component（`app/page.tsx`）が API から取得する。追加・更新・削除は
  Client Component（`app/todo-app.tsx`）がブラウザから直接 API を叩くため、backend 側に CORS を入れてある。
- sqlx はコンパイル時検証マクロ（`query!`）を使わず実行時クエリにしている。ビルドに DB 接続が要らない。
- `PORT` や `CORS_ORIGIN` を変えるときは frontend の `NEXT_PUBLIC_API_BASE_URL` も合わせる。

## テスト

```sh
cd backend && cargo test
cd frontend && npm run lint && npx tsc --noEmit
```
