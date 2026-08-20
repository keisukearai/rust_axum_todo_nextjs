"use client";

import { useMemo, useState } from "react";
import {
  createTodo,
  deleteTodo,
  updateTodo,
  type Todo,
} from "@/lib/api";
import TodoItem from "./todo-item";

type Filter = "all" | "active" | "done";

const FILTERS: { key: Filter; label: string }[] = [
  { key: "all", label: "すべて" },
  { key: "active", label: "未完了" },
  { key: "done", label: "完了" },
];

export default function TodoApp({
  initialTodos,
  loadError,
}: {
  initialTodos: Todo[];
  loadError: string | null;
}) {
  const [todos, setTodos] = useState<Todo[]>(initialTodos);
  const [filter, setFilter] = useState<Filter>("all");
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(loadError);

  const visible = useMemo(() => {
    if (filter === "active") return todos.filter((t) => !t.completed);
    if (filter === "done") return todos.filter((t) => t.completed);
    return todos;
  }, [todos, filter]);

  const remaining = todos.filter((t) => !t.completed).length;

  async function run(action: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (err) {
      setError(err instanceof Error ? err.message : "操作に失敗しました");
    } finally {
      setBusy(false);
    }
  }

  function handleAdd(event: React.FormEvent) {
    event.preventDefault();
    if (!draft.trim()) return;

    void run(async () => {
      const created = await createTodo(draft);
      setTodos((prev) => [created, ...prev]);
      setDraft("");
    });
  }

  function handleToggle(todo: Todo) {
    void run(async () => {
      const updated = await updateTodo(todo.id, { completed: !todo.completed });
      setTodos((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
    });
  }

  function handleRename(todo: Todo, title: string) {
    if (title.trim() === todo.title) return;

    void run(async () => {
      const updated = await updateTodo(todo.id, { title });
      setTodos((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
    });
  }

  function handleDelete(todo: Todo) {
    void run(async () => {
      await deleteTodo(todo.id);
      setTodos((prev) => prev.filter((t) => t.id !== todo.id));
    });
  }

  return (
    <section className="card">
      <form className="composer" onSubmit={handleAdd}>
        <input
          className="composer-input"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="やることを書く"
          aria-label="やること"
          maxLength={500}
        />
        <button className="button" type="submit" disabled={busy || !draft.trim()}>
          追加
        </button>
      </form>

      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}

      <div className="toolbar">
        <div className="filters" role="group" aria-label="絞り込み">
          {FILTERS.map(({ key, label }) => (
            <button
              key={key}
              type="button"
              className={`filter ${filter === key ? "filter-active" : ""}`}
              aria-pressed={filter === key}
              onClick={() => setFilter(key)}
            >
              {label}
            </button>
          ))}
        </div>
        <span className="count">未完了 {remaining} 件</span>
      </div>

      {visible.length === 0 ? (
        <p className="empty">
          {todos.length === 0 ? "まだ TODO はありません" : "この条件に合う TODO はありません"}
        </p>
      ) : (
        <ul className="list">
          {visible.map((todo) => (
            <TodoItem
              key={todo.id}
              todo={todo}
              busy={busy}
              onToggle={() => handleToggle(todo)}
              onRename={(title) => handleRename(todo, title)}
              onDelete={() => handleDelete(todo)}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
