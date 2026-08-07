import { open } from "@tauri-apps/plugin-dialog";
import { motion } from "framer-motion";
import { FileSearch, Loader2, ScanLine, ShieldCheck } from "lucide-react";
import { useState } from "react";
import { api } from "../lib/api";
import { baseName } from "../lib/format";
import { useStore } from "../store";
import { Empty } from "./ui";

/** chdman info devuelve pares "Etiqueta: valor"; los traducimos a algo legible. */
const LABELS: Record<string, string> = {
  "Input file": "Archivo",
  "File Version": "Versión de CHD",
  "Logical size": "Tamaño original",
  "Hunk Size": "Tamaño de hunk",
  "Total Hunks": "Hunks totales",
  "Unit Size": "Tamaño de unidad",
  "Compression": "Compresión",
  "CHD size": "Tamaño del CHD",
  "Ratio": "Ratio",
  "SHA1": "SHA1",
  "Data SHA1": "SHA1 de datos",
  "Parent SHA1": "SHA1 del padre",
};

function parseInfo(text: string) {
  const pairs: { key: string; value: string }[] = [];
  const metadata: string[] = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;
    if (line.startsWith("Metadata:")) {
      metadata.push(line.replace(/^Metadata:\s*/, ""));
      continue;
    }
    const i = line.indexOf(":");
    if (i > 0 && i < 24) {
      const key = line.slice(0, i).trim();
      const value = line.slice(i + 1).trim();
      if (value) pairs.push({ key: LABELS[key] ?? key, value });
    }
  }
  return { pairs, metadata };
}

export function InspectView() {
  const { notify, refreshJobs } = useStore();
  const [path, setPath] = useState<string | null>(null);
  const [raw, setRaw] = useState("");
  const [busy, setBusy] = useState(false);

  async function pick() {
    const res = await open({ multiple: false, filters: [{ name: "CHD", extensions: ["chd"] }] });
    if (!res) return;
    const p = res as string;
    setPath(p);
    setBusy(true);
    setRaw("");
    try {
      setRaw(await api.chdInfo(p));
    } catch (e) {
      notify("error", String(e));
    } finally {
      setBusy(false);
    }
  }

  async function verify() {
    if (!path) return;
    await api.addJobs([{ input: path, mode: "verify", system: "auto" }]);
    await refreshJobs();
    notify("ok", "Verificación en cola · míralo en la pestaña Convertir");
  }

  const { pairs, metadata } = parseInfo(raw);

  return (
    <div className="scroll flex-1 p-5">
      <div className="mb-4 flex items-end justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">
            Inspeccionar un <span className="accent-text">CHD</span>
          </h1>
          <p className="mt-0.5 text-xs text-[var(--color-muted)]">
            Mira qué lleva dentro, con qué códecs se comprimió y comprueba que no está dañado.
          </p>
        </div>
        <div className="flex gap-2">
          <button className="btn btn-ghost" onClick={pick}>
            <FileSearch size={15} /> Abrir CHD
          </button>
          {path && (
            <button className="btn btn-primary" onClick={verify}>
              <ShieldCheck size={15} /> Verificar
            </button>
          )}
        </div>
      </div>

      {!path && (
        <Empty
          icon={<ScanLine size={22} />}
          title="Ningún archivo abierto"
          desc="Abre un .chd para ver su versión, el tamaño de los hunks, los códecs usados y sus firmas SHA1."
          action={
            <button className="btn btn-ghost" onClick={pick}>
              <FileSearch size={15} /> Elegir archivo
            </button>
          }
        />
      )}

      {path && (
        <>
          <p className="mb-3 truncate text-[0.78rem] font-medium" title={path}>
            {baseName(path)}
          </p>

          {busy ? (
            <div className="flex items-center gap-2 py-10 text-sm text-[var(--color-muted)]">
              <Loader2 size={16} className="animate-spin" /> Leyendo la cabecera…
            </div>
          ) : (
            <>
              <div className="grid grid-cols-2 gap-2">
                {pairs.map((p, i) => (
                  <motion.div
                    key={`${p.key}-${i}`}
                    initial={{ opacity: 0, y: 6 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: i * 0.02 }}
                    className="glass rounded-xl px-3 py-2.5"
                  >
                    <p className="text-[0.66rem] uppercase tracking-wide text-[var(--color-faint)]">
                      {p.key}
                    </p>
                    <p className="selectable mono mt-0.5 truncate text-[0.78rem]" title={p.value}>
                      {p.value}
                    </p>
                  </motion.div>
                ))}
              </div>

              {metadata.length > 0 && (
                <div className="glass mt-4 rounded-xl p-3">
                  <p className="mb-2 text-[0.72rem] font-semibold">Metadatos de pistas</p>
                  <ul className="selectable mono flex flex-col gap-1 text-[0.68rem] leading-relaxed text-[var(--color-muted)]">
                    {metadata.map((m, i) => (
                      <li key={i} className="truncate" title={m}>
                        {m}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {!pairs.length && raw && (
                <pre className="selectable mono glass mt-2 max-h-80 overflow-auto rounded-xl p-3 text-[0.68rem] leading-relaxed whitespace-pre-wrap">
                  {raw}
                </pre>
              )}
            </>
          )}
        </>
      )}
    </div>
  );
}
