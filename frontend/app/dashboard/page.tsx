"use client";

import { useEffect, useMemo, useState } from "react";
import {
  fetchObjects,
  fetchTransactions,
  fetchErrors,
  fetchInspect,
  fetchGraph,
  uploadToWalrus,
  importFromWalrus,
  type TrackedObject,
  type Transaction,
  type ErrorEntry,
  type InspectResult,
  type ObjectGraph,
} from "@/lib/api";

const navItems = [
  ["database", "Object Registry"],
  ["pulse", "Tx Timeline"],
  ["alert", "Error Log"],
  ["graph", "Object Graph"],
  ["search", "Inspect Object"],
  ["cloud", "Walrus Sync"],
];


function shorten(id: string): string {
  return id.length > 13 ? `${id.slice(0, 6)}...${id.slice(-4)}` : id;
}

// Derive a type badge label from a Move object type string.
// Packages have no object_type; capability objects are matched by name.
function deriveBadge(objectType: string | null): string {
  if (!objectType) return "Package";
  if (objectType.includes("AdminCap")) return "AdminCap";
  if (objectType.includes("TreasuryCap")) return "TreasuryCap";
  if (objectType.includes("UpgradeCap")) return "UpgradeCap";
  // Last `::Segment` without generic params, e.g. 0x2::coin::Coin<..> -> Coin
  const tail = objectType.split("<")[0].split("::").pop();
  return tail || "Object";
}

interface Row {
  alias: string;
  objectId: string;
  fullId: string;
  badge: string;
  pkg: string;
  network: string;
}

function toRow(o: TrackedObject): Row {
  return {
    alias: o.alias ?? shorten(o.object_id),
    objectId: shorten(o.object_id),
    fullId: o.object_id,
    badge: deriveBadge(o.object_type),
    pkg: o.package_id ? shorten(o.package_id) : "—",
    network: o.network,
  };
}

function Icon({ name }: { name: string }) {
  const common = {
    width: 22,
    height: 22,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.8,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
  };

  if (name === "database") {
    return (
      <svg {...common}>
        <ellipse cx="12" cy="5" rx="8" ry="3" />
        <path d="M4 5v6c0 1.7 3.6 3 8 3s8-1.3 8-3V5" />
        <path d="M4 11v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6" />
      </svg>
    );
  }

  if (name === "pulse") {
    return (
      <svg {...common}>
        <path d="M3 12h4l2-5 4 10 2-5h6" />
      </svg>
    );
  }

  if (name === "alert") {
    return (
      <svg {...common}>
        <path d="m12 4 9 16H3L12 4Z" />
        <path d="M12 9v4" />
        <path d="M12 17h.01" />
      </svg>
    );
  }

  if (name === "graph") {
    return (
      <svg {...common}>
        <circle cx="6" cy="6" r="2" />
        <circle cx="18" cy="5" r="2" />
        <circle cx="8" cy="18" r="2" />
        <path d="M8 7.2 16 5.8" />
        <path d="M7 8v8" />
        <path d="M9.6 17 18 7" />
      </svg>
    );
  }

  if (name === "search") {
    return (
      <svg {...common}>
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.8-3.8" />
      </svg>
    );
  }

  if (name === "cloud") {
    return (
      <svg {...common}>
        <path d="M17.5 18H8a5 5 0 1 1 1.1-9.9A6.2 6.2 0 0 1 21 11.4 3.5 3.5 0 0 1 17.5 18Z" />
      </svg>
    );
  }

  if (name === "cube") {
    return (
      <svg {...common} width={16} height={16}>
        <path d="m12 3 6 3.4v6.8L12 17l-6-3.8V6.4L12 3Z" />
        <path d="M6 6.4 12 10l6-3.6" />
        <path d="M12 10v7" />
      </svg>
    );
  }

  if (name === "tag") {
    return (
      <svg {...common}>
        <path d="M20 13.4 13.4 20a2 2 0 0 1-2.8 0L4 13.4V4h9.4L20 10.6a2 2 0 0 1 0 2.8Z" />
        <path d="M8 8h.01" />
      </svg>
    );
  }

  return (
    <svg {...common}>
      <path d="M8 7h8" />
      <path d="M8 12h8" />
      <path d="M8 17h8" />
    </svg>
  );
}

function Logo() {
  return (
    <div className="flex items-center justify-between text-sm font-extrabold text-[#D9D9D9]">
      <img
        src="/suiscope-logo.svg"
        alt="SuiScope"
        className="h-16 w-auto object-contain"
      />
      <div className="flex gap-4 text-xs leading-tight max-[760px]:hidden">
        <span>DEBUG</span>
        <span>REGISTRY</span>
      </div>
    </div>
  );
}

function TypeBadge({ label }: { label: string }) {
  if (label === "AdminCap") {
    return (
      <span className="inline-flex h-[25px] w-fit items-center rounded border border-[#4BFFA5] bg-[#4BFFA526] px-3 text-base leading-none text-[#4BFFA5]">
        {label}
      </span>
    );
  }

  if (label === "TreasuryCap") {
    return (
      <span className="inline-flex h-[25px] w-fit items-center rounded border border-[#C4B93A] bg-[#C4B93A26] px-3 text-base leading-none text-[#C4B93A]">
        {label}
      </span>
    );
  }

  if (label === "UpgradeCap") {
    return (
      <span className="inline-flex h-[25px] w-fit items-center rounded border border-[#D9D9D99E] bg-[#D9D9D926] px-3 text-base leading-none text-[#D9D9D9]">
        {label}
      </span>
    );
  }

  return (
    <span className="inline-flex h-[25px] w-fit items-center rounded border border-[#2D9499] bg-[#1B5B682E] px-3 text-base leading-none text-[#2D9499]">
      {label}
    </span>
  );
}

function NetworkLabel({ network }: { network: string }) {
  return (
    <span className={network === "mainnet" ? "text-[#4BFFA5]" : "text-[#C4B93A]"}>
      {network}
    </span>
  );
}

function StatItem({
  icon,
  label,
  value,
}: {
  icon: string;
  label: string;
  value: number | string;
}) {
  return (
    <span className="inline-flex items-center gap-[7px]">
      <Icon name={icon} /> {label} <strong>{value}</strong>
    </span>
  );
}

export default function Dashboard() {
  const [objects, setObjects] = useState<TrackedObject[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [errors, setErrors] = useState<ErrorEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [activeTab, setActiveTab] = useState("Object Registry");

  const [query, setQuery] = useState("");
  const [network, setNetwork] = useState("all");
  const [copied, setCopied] = useState<string | null>(null);

  // Inspect tab state
  const [inspectQuery, setInspectQuery] = useState("");
  const [inspectResult, setInspectResult] = useState<InspectResult | null>(null);
  const [inspectLoading, setInspectLoading] = useState(false);
  const [inspectError, setInspectError] = useState<string | null>(null);

  // Graph tab state
  const [graph, setGraph] = useState<ObjectGraph | null>(null);
  const [graphLoading, setGraphLoading] = useState(false);
  const [graphError, setGraphError] = useState<string | null>(null);

  // Walrus tab state
  const [walrusAction, setWalrusAction] = useState<"upload" | "import">("upload");
  const [walrusLoading, setWalrusLoading] = useState(false);
  const [walrusError, setWalrusError] = useState<string | null>(null);
  const [walrusSuccess, setWalrusSuccess] = useState<string | null>(null);
  const [walrusBlobId, setWalrusBlobId] = useState("");
  const [lastUploadedBlobId, setLastUploadedBlobId] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [objs, txs, errs] = await Promise.all([
          fetchObjects(),
          fetchTransactions().catch(() => []),
          fetchErrors().catch(() => []),
        ]);
        if (cancelled) return;
        setObjects(objs);
        setTransactions(txs);
        setErrors(errs);
        setLoadError(null);
      } catch (e) {
        if (!cancelled) {
          setLoadError(e instanceof Error ? e.message : "Failed to load data");
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const networks = useMemo(
    () => Array.from(new Set(objects.map((o) => o.network))).sort(),
    [objects],
  );

  const rows = useMemo(() => objects.map(toRow), [objects]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return rows.filter((r) => {
      if (network !== "all" && r.network !== network) return false;
      if (!q) return true;
      return (
        r.alias.toLowerCase().includes(q) ||
        r.fullId.toLowerCase().includes(q) ||
        r.pkg.toLowerCase().includes(q)
      );
    });
  }, [rows, query, network]);

  const packageCount = useMemo(
    () => rows.filter((r) => r.badge === "Package").length,
    [rows],
  );

  async function copyId(fullId: string) {
    try {
      await navigator.clipboard.writeText(fullId);
      setCopied(fullId);
      setTimeout(() => setCopied((c) => (c === fullId ? null : c)), 1200);
    } catch {
      /* clipboard unavailable */
    }
  }

  async function runInspect(id: string) {
    const trimmed = id.trim();
    if (!trimmed) return;
    setInspectLoading(true);
    setInspectError(null);
    setInspectResult(null);
    try {
      const result = await fetchInspect(trimmed);
      if (result.error) {
        setInspectError(result.error);
      } else {
        setInspectResult(result);
      }
    } catch (e) {
      setInspectError(e instanceof Error ? e.message : "Request failed");
    } finally {
      setInspectLoading(false);
    }
  }

  async function loadGraph() {
    setGraphLoading(true);
    setGraphError(null);
    try {
      const data = await fetchGraph();
      setGraph(data);
    } catch (e) {
      setGraphError(e instanceof Error ? e.message : "Failed to load graph");
    } finally {
      setGraphLoading(false);
    }
  }

  async function handleWalrusUpload() {
    setWalrusLoading(true);
    setWalrusError(null);
    setWalrusSuccess(null);
    try {
      const result = await uploadToWalrus();
      setLastUploadedBlobId(result.blob_id);
      setWalrusSuccess(`Upload successful! Blob ID: ${result.blob_id}`);
    } catch (e) {
      setWalrusError(e instanceof Error ? e.message : "Upload failed");
    } finally {
      setWalrusLoading(false);
    }
  }

  async function handleWalrusImport() {
    const trimmed = walrusBlobId.trim();
    if (!trimmed) {
      setWalrusError("Please enter a Blob ID");
      return;
    }
    setWalrusLoading(true);
    setWalrusError(null);
    setWalrusSuccess(null);
    try {
      await importFromWalrus(trimmed);
      setWalrusSuccess("Registry imported and merged successfully!");
      setWalrusBlobId("");
      // Reload objects after import
      const objs = await fetchObjects();
      setObjects(objs);
    } catch (e) {
      setWalrusError(e instanceof Error ? e.message : "Import failed");
    } finally {
      setWalrusLoading(false);
    }
  }

  return (
    <main className="min-h-screen bg-[#030912]">
      <aside className="top-0 flex flex-col border-r border-[#D9D9D933] bg-[#061227] p-4 min-[761px]:fixed min-[761px]:left-0 min-[761px]:top-0 min-[761px]:h-screen min-[761px]:w-[320px] min-[761px]:px-4 min-[761px]:py-4 max-[1050px]:min-[761px]:w-[260px]">
        <div className="-mx-4 border-b border-[#D9D9D933] px-4 pb-3">
          <Logo />
        </div>

        <section className="-mx-4 border-b border-[#D9D9D933] px-4 py-3">
          <h2 className="mb-2 mt-0 text-base font-medium text-[#D9D9D9]">PROJECT</h2>
          <p className="m-0 flex items-center gap-2 text-sm font-bold text-[#D9D9D9]">
            <span className="h-2 w-2 rounded-full bg-[#4BFFA5]" />
            Local Registry
          </p>
        </section>

        <section className="flex-1 py-3 min-h-0 overflow-y-auto">
          <h2 className="mb-3 mt-0 text-base font-medium text-[#D9D9D9]">NAVIGATION</h2>
          <nav
            aria-label="Primary navigation"
            className="grid gap-2 max-[760px]:grid-cols-2"
          >
            {navItems.map(([icon, label]) => (
              <a
                href="#"
                onClick={(e) => { e.preventDefault(); setActiveTab(label); }}
                className={`flex items-center gap-2 text-sm text-[#D9D9D9] transition-colors hover:text-white py-1 ${
                  label === activeTab ? "text-white font-bold" : ""
                }`}
                key={label}
              >
                <Icon name={icon} />
                <span>{label}</span>
              </a>
            ))}
          </nav>
        </section>

        <div className="-mx-4 border-t border-[#D9D9D933] px-4 pt-2 text-center text-xs text-[#8F98AA] max-[760px]:hidden">
          <code className="text-xs text-[#4BFFA5] block mb-1">suiscope publish</code>
          <span className="block text-[10px]">Auto-registers all spawned</span>
        </div>
      </aside>

      <section className="min-w-0 bg-[#030912] min-[761px]:ml-[320px] max-[1050px]:min-[761px]:ml-[260px]">
        <header className="border-b border-[#D9D9D933] bg-[#061227] px-[18px] py-5 min-[761px]:min-h-[123px] min-[761px]:px-[42px] min-[761px]:pb-3.5">
          <div className="mb-[26px] flex flex-wrap gap-[15px] text-sm text-[#D9D9D9] [&_svg]:h-4 [&_svg]:w-4">
            <StatItem icon="cube" label="Packages" value={packageCount} />
            <StatItem icon="cube" label="Objects" value={objects.length} />
            <StatItem icon="pulse" label="Transactions" value={transactions.length} />
            <StatItem icon="alert" label="Errors" value={errors.length} />
          </div>
          <h1 className="mb-1.5 mt-0 text-xl font-extrabold text-[#D9D9D9]">
            {activeTab}
          </h1>
          <p className="m-0 text-base text-[#8F98AA]">
            {activeTab === "Object Registry" && "All tracked Package IDs and Object IDs for this project"}
            {activeTab === "Error Log" && "Recent errors and Move aborts"}
            {activeTab === "Tx Timeline" && "Recent transactions"}
            {activeTab === "Inspect Object" && "Fetch live on-chain state for any object or package ID"}
            {activeTab === "Object Graph" && "Visualize relationships between objects and packages"}
            {activeTab === "Walrus Sync" && "Sync registry data to Walrus storage"}
            {activeTab !== "Object Registry" && activeTab !== "Error Log" && activeTab !== "Tx Timeline" && activeTab !== "Inspect Object" && activeTab !== "Object Graph" && activeTab !== "Walrus Sync" && "Details and settings"}
          </p>
        </header>

        <div className="px-[18px] pt-[22px] min-[761px]:pl-6 min-[761px]:pr-[26px] min-[1051px]:pl-[53px]">
          {activeTab === "Object Registry" && (
            <>
              <div className="mb-6 flex flex-col gap-4 min-[900px]:flex-row min-[900px]:items-center min-[900px]:justify-between">
            <label className="flex h-[52px] w-full max-w-[536px] items-center gap-3.5 rounded-md border border-[#29466D] bg-[#061227] px-[30px] text-[#8F98AA]">
              <Icon name="search" />
              <input
                aria-label="Search registry"
                placeholder="Search alias, ID, or package.."
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                className="w-full min-w-0 border-0 bg-transparent text-[#D9D9D9] outline-0 placeholder:text-[#8F98AA]"
              />
            </label>

            <div className="flex items-center gap-14 text-xl text-[#8F98AA]">
              <div className="relative flex h-[58px] items-center rounded-md border border-[#29466D] bg-[#061227] px-3.5 text-2xl text-[#D9D9D9]">
                <select
                  aria-label="Filter by network"
                  value={network}
                  onChange={(e) => setNetwork(e.target.value)}
                  className="cursor-pointer appearance-none bg-transparent pr-8 text-[#D9D9D9] outline-0"
                >
                  <option value="all" className="bg-[#061227]">
                    All Networks
                  </option>
                  {networks.map((n) => (
                    <option key={n} value={n} className="bg-[#061227]">
                      {n}
                    </option>
                  ))}
                </select>
                <svg
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                  className="pointer-events-none absolute right-3.5"
                >
                  <path d="m6 9 6 6 6-6" />
                </svg>
              </div>
              <span>
                {filtered.length}/{rows.length} objects
              </span>
            </div>
          </div>

          <div
            className="min-h-[calc(100vh-220px)] overflow-x-auto rounded-[5px] border border-[#122848] bg-[#061227]"
            role="table"
            aria-label="Object registry"
          >
            <div
              className="grid min-h-[70px] min-w-[900px] grid-cols-[minmax(210px,1.1fr)_minmax(170px,0.9fr)_minmax(130px,0.7fr)_minmax(145px,0.8fr)_minmax(120px,0.7fr)] items-center px-7 text-base font-bold text-[#8F98AA]"
              role="row"
            >
              <span role="columnheader">ALIAS</span>
              <span role="columnheader">OBJECT ID</span>
              <span role="columnheader">TYPE</span>
              <span role="columnheader">PACKAGE</span>
              <span role="columnheader">NETWORK</span>
            </div>

            {loading && (
              <div className="px-7 py-10 text-base text-[#8F98AA]">
                Loading registry…
              </div>
            )}

            {!loading && loadError && (
              <div className="px-7 py-10 text-base text-[#C4B93A]">
                Couldn’t reach the dashboard API ({loadError}). Is{" "}
                <code className="text-[#4BFFA5]">suiscope dashboard</code> running
                on port 7731?
              </div>
            )}

            {!loading && !loadError && filtered.length === 0 && (
              <div className="px-7 py-10 text-base text-[#8F98AA]">
                {rows.length === 0
                  ? "No tracked objects yet. Run suiscope publish to register some."
                  : "No objects match your filters."}
              </div>
            )}

            {!loading &&
              !loadError &&
              filtered.map((row) => (
                <div
                  className="grid min-h-[84px] min-w-[900px] grid-cols-[minmax(210px,1.1fr)_minmax(170px,0.9fr)_minmax(130px,0.7fr)_minmax(145px,0.8fr)_minmax(120px,0.7fr)] items-center px-7 text-sm text-[#8F98AA]"
                  role="row"
                  key={row.fullId}
                >
                  <span className="inline-flex items-center gap-[18px] text-[#D9D9D9] [&_svg]:text-[#4BFFA5]" role="cell">
                    <Icon name="tag" />
                    {row.alias}
                  </span>
                  <span className="inline-flex items-center gap-2" role="cell">
                    {row.objectId}
                    <button
                      aria-label={`Copy ${row.alias} object id`}
                      onClick={() => copyId(row.fullId)}
                      className="grid h-6 w-6 place-items-center border-0 bg-transparent p-0 text-[#8F98AA] hover:text-[#4BFFA5]"
                    >
                      {copied === row.fullId ? (
                        <svg
                          width="17"
                          height="17"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="#4BFFA5"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          aria-hidden="true"
                        >
                          <path d="M20 6 9 17l-5-5" />
                        </svg>
                      ) : (
                        <svg
                          width="17"
                          height="17"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          aria-hidden="true"
                        >
                          <rect width="13" height="13" x="9" y="9" rx="2" />
                          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                        </svg>
                      )}
                    </button>
                  </span>
                  <span role="cell">
                    <TypeBadge label={row.badge} />
                  </span>
                  <span role="cell">{row.pkg}</span>
                  <span role="cell">
                    <NetworkLabel network={row.network} />
                  </span>
                </div>
              ))}
            </div>
            </>
          )}

          {activeTab === "Error Log" && (
            <div className="min-h-[calc(100vh-220px)] overflow-x-auto rounded-[5px] border border-[#122848] bg-[#061227]" role="table" aria-label="Error Log">
              <div className="grid min-h-[70px] min-w-[900px] grid-cols-[minmax(150px,1fr)_minmax(150px,1fr)_minmax(150px,1fr)_minmax(300px,2fr)] items-center px-7 text-base font-bold text-[#8F98AA]" role="row">
                <span role="columnheader">ERROR CODE</span>
                <span role="columnheader">NETWORK</span>
                <span role="columnheader">TX DIGEST</span>
                <span role="columnheader">MESSAGE</span>
              </div>
              {loading && <div className="px-7 py-10 text-base text-[#8F98AA]">Loading errors…</div>}
              {!loading && errors.length === 0 && <div className="px-7 py-10 text-base text-[#8F98AA]">No errors tracked yet. Run suiscope explain to log an error.</div>}
              {!loading && errors.map((err, i) => (
                <div key={i} className="grid min-h-[84px] min-w-[900px] grid-cols-[minmax(150px,1fr)_minmax(150px,1fr)_minmax(150px,1fr)_minmax(300px,2fr)] items-center px-7 text-sm text-[#8F98AA]" role="row">
                  <span className="text-[#C4B93A] font-medium" role="cell">{err.error_code || "Unknown"}</span>
                  <span role="cell"><NetworkLabel network={err.network} /></span>
                  <span role="cell" className="text-[#D9D9D9]">{err.tx_digest ? shorten(err.tx_digest) : "-"}</span>
                  <span role="cell" className="text-[#D9D9D9]">{err.error_message}</span>
                </div>
              ))}
            </div>
          )}

          {activeTab === "Inspect Object" && (
            <div className="max-w-[860px]">
              {/* Search bar */}
              <form
                className="mb-8 flex gap-3"
                onSubmit={(e) => { e.preventDefault(); runInspect(inspectQuery); }}
              >
                <label className="flex h-[52px] flex-1 items-center gap-3.5 rounded-md border border-[#29466D] bg-[#061227] px-[30px] text-[#8F98AA]">
                  <Icon name="search" />
                  <input
                    aria-label="Object or package ID"
                    placeholder="Enter object ID or alias (e.g. 0x2)"
                    value={inspectQuery}
                    onChange={(e) => setInspectQuery(e.target.value)}
                    className="w-full min-w-0 border-0 bg-transparent text-[#D9D9D9] outline-0 placeholder:text-[#8F98AA]"
                  />
                </label>
                <button
                  type="submit"
                  disabled={inspectLoading || !inspectQuery.trim()}
                  className="h-[52px] rounded-md border border-[#2D9499] bg-[#1B5B682E] px-6 text-base text-[#2D9499] transition-colors hover:bg-[#2D949926] disabled:opacity-40"
                >
                  {inspectLoading ? "Fetching…" : "Inspect"}
                </button>
              </form>

              {/* Error state */}
              {inspectError && (
                <div className="mb-6 rounded-md border border-[#e96b6b44] bg-[#e96b6b14] px-6 py-4 text-sm text-[#e96b6b]">
                  {inspectError}
                </div>
              )}

              {/* Loading skeleton */}
              {inspectLoading && (
                <div className="rounded-[5px] border border-[#122848] bg-[#061227] px-8 py-10 text-base text-[#8F98AA]">
                  Fetching on-chain state…
                </div>
              )}

              {/* Result */}
              {!inspectLoading && inspectResult && (
                <div className="rounded-[5px] border border-[#122848] bg-[#061227]">
                  {/* Metadata section */}
                  <div className="border-b border-[#122848] px-8 py-6">
                    <h2 className="mb-5 mt-0 text-base font-bold uppercase tracking-wide text-[#8F98AA]">Metadata</h2>
                    <dl className="grid grid-cols-[minmax(140px,auto)_1fr] gap-x-8 gap-y-4 text-sm">
                      <dt className="text-[#8F98AA]">Object ID</dt>
                      <dd className="m-0 flex items-center gap-2 font-mono text-[#D9D9D9] break-all">
                        {inspectResult.object_id}
                        <a
                          href={inspectResult.explorer_object_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          aria-label="View on explorer"
                          className="shrink-0 text-[#2D9499] hover:text-[#4BFFA5]"
                        >
                          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                            <polyline points="15 3 21 3 21 9" />
                            <line x1="10" y1="14" x2="21" y2="3" />
                          </svg>
                        </a>
                      </dd>

                      <dt className="text-[#8F98AA]">Type</dt>
                      <dd className="m-0 text-[#D9D9D9] break-all">
                        {inspectResult.object_type ? (
                          <TypeBadge label={deriveBadge(inspectResult.object_type)} />
                        ) : (
                          <TypeBadge label="Package" />
                        )}
                        {inspectResult.object_type && (
                          <span className="ml-2 font-mono text-xs text-[#8F98AA]">{inspectResult.object_type}</span>
                        )}
                      </dd>

                      <dt className="text-[#8F98AA]">Owner</dt>
                      <dd className="m-0 font-mono text-[#D9D9D9] break-all">{inspectResult.owner ?? "—"}</dd>

                      <dt className="text-[#8F98AA]">Version</dt>
                      <dd className="m-0 text-[#D9D9D9]">{inspectResult.version ?? "—"}</dd>

                      <dt className="text-[#8F98AA]">Digest</dt>
                      <dd className="m-0 font-mono text-[#D9D9D9] break-all">{inspectResult.digest ?? "—"}</dd>

                      <dt className="text-[#8F98AA]">Network</dt>
                      <dd className="m-0"><NetworkLabel network={inspectResult.network} /></dd>

                      <dt className="text-[#8F98AA]">Storage Rebate</dt>
                      <dd className="m-0 text-[#D9D9D9]">{inspectResult.storage_rebate ?? "—"}</dd>

                      {inspectResult.previous_transaction && (
                        <>
                          <dt className="text-[#8F98AA]">Last Tx</dt>
                          <dd className="m-0 flex items-center gap-2 font-mono text-[#D9D9D9] break-all">
                            {shorten(inspectResult.previous_transaction)}
                            {inspectResult.explorer_tx_url && (
                              <a
                                href={inspectResult.explorer_tx_url}
                                target="_blank"
                                rel="noopener noreferrer"
                                aria-label="View transaction on explorer"
                                className="shrink-0 text-[#2D9499] hover:text-[#4BFFA5]"
                              >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                                  <polyline points="15 3 21 3 21 9" />
                                  <line x1="10" y1="14" x2="21" y2="3" />
                                </svg>
                              </a>
                            )}
                          </dd>
                        </>
                      )}
                    </dl>
                  </div>

                  {/* Fields / content section */}
                  <div className="px-8 py-6">
                    <h2 className="mb-5 mt-0 text-base font-bold uppercase tracking-wide text-[#8F98AA]">Fields</h2>
                    {inspectResult.content && Object.keys(inspectResult.content).length > 0 ? (
                      <div className="overflow-x-auto rounded border border-[#122848]">
                        <div className="grid min-w-[500px] grid-cols-[minmax(160px,0.6fr)_1fr] border-b border-[#122848] px-5 py-3 text-xs font-bold uppercase tracking-wide text-[#8F98AA]">
                          <span>Field</span>
                          <span>Value</span>
                        </div>
                        {Object.entries(inspectResult.content).map(([key, val]) => {
                          const display =
                            typeof val === "string"
                              ? val
                              : typeof val === "object" && val !== null
                              ? JSON.stringify(val)
                              : String(val);
                          return (
                            <div
                              key={key}
                              className="grid min-w-[500px] grid-cols-[minmax(160px,0.6fr)_1fr] items-start border-b border-[#122848] px-5 py-3 last:border-0 text-sm"
                            >
                              <span className="text-[#4BFFA5] font-mono">{key}</span>
                              <span className="text-[#D9D9D9] font-mono break-all">{display}</span>
                            </div>
                          );
                        })}
                      </div>
                    ) : (
                      <p className="m-0 text-sm text-[#8F98AA]">
                        No content fields available — this is likely a package or unstructured object.
                      </p>
                    )}
                  </div>
                </div>
              )}

              {/* Empty state */}
              {!inspectLoading && !inspectResult && !inspectError && (
                <div className="rounded-[5px] border border-[#122848] bg-[#061227] px-8 py-14 text-center text-sm text-[#8F98AA]">
                  Enter an object ID above and click Inspect to fetch its live on-chain state.
                </div>
              )}
            </div>
          )}

          {activeTab === "Tx Timeline" && (
            <div className="min-h-[calc(100vh-220px)] overflow-x-auto rounded-[5px] border border-[#122848] bg-[#061227]" role="table" aria-label="Tx Timeline">
              <div className="grid min-h-[70px] min-w-[900px] grid-cols-[minmax(150px,1fr)_minmax(120px,0.8fr)_minmax(120px,0.8fr)_minmax(200px,1.5fr)_minmax(150px,1fr)] items-center px-7 text-base font-bold text-[#8F98AA]" role="row">
                <span role="columnheader">TX DIGEST</span>
                <span role="columnheader">NETWORK</span>
                <span role="columnheader">STATUS</span>
                <span role="columnheader">ACTION</span>
                <span role="columnheader">GAS USED</span>
              </div>
              {loading && <div className="px-7 py-10 text-base text-[#8F98AA]">Loading transactions…</div>}
              {!loading && transactions.length === 0 && <div className="px-7 py-10 text-base text-[#8F98AA]">No transactions tracked yet.</div>}
              {!loading && transactions.map((tx, i) => (
                <div key={i} className="grid min-h-[84px] min-w-[900px] grid-cols-[minmax(150px,1fr)_minmax(120px,0.8fr)_minmax(120px,0.8fr)_minmax(200px,1.5fr)_minmax(150px,1fr)] items-center px-7 text-sm text-[#8F98AA]" role="row">
                  <span className="text-[#D9D9D9] font-mono" role="cell">
                    {tx.tx_digest ? shorten(tx.tx_digest) : "-"}
                  </span>
                  <span role="cell"><NetworkLabel network={tx.network} /></span>
                  <span role="cell" className={tx.status && tx.status.toLowerCase() === "success" ? "text-[#4BFFA5]" : "text-[#e96b6b]"}>
                    {tx.status || "Unknown"}
                  </span>
                  <span role="cell" className="text-[#D9D9D9]">
                    {tx.function ? `${tx.module_name}::${tx.function}` : (tx.command || "Unknown")}
                  </span>
                  <span role="cell" className="text-[#D9D9D9]">
                    {tx.gas_used ? tx.gas_used.toLocaleString() : "-"}
                  </span>
                </div>
              ))}
            </div>
          )}

          {activeTab === "Object Graph" && (
            <div className="max-w-[1200px]">
              <div className="mb-6 flex items-center justify-between">
                <p className="text-sm text-[#8F98AA]">
                  {graph ? `${graph.nodes.length} nodes, ${graph.edges.length} edges` : "Load graph data to visualize"}
                </p>
                <button
                  onClick={loadGraph}
                  disabled={graphLoading}
                  className="rounded-md border border-[#2D9499] bg-[#1B5B682E] px-4 py-2 text-sm text-[#2D9499] transition-colors hover:bg-[#2D949926] disabled:opacity-40"
                >
                  {graphLoading ? "Loading..." : graph ? "Refresh Graph" : "Load Graph"}
                </button>
              </div>

              {graphError && (
                <div className="mb-6 rounded-md border border-[#e96b6b44] bg-[#e96b6b14] px-6 py-4 text-sm text-[#e96b6b]">
                  {graphError}
                </div>
              )}

              {graphLoading && (
                <div className="min-h-[calc(100vh-280px)] rounded-[5px] border border-[#122848] bg-[#061227] px-8 py-10 text-center text-base text-[#8F98AA]">
                  Loading graph data...
                </div>
              )}

              {!graphLoading && !graph && !graphError && (
                <div className="min-h-[calc(100vh-280px)] rounded-[5px] border border-[#122848] bg-[#061227] px-8 py-14 text-center">
                  <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="mx-auto mb-4 text-[#2D9499]">
                    <circle cx="6" cy="6" r="2" />
                    <circle cx="18" cy="5" r="2" />
                    <circle cx="8" cy="18" r="2" />
                    <path d="M8 7.2 16 5.8" />
                    <path d="M7 8v8" />
                    <path d="M9.6 17 18 7" />
                  </svg>
                  <h3 className="mb-2 text-lg font-bold text-[#D9D9D9]">Object Graph</h3>
                  <p className="text-sm text-[#8F98AA]">Click "Load Graph" to visualize relationships between objects.</p>
                </div>
              )}

              {!graphLoading && graph && (
                <div className="min-h-[calc(100vh-280px)] rounded-[5px] border border-[#122848] bg-[#061227] p-6">
                  {graph.nodes.length === 0 ? (
                    <div className="py-20 text-center text-sm text-[#8F98AA]">
                      No objects in registry. Run <code className="text-[#4BFFA5]">suiscope publish</code> to add some.
                    </div>
                  ) : (
                    <div className="space-y-6">
                      <div>
                        <h3 className="mb-3 text-base font-bold text-[#D9D9D9]">Nodes</h3>
                        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                          {graph.nodes.map((node) => (
                            <div key={node.id} className="rounded border border-[#122848] bg-[#030912] p-3">
                              <div className="mb-1 flex items-center justify-between">
                                <span className="font-mono text-sm font-bold text-[#4BFFA5]">{node.label}</span>
                                <TypeBadge label={node.type === "package" ? "Package" : "Object"} />
                              </div>
                              <p className="mb-1 font-mono text-xs text-[#8F98AA] break-all">{node.id}</p>
                              <p className="text-xs text-[#8F98AA]">
                                Network: <NetworkLabel network={node.network} />
                              </p>
                            </div>
                          ))}
                        </div>
                      </div>

                      {graph.edges.length > 0 && (
                        <div>
                          <h3 className="mb-3 text-base font-bold text-[#D9D9D9]">Relationships</h3>
                          <div className="space-y-2">
                            {graph.edges.map((edge, i) => (
                              <div key={i} className="flex items-center gap-3 rounded border border-[#122848] bg-[#030912] p-3 text-sm">
                                <span className="font-mono text-[#D9D9D9]">{shorten(edge.from)}</span>
                                <span className="text-[#8F98AA]">→</span>
                                <span className="rounded bg-[#2D949926] px-2 py-1 text-xs text-[#2D9499]">{edge.label}</span>
                                <span className="text-[#8F98AA]">→</span>
                                <span className="font-mono text-[#D9D9D9]">{shorten(edge.to)}</span>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {activeTab === "Walrus Sync" && (
            <div className="max-w-[860px]">
              <div className="mb-6 flex gap-3">
                <button
                  onClick={() => { setWalrusAction("upload"); setWalrusError(null); setWalrusSuccess(null); }}
                  className={`flex-1 rounded-md border px-4 py-2 text-sm transition-colors ${
                    walrusAction === "upload"
                      ? "border-[#2D9499] bg-[#1B5B682E] text-[#2D9499] font-bold"
                      : "border-[#29466D] bg-[#061227] text-[#8F98AA] hover:text-[#D9D9D9]"
                  }`}
                >
                  Upload to Walrus
                </button>
                <button
                  onClick={() => { setWalrusAction("import"); setWalrusError(null); setWalrusSuccess(null); }}
                  className={`flex-1 rounded-md border px-4 py-2 text-sm transition-colors ${
                    walrusAction === "import"
                      ? "border-[#2D9499] bg-[#1B5B682E] text-[#2D9499] font-bold"
                      : "border-[#29466D] bg-[#061227] text-[#8F98AA] hover:text-[#D9D9D9]"
                  }`}
                >
                  Import from Walrus
                </button>
              </div>

              {walrusError && (
                <div className="mb-6 rounded-md border border-[#e96b6b44] bg-[#e96b6b14] px-6 py-4 text-sm text-[#e96b6b]">
                  {walrusError}
                </div>
              )}

              {walrusSuccess && (
                <div className="mb-6 rounded-md border border-[#4BFFA544] bg-[#4BFFA514] px-6 py-4 text-sm text-[#4BFFA5]">
                  {walrusSuccess}
                </div>
              )}

              {walrusAction === "upload" && (
                <div className="rounded-[5px] border border-[#122848] bg-[#061227] p-8">
                  <h3 className="mb-4 text-lg font-bold text-[#D9D9D9]">Upload Registry to Walrus</h3>
                  <p className="mb-6 text-sm text-[#8F98AA]">
                    Upload your local registry database to Walrus decentralized storage. You'll receive a Blob ID that can be shared with teammates to sync registries across machines.
                  </p>
                  <button
                    onClick={handleWalrusUpload}
                    disabled={walrusLoading}
                    className="w-full rounded-md border border-[#2D9499] bg-[#1B5B682E] px-6 py-3 text-base text-[#2D9499] transition-colors hover:bg-[#2D949926] disabled:opacity-40"
                  >
                    {walrusLoading ? "Uploading..." : "Upload Registry"}
                  </button>
                  {lastUploadedBlobId && (
                    <div className="mt-6 rounded border border-[#122848] bg-[#030912] p-4">
                      <p className="mb-2 text-sm font-bold text-[#D9D9D9]">Last Uploaded Blob ID:</p>
                      <div className="flex items-center gap-2">
                        <code className="flex-1 font-mono text-sm text-[#4BFFA5] break-all">{lastUploadedBlobId}</code>
                        <button
                          onClick={() => copyId(lastUploadedBlobId)}
                          className="grid h-8 w-8 place-items-center rounded border-0 bg-transparent p-0 text-[#8F98AA] hover:text-[#4BFFA5]"
                          aria-label="Copy blob ID"
                        >
                          {copied === lastUploadedBlobId ? (
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#4BFFA5" strokeWidth="2">
                              <path d="M20 6 9 17l-5-5" />
                            </svg>
                          ) : (
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                              <rect width="13" height="13" x="9" y="9" rx="2" />
                              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                            </svg>
                          )}
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              )}

              {walrusAction === "import" && (
                <div className="rounded-[5px] border border-[#122848] bg-[#061227] p-8">
                  <h3 className="mb-4 text-lg font-bold text-[#D9D9D9]">Import Registry from Walrus</h3>
                  <p className="mb-6 text-sm text-[#8F98AA]">
                    Import and merge a registry database from Walrus using its Blob ID. This will merge the remote data with your local registry without overwriting existing entries.
                  </p>
                  <form
                    onSubmit={(e) => { e.preventDefault(); handleWalrusImport(); }}
                    className="space-y-4"
                  >
                    <label className="block">
                      <span className="mb-2 block text-sm font-medium text-[#D9D9D9]">Blob ID</span>
                      <input
                        type="text"
                        value={walrusBlobId}
                        onChange={(e) => setWalrusBlobId(e.target.value)}
                        placeholder="Enter Walrus Blob ID"
                        className="w-full rounded-md border border-[#29466D] bg-[#061227] px-4 py-3 font-mono text-sm text-[#D9D9D9] outline-0 placeholder:text-[#8F98AA] focus:border-[#2D9499]"
                      />
                    </label>
                    <button
                      type="submit"
                      disabled={walrusLoading || !walrusBlobId.trim()}
                      className="w-full rounded-md border border-[#2D9499] bg-[#1B5B682E] px-6 py-3 text-base text-[#2D9499] transition-colors hover:bg-[#2D949926] disabled:opacity-40"
                    >
                      {walrusLoading ? "Importing..." : "Import & Merge"}
                    </button>
                  </form>
                </div>
              )}
            </div>
          )}
        </div>
      </section>
    </main>
  );
}
