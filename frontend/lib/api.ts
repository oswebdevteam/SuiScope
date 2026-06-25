// Client for the SuiScope dashboard API (the Axum `suiscope-dashboard` crate).
// Override the base URL with NEXT_PUBLIC_API_URL when the server runs elsewhere.
export const API_BASE =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:7731";


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

export interface InspectResult {
  object_id: string;
  version: string | null;
  digest: string | null;
  object_type: string | null;
  owner: string | null;
  previous_transaction: string | null;
  storage_rebate: string | null;
  content: Record<string, unknown> | null;
  explorer_tx_url: string | null;
  explorer_object_url: string;
  network: string;
  error?: string;
}

export interface GraphNode {
  id: string;
  label: string;
  type: string;
  network: string;
  package_id: string | null;
}

export interface GraphEdge {
  from: string;
  to: string;
  label: string;
}

export interface ObjectGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
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
export const fetchInspect = (id: string) =>
  getJson<InspectResult>(`/api/inspect/${encodeURIComponent(id)}`);
export const fetchGraph = () => getJson<ObjectGraph>("/api/graph");

export async function uploadToWalrus(): Promise<{ blob_id: string; message: string }> {
  const res = await fetch(`${API_BASE}/api/walrus/upload`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: "Upload failed" }));
    throw new Error(err.error || `Upload failed with status ${res.status}`);
  }
  return await res.json();
}

export async function importFromWalrus(blobId: string): Promise<{ message: string }> {
  const res = await fetch(`${API_BASE}/api/walrus/import/${encodeURIComponent(blobId)}`, {
    method: "GET",
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: "Import failed" }));
    throw new Error(err.error || `Import failed with status ${res.status}`);
  }
  return await res.json();
}
