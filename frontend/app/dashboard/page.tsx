"use client";

import { useEffect, useMemo, useState } from "react";
import {
  fetchObjects,
  fetchTransactions,
  fetchErrors,
  type TrackedObject,
} from "@/lib/api";

const navItems = [
  ["database", "Object Registry"],
  ["pulse", "Tx Timeline"],
  ["alert", "Error Log"],
  ["graph", "Object Graph"],
  ["search", "Inspect Object"],
  ["cloud", "Walrus Sync"],
];

// --- helpers ---------------------------------------------------------------

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
    <div className="ml-0 mb-7 inline-flex flex-col items-start text-base font-extrabold tracking-normal text-[#D9D9D9] min-[761px]:ml-2.5 min-[761px]:mb-0">
      <img
        src="/suiscope-logo.svg"
        alt="SuiScope"
        className="mb-[-16px] h-20 w-[92px] object-contain object-left"
      />
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
  const [txCount, setTxCount] = useState(0);
  const [errorCount, setErrorCount] = useState(0);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [query, setQuery] = useState("");
  const [network, setNetwork] = useState("all");
  const [copied, setCopied] = useState<string | null>(null);

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
        setTxCount(txs.length);
        setErrorCount(errs.length);
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

  return (
    <main className="min-h-screen bg-[#030912]">
      <aside className="top-0 flex min-h-0 flex-col border-r border-[#D9D9D933] bg-[#061227] p-6 min-[761px]:fixed min-[761px]:left-0 min-[761px]:top-0 min-[761px]:h-screen min-[761px]:w-[400px] min-[761px]:px-[22px] min-[761px]:pb-14 min-[761px]:pt-10 max-[1050px]:min-[761px]:w-[280px]">
        <div className="-mx-6 border-b border-[#D9D9D933] px-6 min-[761px]:-mx-[22px] min-[761px]:px-[22px]">
          <Logo />
          <div className="-mt-[42px] mb-[94px] ml-[124px] flex gap-[26px] text-xl font-extrabold text-[#D9D9D9] max-[1050px]:ml-[94px] max-[1050px]:gap-4 max-[1050px]:text-[17px] max-[760px]:hidden">
            <span>DEBUG</span>
            <span>REGISTRY</span>
          </div>
        </div>

        <section className="-mx-6 mb-6 border-b border-[#D9D9D933] px-6 py-6 min-[761px]:-mx-[22px] min-[761px]:mb-0 min-[761px]:px-[22px]">
          <h2 className="mb-8 mt-0 text-[22px] font-medium text-[#D9D9D9]">PROJECT</h2>
          <p className="m-0 flex items-center gap-2 text-xl font-extrabold text-[#D9D9D9]">
            <span className="h-2.5 w-2.5 rounded-full bg-[#4BFFA5]" />
            Local Registry
          </p>
        </section>

        <section className="mb-6 flex-1 py-6 min-[761px]:mb-0">
          <h2 className="mb-8 mt-0 text-[22px] font-medium text-[#D9D9D9]">NAVIGATION</h2>
          <nav
            aria-label="Primary navigation"
            className="grid gap-[18px] min-[761px]:gap-7 max-[760px]:grid-cols-2"
          >
            {navItems.map(([icon, label]) => (
              <a
                href="#"
                className={`flex min-h-7 items-center gap-[18px] text-[15px] text-[#D9D9D9] transition-colors hover:text-white min-[761px]:text-xl ${
                  label === "Object Registry" ? "text-white" : ""
                }`}
                key={label}
              >
                <Icon name={icon} />
                <span>{label}</span>
              </a>
            ))}
          </nav>
        </section>

        <div className="-mx-[22px] grid justify-items-center gap-[18px] border-t border-[#D9D9D933] px-[22px] pt-4 text-sm text-[#8F98AA] max-[760px]:hidden">
          <code className="text-sm text-[#4BFFA5]">suiscope publish</code>
          <span>Auto-registers all spawned</span>
        </div>
      </aside>

      <section className="min-w-0 bg-[#030912] min-[761px]:ml-[400px] max-[1050px]:min-[761px]:ml-[280px]">
        <header className="border-b border-[#D9D9D933] bg-[#061227] px-[18px] py-5 min-[761px]:min-h-[123px] min-[761px]:px-[42px] min-[761px]:pb-3.5">
          <div className="mb-[26px] flex flex-wrap gap-[15px] text-sm text-[#D9D9D9] [&_svg]:h-4 [&_svg]:w-4">
            <StatItem icon="cube" label="Packages" value={packageCount} />
            <StatItem icon="cube" label="Objects" value={objects.length} />
            <StatItem icon="pulse" label="Transactions" value={txCount} />
            <StatItem icon="alert" label="Errors" value={errorCount} />
          </div>
          <h1 className="mb-1.5 mt-0 text-xl font-extrabold text-[#D9D9D9]">
            Object Registry
          </h1>
          <p className="m-0 text-base text-[#8F98AA]">
            All tracked Package IDs and Object IDs for this project
          </p>
        </header>

        <div className="px-[18px] pt-[22px] min-[761px]:pl-6 min-[761px]:pr-[26px] min-[1051px]:pl-[53px]">
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
        </div>
      </section>
    </main>
  );
}
