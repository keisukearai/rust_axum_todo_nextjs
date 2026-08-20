export type Todo = {
  id: number;
  title: string;
  completed: boolean;
  created_at: string;
  updated_at: string;
};

export type TodoPatch = {
  title?: string;
  completed?: boolean;
};

// Rust API のベース URL。ブラウザからも直接叩くので NEXT_PUBLIC_ で公開する。
// NEXT_PUBLIC_* はビルド時にバンドルへ埋め込まれるため、本番ビルドの時点で値が要る。
// 未設定のまま本番に出ると利用者のブラウザが自分の localhost を叩いてしまうので、
// 開発時だけ既定値を許し、本番ビルドではビルドを失敗させる。
const configuredApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL;

if (!configuredApiBaseUrl && process.env.NODE_ENV === "production") {
  throw new Error(
    "NEXT_PUBLIC_API_BASE_URL が未設定です。ビルド前に .env.local などで指定してください",
  );
}

export const API_BASE_URL = configuredApiBaseUrl ?? "http://localhost:8080";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    cache: "no-store",
    headers: init?.body ? { "content-type": "application/json" } : undefined,
  });

  if (!res.ok) {
    throw new Error(await errorMessage(res));
  }

  if (res.status === 204) {
    return undefined as T;
  }

  return (await res.json()) as T;
}

async function errorMessage(res: Response): Promise<string> {
  try {
    const body = (await res.json()) as { error?: string };
    if (body.error) return body.error;
  } catch {
    // JSON でないレスポンスはステータスだけ返す
  }
  return `API エラー (${res.status})`;
}

export function fetchTodos(): Promise<Todo[]> {
  return request<Todo[]>("/api/todos");
}

export function createTodo(title: string): Promise<Todo> {
  return request<Todo>("/api/todos", {
    method: "POST",
    body: JSON.stringify({ title }),
  });
}

export function updateTodo(id: number, patch: TodoPatch): Promise<Todo> {
  return request<Todo>(`/api/todos/${id}`, {
    method: "PATCH",
    body: JSON.stringify(patch),
  });
}

export function deleteTodo(id: number): Promise<void> {
  return request<void>(`/api/todos/${id}`, { method: "DELETE" });
}
