import { fetchTodos, type Todo } from "@/lib/api";
import TodoApp from "./todo-app";

// 一覧は常に API から取り直す（ビルド時に固定しない）
export const dynamic = "force-dynamic";

export default async function Home() {
  let initialTodos: Todo[] = [];
  let loadError: string | null = null;

  try {
    initialTodos = await fetchTodos();
  } catch (err) {
    loadError =
      err instanceof Error
        ? `API に接続できませんでした: ${err.message}`
        : "API に接続できませんでした";
  }

  return (
    <main className="page">
      <header className="header">
        <h1>TODO</h1>
        <p className="subtitle">Rust (Axum) + Next.js + PostgreSQL</p>
      </header>
      <TodoApp initialTodos={initialTodos} loadError={loadError} />
    </main>
  );
}
