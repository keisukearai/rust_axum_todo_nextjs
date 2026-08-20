"use client";

import { useState } from "react";
import type { Todo } from "@/lib/api";

export default function TodoItem({
  todo,
  busy,
  onToggle,
  onRename,
  onDelete,
}: {
  todo: Todo;
  busy: boolean;
  onToggle: () => void;
  onRename: (title: string) => void;
  onDelete: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(todo.title);

  function commit() {
    setEditing(false);
    const title = draft.trim();
    if (!title) {
      setDraft(todo.title);
      return;
    }
    onRename(title);
  }

  function cancel() {
    setEditing(false);
    setDraft(todo.title);
  }

  return (
    <li className={`item ${todo.completed ? "item-done" : ""}`}>
      <input
        type="checkbox"
        className="checkbox"
        checked={todo.completed}
        disabled={busy}
        onChange={onToggle}
        aria-label={`${todo.title} を完了にする`}
      />

      {editing ? (
        <input
          className="edit-input"
          value={draft}
          autoFocus
          maxLength={500}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") commit();
            if (e.key === "Escape") cancel();
          }}
          aria-label="タイトルを編集"
        />
      ) : (
        <button
          type="button"
          className="title"
          onClick={() => {
            setDraft(todo.title);
            setEditing(true);
          }}
          title="クリックで編集"
        >
          {todo.title}
        </button>
      )}

      <button
        type="button"
        className="delete"
        onClick={onDelete}
        disabled={busy}
        aria-label={`${todo.title} を削除`}
      >
        削除
      </button>
    </li>
  );
}
