import { open } from "@tauri-apps/plugin-dialog";
import { AnimatePresence, motion } from "framer-motion";
import {
  AlertTriangle,
  ChevronRight,
  Disc,
  FolderOpen,
  Loader2,
  Lock,
  Package2,
  Scissors,
  ShieldAlert,
  Sparkles,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api } from "../lib/api";
import { bytes } from "../lib/format";
import { KIND_COLOR, KIND_LABEL, languageTotals, type Ps3Scan } from "../lib/ps3";
import { useStore } from "../store";
import { Toggle } from "./ui";

export function Ps3View() {
  const { notify, refreshJobs, settings, patchSettings, tools, refreshTools } = useStore();
  const [scan, setScan] = useState<Ps3Scan | null>(null);
  const [busy, setBusy] = useState(false);
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [confirming, setConfirming] = useState(false);

  useEffect(() => {
    refreshTools();
  }, []);

  const ps3iso = tools.find((t) => t.id === "ps3iso");
  const missing = ps3iso && !ps3iso.found;

  const langs = useMemo(() => (scan ? languageTotals(scan.entries) : []), [scan]);
  const selectedSize = useMemo(
    () => (scan ? scan.entries.filter((e) => picked.has(e.path)).reduce((a, e) => a + e.size, 0) : 0),
    [scan, picked],
  );

  /** Un ISO no se puede inspeccionar: primero hay que extraerlo a una carpeta. */
  async function pickIso() {
    const res = await open({ multiple: false, filters: [{ name: "ISO de PS3", extensions: ["iso"] }] });
    if (!res) return;
    await api.addJobs([{ input: res as string, mode: "ps3extract", system: "ps3" }]);
    await refreshJobs();
    notify(
      "ok",
      "Extrayendo. Cuando termine, abre la carpeta resultante aquí con «Abrir carpeta».",
    );
  }

  async function pickFolder() {
    const res = await open({ directory: true, multiple: false });
    if (!res) return;
    setBusy(true);
    setPicked(new Set());
    try {
      const s = await api.ps3Scan(res as string);
      setScan(s);
      if (!s.valid) {
        notify("warn", "Esa carpeta no parece un juego de PS3 extraído");
      } else {
        setPicked(new Set(s.entries.filter((e) => e.suggested).map((e) => e.path)));
      }
    } catch (e) {
      notify("error", String(e));
    } finally {
      setBusy(false);
    }
  }

  function toggle(path: string) {
    setPicked((p) => {
      const n = new Set(p);
      n.has(path) ? n.delete(path) : n.add(path);
      return n;
    });
  }

  /** Marca de golpe todo lo de un idioma. */
  function toggleLang(lang: string) {
    if (!scan) return;
    const paths = scan.entries.filter((e) => e.lang === lang && !e.protected).map((e) => e.path);
    const allOn = paths.every((p) => picked.has(p));
    setPicked((p) => {
      const n = new Set(p);
      paths.forEach((x) => (allOn ? n.delete(x) : n.add(x)));
      return n;
    });
  }

  async function applyTrim() {
    if (!scan || !picked.size) return;
    setBusy(true);
    try {
      const r = await api.ps3Trim(scan.dir, [...picked]);
      notify("ok", `${r.removed} elementos borrados · ${bytes(r.freed)} liberados`);
      if (r.skipped.length) notify("warn", `${r.skipped.length} no se pudieron borrar`);
      setScan(await api.ps3Scan(scan.dir));
      setPicked(new Set());
    } catch (e) {
      notify("error", String(e));
    } finally {
      setBusy(false);
      setConfirming(false);
    }
  }

  async function rebuild() {
    if (!scan) return;
    await api.addJobs([{ input: scan.dir, mode: "ps3build", system: "ps3" }]);
    await refreshJobs();
    notify("ok", "Reconstruyendo el ISO");
  }

  return (
    <div className="scroll flex-1 p-5">
      <div className="mb-4 flex items-end justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">
            PlayStation <span className="accent-text">3</span>
          </h1>
          <p className="mt-0.5 text-xs text-[var(--color-muted)]">
            Quita packs de idioma y el actualizador del disco para que el juego ocupe bastante menos.
          </p>
        </div>
        <div className="flex gap-2">
          <button className="btn btn-ghost" onClick={pickIso} disabled={!!missing}>
            <Disc size={15} /> Extraer un ISO
          </button>
          <button className="btn btn-primary" onClick={pickFolder} disabled={busy}>
            {busy ? <Loader2 size={15} className="animate-spin" /> : <FolderOpen size={15} />}
            Abrir carpeta
          </button>
        </div>
      </div>

      {missing && (
        <div className="mb-4 flex items-start gap-3 rounded-xl border border-amber-400/25 bg-amber-400/[0.07] p-3.5">
          <AlertTriangle size={17} className="mt-0.5 shrink-0 text-amber-400" />
          <div className="min-w-0 flex-1">
            <p className="text-[0.8rem] font-medium">Faltan las ps3iso-utils</p>
            <p className="mt-1 text-[0.7rem] leading-relaxed text-[var(--color-muted)]">
              Hacen falta para extraer y reconstruir ISOs. Analizar una carpeta ya extraída sí
              funciona sin ellas.
            </p>
          </div>
          <button
            className="btn btn-ghost shrink-0"
            onClick={() => useStore.setState({ view: "settings" })}
          >
            <Package2 size={15} /> Instalar
          </button>
        </div>
      )}

      {!scan && (
        <>
          <div className="glass rounded-2xl p-5">
            <h2 className="text-[0.85rem] font-semibold">Cómo funciona</h2>
            <ol className="mt-3 flex flex-col gap-2.5">
              {[
                ["Extrae el ISO a una carpeta", "Un ISO no se puede tocar por dentro. Si ya tienes el juego en formato carpeta, sáltate este paso."],
                ["Abre la carpeta aquí", "CHD Studio mira qué hay dentro y marca lo que se puede quitar sin romper nada."],
                ["Revisa y borra", "Tú decides qué cae. Los archivos que el juego necesita para arrancar están bloqueados."],
                ["Reconstruye el ISO", "Opcional: si juegas en formato carpeta, ya has terminado."],
              ].map(([t, d], i) => (
                <li key={t} className="flex gap-3">
                  <span
                    className="mt-0.5 grid h-5 w-5 shrink-0 place-items-center rounded-full text-[0.65rem] font-bold text-[#0a0c12]"
                    style={{ background: "var(--accent)" }}
                  >
                    {i + 1}
                  </span>
                  <span>
                    <span className="block text-[0.8rem] font-medium">{t}</span>
                    <span className="block text-[0.68rem] leading-snug text-[var(--color-muted)]">{d}</span>
                  </span>
                </li>
              ))}
            </ol>
          </div>

          <div className="mt-3 flex items-start gap-3 rounded-xl border border-amber-400/25 bg-amber-400/[0.07] p-3.5">
            <ShieldAlert size={17} className="mt-0.5 shrink-0 text-amber-400" />
            <p className="text-[0.72rem] leading-relaxed text-[var(--color-muted)]">
              <span className="font-medium text-[var(--color-ink)]">Guarda una copia antes.</span> Hay
              juegos que llevan un índice de sus propios archivos y se cuelgan si falta uno, aunque sea
              un vídeo en un idioma que no usas. La detección por nombre acierta casi siempre, pero no
              es infalible: prueba el juego después de adelgazarlo.
            </p>
          </div>
        </>
      )}

      {scan && (
        <>
          <div className="glass mb-3 flex flex-wrap items-center gap-4 rounded-2xl p-3.5">
            <div className="min-w-0 flex-1">
              <p className="truncate text-[0.85rem] font-semibold">
                {scan.title ?? "Juego sin identificar"}
              </p>
              <p className="selectable mono mt-0.5 truncate text-[0.66rem] text-[var(--color-faint)]">
                {scan.dir}
              </p>
            </div>
            <div className="text-right">
              <p className="text-[0.66rem] text-[var(--color-faint)]">Ocupa ahora</p>
              <p className="text-[0.95rem] font-semibold">{bytes(scan.total)}</p>
            </div>
            {selectedSize > 0 && (
              <div className="text-right">
                <p className="text-[0.66rem] text-[var(--color-faint)]">Quedaría en</p>
                <p className="text-[0.95rem] font-semibold text-emerald-400">
                  {bytes(scan.total - selectedSize)}
                </p>
              </div>
            )}
          </div>

          {langs.length > 0 && (
            <div className="glass mb-3 rounded-2xl p-3.5">
              <p className="mb-2 flex items-center gap-2 text-[0.8rem] font-semibold">
                <Sparkles size={14} style={{ color: "var(--accent)" }} />
                Idiomas detectados
              </p>
              <div className="flex flex-wrap gap-1.5">
                {langs.map((l) => {
                  const paths = scan.entries.filter((e) => e.lang === l.lang && !e.protected);
                  const on = paths.length > 0 && paths.every((e) => picked.has(e.path));
                  return (
                    <button
                      key={l.lang}
                      onClick={() => toggleLang(l.lang)}
                      className="chip transition-colors"
                      style={{
                        borderColor: on ? "#fb718566" : "var(--color-edge)",
                        background: on ? "#fb718522" : "#ffffff08",
                        color: on ? "#fca5a5" : "var(--color-muted)",
                      }}
                    >
                      {l.lang} · {bytes(l.size)}
                    </button>
                  );
                })}
              </div>
              <p className="mt-2 text-[0.66rem] leading-relaxed text-[var(--color-faint)]">
                Toca un idioma para marcarlo entero. Deja siempre al menos el que vayas a jugar.
              </p>
            </div>
          )}

          <div className="mb-2 flex items-center justify-between">
            <h2 className="text-[0.8rem] font-semibold">
              {picked.size > 0
                ? `${picked.size} marcados · ${bytes(selectedSize)}`
                : `${scan.entries.length} elementos`}
            </h2>
            <div className="flex gap-2">
              {picked.size > 0 && (
                <button
                  className="btn btn-ghost px-2.5 py-1 text-xs"
                  onClick={() => setPicked(new Set())}
                >
                  Desmarcar todo
                </button>
              )}
              <button
                className="btn btn-ghost px-2.5 py-1 text-xs"
                onClick={rebuild}
                disabled={!!missing}
              >
                <Disc size={13} /> Reconstruir ISO
              </button>
              <button
                className="btn btn-primary px-2.5 py-1 text-xs"
                onClick={() => setConfirming(true)}
                disabled={!picked.size || busy}
              >
                <Scissors size={13} /> Borrar marcados
              </button>
            </div>
          </div>

          <ul className="flex flex-col gap-1">
            {scan.entries.slice(0, 400).map((e) => {
              const on = picked.has(e.path);
              return (
                <li
                  key={e.path}
                  className="glass flex items-center gap-3 rounded-lg px-3 py-2"
                  style={on ? { borderColor: "#fb718544", background: "#fb718510" } : undefined}
                >
                  <input
                    type="checkbox"
                    checked={on}
                    disabled={e.protected}
                    onChange={() => toggle(e.path)}
                    className="h-3.5 w-3.5 shrink-0 accent-[var(--accent)]"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="flex items-center gap-1.5">
                      {e.is_dir && <ChevronRight size={12} className="shrink-0 text-[var(--color-faint)]" />}
                      <span className="truncate text-[0.78rem]" title={e.path}>
                        {e.path}
                      </span>
                      {e.protected && <Lock size={11} className="shrink-0 text-[var(--color-faint)]" />}
                    </span>
                    {e.note && (
                      <span className="block truncate text-[0.64rem] text-[var(--color-faint)]">
                        {e.note}
                      </span>
                    )}
                  </span>
                  <span
                    className="chip shrink-0"
                    style={{
                      borderColor: `${KIND_COLOR[e.kind]}44`,
                      background: `${KIND_COLOR[e.kind]}14`,
                      color: KIND_COLOR[e.kind],
                    }}
                  >
                    {KIND_LABEL[e.kind]}
                  </span>
                  <span className="mono w-[68px] shrink-0 text-right text-[0.7rem] text-[var(--color-muted)]">
                    {bytes(e.size)}
                  </span>
                </li>
              );
            })}
          </ul>
          {scan.entries.length > 400 && (
            <p className="mt-2 text-center text-[0.68rem] text-[var(--color-faint)]">
              Se muestran los 400 elementos más grandes de {scan.entries.length}.
            </p>
          )}

          <div className="glass mt-3 rounded-2xl p-3">
            <Toggle
              checked={settings.ps3_split_fat32}
              onChange={(v) => patchSettings({ ps3_split_fat32: v })}
              label="Partir en trozos de 4 GB al reconstruir"
              hint="Necesario solo si vas a copiar el ISO a un disco formateado en FAT32, que no admite archivos más grandes."
            />
          </div>
        </>
      )}

      <AnimatePresence>
        {confirming && scan && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-6"
            onClick={() => setConfirming(false)}
          >
            <motion.div
              initial={{ scale: 0.94, y: 12 }}
              animate={{ scale: 1, y: 0 }}
              exit={{ scale: 0.96, y: 8 }}
              onClick={(ev) => ev.stopPropagation()}
              className="glass-strong w-full max-w-md rounded-2xl p-5"
            >
              <h3 className="flex items-center gap-2 text-[0.9rem] font-semibold">
                <ShieldAlert size={17} className="text-amber-400" />
                Se va a borrar de verdad
              </h3>
              <p className="mt-2 text-[0.75rem] leading-relaxed text-[var(--color-muted)]">
                Vas a eliminar <span className="font-medium text-[var(--color-ink)]">{picked.size}</span>{" "}
                elementos y liberar{" "}
                <span className="font-medium text-[var(--color-ink)]">{bytes(selectedSize)}</span> de{" "}
                <span className="mono">{scan.dir.split(/[\\/]/).pop()}</span>.
              </p>
              <p className="mt-2 text-[0.72rem] leading-relaxed text-[var(--color-muted)]">
                Esto no se puede deshacer y no va a la papelera. Si no tienes copia del ISO original,
                cancela y hazla primero.
              </p>
              <div className="mt-4 flex justify-end gap-2">
                <button className="btn btn-ghost" onClick={() => setConfirming(false)}>
                  Cancelar
                </button>
                <button className="btn btn-primary" onClick={applyTrim} disabled={busy}>
                  {busy ? <Loader2 size={15} className="animate-spin" /> : <Scissors size={15} />}
                  Borrar {picked.size}
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
