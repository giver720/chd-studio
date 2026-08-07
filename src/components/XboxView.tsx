import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AnimatePresence, motion } from "framer-motion";
import {
  AlertTriangle,
  FolderInput,
  Loader2,
  Package2,
  Play,
  ScanSearch,
  Trash2,
  X,
} from "lucide-react";
import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { bytes } from "../lib/format";
import { useStore } from "../store";
import { DropZone } from "./DropZone";
import { Toggle } from "./ui";

export function XboxView({ dragging }: { dragging: boolean }) {
  const {
    notify,
    refreshJobs,
    settings,
    patchSettings,
    tools,
    refreshTools,
    consoleFiles,
    addConsoleFiles,
    removeConsoleFile,
    clearConsoleFiles,
  } = useStore();
  const files = consoleFiles.xbox360;

  /** Ficha del juego leída con --dry-run, por ruta de archivo. */
  const [titles, setTitles] = useState<Record<string, string>>({});
  const [probing, setProbing] = useState(false);

  // Se re-comprueba al entrar por si acabas de instalar iso2god desde Ajustes
  useEffect(() => {
    refreshTools();
  }, []);

  const iso2god = tools.find((t) => t.id === "iso2god");
  const missing = iso2god && !iso2god.found;

  async function handlePaths(paths: string[]) {
    const infos = await api.inspectPaths(paths);
    const ok = infos.filter((i) => i.ext === "iso");
    if (!ok.length) {
      notify("warn", "Ahí no hay ningún archivo .iso");
      return;
    }
    addConsoleFiles("xbox360", ok);
  }

  /** Lee la ficha de cada ISO sin convertir: sirve para validar antes de empezar. */
  async function probeAll() {
    setProbing(true);
    const found: Record<string, string> = {};
    for (const f of files) {
      try {
        const text = await api.xboxProbe(f.path);
        const line = text
          .split(/\r?\n/)
          .map((l) => l.trim())
          .find((l) => /title|name/i.test(l) && l.includes(":"));
        found[f.path] = line ?? text.split(/\r?\n/).find((l) => l.trim()) ?? "sin datos";
      } catch (e) {
        found[f.path] = `no se pudo leer: ${e}`;
      }
    }
    setTitles((t) => ({ ...t, ...found }));
    setProbing(false);
  }

  async function run() {
    if (!files.length) return;
    await api.addJobs(files.map((f) => ({ input: f.path, mode: "iso2god", system: "xbox360" })));
    clearConsoleFiles("xbox360");
    await refreshJobs();
    notify("ok", `${files.length} ${files.length === 1 ? "conversión" : "conversiones"} en cola`);
  }

  return (
    <div className="scroll flex-1 p-5">
      <div className="mb-4 flex items-end justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">
            Xbox <span className="accent-text">360</span>
          </h1>
          <p className="mt-0.5 text-xs text-[var(--color-muted)]">
            Convierte ISOs al formato GOD, el mismo que usa la consola para los juegos descargados.
          </p>
        </div>
        <div className="flex gap-2">
          {files.length > 0 && (
            <button className="btn btn-ghost" onClick={probeAll} disabled={probing || !!missing}>
              {probing ? <Loader2 size={15} className="animate-spin" /> : <ScanSearch size={15} />}
              Analizar
            </button>
          )}
          {files.length > 0 && (
            <button className="btn btn-primary" onClick={run} disabled={!!missing}>
              <Play size={15} /> Convertir {files.length}
            </button>
          )}
        </div>
      </div>

      {missing && (
        <div className="mb-4 flex items-start gap-3 rounded-xl border border-amber-400/25 bg-amber-400/[0.07] p-3.5">
          <AlertTriangle size={17} className="mt-0.5 shrink-0 text-amber-400" />
          <div className="min-w-0 flex-1">
            <p className="text-[0.8rem] font-medium">Falta iso2god</p>
            <p className="mt-1 text-[0.7rem] leading-relaxed text-[var(--color-muted)]">
              Es un único ejecutable de 1 MB. CHD Studio puede descargarlo por ti.
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

      <DropZone
        dragging={dragging}
        compact={files.length > 0}
        title="Suelta aquí tus ISO de Xbox 360"
        hint="También acepta ISOs de Xbox original. Cada juego sale en su propia carpeta, listo para copiar al disco."
        extensions={["iso"]}
        onPaths={handlePaths}
      />

      <div className="glass mt-4 rounded-2xl p-3">
        <Toggle
          checked={settings.xbox_trim}
          onChange={(v) => patchSettings({ xbox_trim: v })}
          label="Recortar el espacio vacío"
          hint="Los discos de Xbox 360 llevan una zona de relleno enorme que no hace falta. Aquí es donde está casi todo el ahorro."
        />
        <div className="mt-2 flex items-center gap-3 px-3">
          <label className="text-[0.72rem] font-medium text-[var(--color-muted)]">
            Carpeta de salida
          </label>
          <button
            className="btn btn-ghost ml-auto max-w-[60%] justify-start"
            onClick={async () => {
              const res = await open({ directory: true, multiple: false });
              if (res) await patchSettings({ output_dir: res as string });
            }}
          >
            <FolderInput size={15} className="shrink-0" />
            <span className="truncate">{settings.output_dir || "Junto al archivo original"}</span>
          </button>
        </div>
      </div>

      {files.length > 0 ? (
        <>
          <div className="mb-2 mt-5 flex items-center justify-between">
            <h2 className="text-[0.8rem] font-semibold">
              {files.length} {files.length === 1 ? "imagen" : "imágenes"}
            </h2>
            <button
              className="btn btn-quiet btn-danger px-2 py-1 text-xs"
              onClick={() => clearConsoleFiles("xbox360")}
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
                    <span className="block truncate text-[0.68rem] text-[var(--color-faint)]">
                      {bytes(f.size)}
                      {titles[f.path] && ` · ${titles[f.path]}`}
                    </span>
                  </span>
                  <button
                    onClick={() => removeConsoleFile("xbox360", f.path)}
                    className="btn btn-quiet btn-danger px-2 py-1 opacity-0 group-hover:opacity-100"
                  >
                    <X size={14} />
                  </button>
                </motion.li>
              ))}
            </AnimatePresence>
          </ul>
        </>
      ) : (
        <div className="mt-6">
          <h2 className="mb-2 text-[0.8rem] font-semibold">Qué es GOD</h2>
          <div className="grid grid-cols-3 gap-2">
            {[
              {
                t: "Formato nativo",
                d: "Games On Demand es como la consola guarda los juegos de la tienda, así que no hace falta parchear nada.",
              },
              {
                t: "Troceado en 1 GB",
                d: "Al partirse en trozos cabe en discos formateados en FAT32, que no admiten archivos de más de 4 GB.",
              },
              {
                t: "Sin relleno",
                d: "Con «Recortar» se tira la zona vacía del disco original, que en muchos juegos son varios GB.",
              },
            ].map((c) => (
              <div key={c.t} className="glass rounded-xl p-3">
                <p className="text-[0.78rem] font-semibold">{c.t}</p>
                <p className="mt-1 text-[0.68rem] leading-snug text-[var(--color-muted)]">{c.d}</p>
              </div>
            ))}
          </div>
          <p className="mt-3 text-[0.68rem] leading-relaxed text-[var(--color-faint)]">
            Lo hace{" "}
            <button
              className="underline"
              onClick={() => openUrl("https://github.com/iliazeus/iso2god-rs")}
            >
              iso2god
            </button>
            . Ojo con una confusión habitual: <span className="mono">.xex</span> no es un formato al
            que convertir, es el ejecutable que va dentro del ISO. Lo que se convierte de verdad es a
            GOD.
          </p>
        </div>
      )}
    </div>
  );
}
