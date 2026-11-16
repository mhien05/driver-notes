export type EntryKind = "delivery" | "bonus" | "expense";
export type ExpenseCategory = "fuel" | "food" | "other";

export interface AuthResponse {
  userId: string;
  email: string;
  token: string;
  expiresAt: number;
}

export interface Shift {
  id: string;
  startedAt: number;
  endedAt: number | null;
  createdAt: number;
  updatedAt: number;
}

export interface Entry {
  id: string;
  shiftId: string | null;
  kind: EntryKind;
  deliveryCount: number | null;
  expenseCategory: ExpenseCategory | null;
  amountVnd: number;
  tipVnd: number;
  occurredAt: number;
  note: string;
  createdAt: number;
  updatedAt: number;
}

export type EntryInput = Pick<
  Entry,
  | "shiftId"
  | "kind"
  | "deliveryCount"
  | "expenseCategory"
  | "amountVnd"
  | "tipVnd"
  | "occurredAt"
  | "note"
>;

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
  ) {
    super(message);
  }
}

const apiOrigin = (import.meta.env.VITE_API_ORIGIN ?? "").replace(/\/$/, "");

export async function api<T>(
  path: string,
  token?: string,
  options: RequestInit = {},
): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${apiOrigin}/api/v1${path}`, {
      ...options,
      headers: {
        ...(options.body ? { "Content-Type": "application/json" } : {}),
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...options.headers,
      },
    });
  } catch {
    throw new ApiError(
      "Không thể kết nối đến máy chủ. Kiểm tra mạng và thử lại.",
      0,
    );
  }

  if (!response.ok) {
    if (
      [502, 503, 504].includes(response.status) ||
      (response.status === 500 &&
        !response.headers.get("content-type")?.includes("application/json"))
    ) {
      throw new ApiError(
        "Không thể kết nối đến máy chủ. Kiểm tra mạng và thử lại.",
        0,
      );
    }
    const body = (await response.json().catch(() => null)) as {
      error?: string;
    } | null;
    throw new ApiError(
      body?.error ?? `Yêu cầu không thành công (${response.status}).`,
      response.status,
    );
  }

  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}
