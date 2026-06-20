// Client for the SuiScope dashboard API (the Axum `suiscope-dashboard` crate).
// Override the base URL with NEXT_PUBLIC_API_URL when the server runs elsewhere.
export const API_BASE =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:7731";

// Mirrors `TrackedObject` in crates/core/src/types.rs
export interface TrackedObject {
  id: number | null;
  object_id: string;
  object_type: string | null;
  alias: string | null;
  owner: string | null;
  package_id: string | null;
  version: string | null;
  digest: string | null;
  tx_digest: string | null;
  network: string;
  created_at: string | null;
  updated_at: string | null;
}

// Mirrors `Transaction` in crates/core/src/types.rs
export interface Transaction {
  id: number | null;
  tx_digest: string;
  command: string | null;
  status: string;
  gas_used: number | null;
  gas_owner: string | null;
  package_id: string | null;
  module_name: string | null;
  function: string | null;
  raw_response: string | null;
  network: string;
  created_at: string | null;
}

// Mirrors `ErrorEntry` in crates/core/src/types.rs
export interface ErrorEntry {
  id: number | null;
  error_code: string | null;
  error_message: string;
  module_id: string | null;
  explanation: string | null;
  tx_digest: string | null;
  network: string;
  created_at: string | null;
}

async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, { cache: "no-store" });
  if (!res.ok) {
    throw new Error(`${path} → ${res.status} ${res.statusText}`);
  }
  return (await res.json()) as T;
}

export const fetchObjects = () => getJson<TrackedObject[]>("/api/objects");
export const fetchTransactions = () =>
  getJson<Transaction[]>("/api/transactions");
export const fetchErrors = () => getJson<ErrorEntry[]>("/api/errors");
