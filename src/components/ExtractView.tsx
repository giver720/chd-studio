import { open } from "@tauri-apps/plugin-dialog";
import { AnimatePresence, motion } from "framer-motion";
import { FileDown, FolderInput, Trash2, X } from "lucide-react";
import { useState } from "react";
import { api } from "../lib/api";
import { bytes } from "../lib/format";
import { useStore } from "../store";
import type { InputInfo } from "../lib/types";
import { DropZone } from "./DropZone";

/** Cada destino es una pareja modo de chdman + extensión resultante. */
const TARGETS = [
  { id: "cue", mode: "extractcd", label: "CUE + BIN", desc: "CD estándar: PS1, Saturn, Sega CD…" },
  { id: "gdi", mode: "extractcd", label: "GDI + BIN", desc: "GD-ROM de Dreamcast y NAOMI" },
  { id: "toc", mode: "extractcd", label: "TOC + BIN", desc: "Tabla de contenidos estilo cdrdao" },
  { id: "iso", mode: "extractdvd", label: "ISO", desc: "DVD: PS2, PSP, Xbox" },
  { id: "img", mode: "extracthd", label: "IMG", desc: "Imagen de disco duro" },
  { id: "bin", mode: "extractraw", label: "BIN crudo", desc: "Volcado sin estructura" },
] as const;

type TargetId = (typeof TARGETS)[number]["id"];

export function ExtractView({ dragging }: { dragging: boolean }) {
  const { notify, refreshJobs, settings, patchSettings } = useStore();
  const [files, setFiles] = useState<InputInfo[]>([]);
  const [target, setTarget] = useState<TargetId>("cue");

  async function handlePaths(paths: string[]) {
    const infos = await api.inspectPaths(paths);
    const chds = infos.filter((i) => i.ext === "chd");
    if (!chds.length) {
      notify("warn", "Ahí no hay ningún archivo .chd");
      return;
    }
    setFiles((prev) => {
      const seen = new Set(prev.map((p) => p.path));
      return [...prev, ...chds.filter((c) => !seen.has(c.path))];
    });
  }

  async function pickOutput() {
    const res = await open({ directory: true, multiple: false });
    if (res) await patchSettings({ output_dir: res as string });
  }

  async function extract() {
    if (!files.length) return;
    const t = TARGETS.find((x) => x.id === target)!;
    await api.addJobs(
      files.map((f) => ({
        input: f.path,
        mode: t.mode,
        system: "auto",
        format: t.mode === "extractcd" ? t.id : null,
      })),
    );
    setFiles([]);
    await refreshJobs();
    notify("ok", `${files.length} ${files.length === 1 ? "extracción" : "extracciones"} en cola`);
  }

  return (
    <div className="scroll flex-1 p-5">
      <div className="mb-4 flex items-end justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">
            Extraer de un <span className="accent-text">CHD</span>
          </h1>
          <p className="mt-0.5 text-xs text-[var(--color-muted)]">
            Devuelve el CHD a su formato original. La reconstrucción es exacta: CHD no pierde datos.
          </p>
        </div>
        {files.length > 0 && (
          <button className="btn btn-primary" onClick={extract}>
            <FileDown size={15} /> Extraer {files.length}
          </button>
        )}
      </div>

      <DropZone
        dragging={dragging}
        compact={files.length > 0}
        title="Suelta aquí tus archivos .chd"
        hint="Si no sabes qué había dentro, míralo antes en la pestaña Inspeccionar: te dice si era un CD, un DVD o un disco duro."
        extensions={["chd"]}
        onPaths={handlePaths}
      />

      <div className="mt-5">
        <h2 className="mb-2 text-[0.8rem] font-semibold">Formato de destino</h2>
        <div className="grid grid-cols-3 gap-2">
          {TARGETS.map((t) => {
            const on = target === t.id;
            return (
              <button
                key={t.id}
                onClick={() => setTarget(t.id)}
                className="relative rounded-xl border p-3 text-left transition-colors"
                style={{
                  borderColor: on ? "color-mix(in srgb, var(--accent) 55%, transparent)" : "var(--color-edge)",
                  background: on ? "var(--accent-soft)" : "rgba(255,255,255,0.03)",
                }}
              >
                <span className="block text-[0.8rem] font-semibold">{t.label}</span>
                <span className="mt-0.5 block text-[0.66rem] leading-snug text-[var(--color-muted)]">
                  {t.desc}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      <div className="glass mt-4 flex items-center gap-3 rounded-2xl p-3">
        <label className="text-[0.72rem] font-medium text-[var(--color-muted)]">Carpeta de salida</label>
        <button className="btn btn-ghost ml-auto max-w-[60%] justify-start" onClick={pickOutput}>
          <FolderInput size={15} className="shrink-0" />
          <span className="truncate">{settings.output_dir || "Junto al archivo original"}</span>
        </button>
      </div>

      {files.length > 0 && (
        <>
          <div className="mb-2 mt-5 flex items-center justify-between">
            <h2 className="text-[0.8rem] font-semibold">
              {files.length} {files.length === 1 ? "archivo" : "archivos"}
            </h2>
            <button
              className="btn btn-quiet btn-danger px-2 py-1 text-xs"
              onClick={() => setFiles([])}
            >
              <Trash2 size={13} /> Vaciar
            </button>
          </div>
          <ul className="flex flex-col gap-1.5">
            <AnimatePresence initial={false}>
              {files.map((f) => (
                <motion.li
                  key={f.path}
                  layout
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, x: -12, transition: { duration: 0.14 } }}
                  className="glass group flex items-center gap-3 rounded-xl px-3 py-2.5"
                >
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-[0.82rem] font-medium">{f.name}</span>
                    <span className="text-[0.68rem] text-[var(--color-faint)]">{bytes(f.size)}</span>
                  </span>
                  <button
                    onClick={() => setFiles((p) => p.filter((x) => x.path !== f.path))}
                    className="btn btn-quiet btn-danger px-2 py-1 opacity-0 group-hover:opacity-100"
                  >
                    <X size={14} />
                  </button>
                </motion.li>
              ))}
            </AnimatePresence>
          </ul>
        </>
      )}
    </div>
  );
}
