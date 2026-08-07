import { open } from "@tauri-apps/plugin-dialog";
import { CheckCircle2, FolderSearch, KeyRound, RotateCcw } from "lucide-react";
import { useStore } from "../store";
import type { Settings } from "../lib/types";

/**
 * Selector de un archivo de claves. El usuario puede señalar el archivo o la
 * carpeta que lo contiene; si no elige nada se usa la ruta por defecto de la
 * herramienta.
 */
export function KeyPicker({
  label,
  file,
  found,
  path,
  hint,
  fallback,
  settingKey,
  onChange,
}: {
  label: string;
  /** Nombre del archivo, para los filtros del diálogo */
  file: string;
  found: boolean;
  path: string | null;
  hint: string;
  fallback: string;
  settingKey: keyof Pick<Settings, "switch_keys_path" | "boot9_path" | "aes_keys_path">;
  onChange: () => void;
}) {
  const { settings, patchSettings } = useStore();
  const custom = settings[settingKey];

  async function pickFile() {
    const ext = file.split(".").pop() ?? "*";
    const res = await open({
      multiple: false,
      filters: [{ name: file, extensions: [ext] }],
    });
    if (!res) return;
    await patchSettings({ [settingKey]: res as string } as Partial<Settings>);
    onChange();
  }

  async function pickFolder() {
    const res = await open({ directory: true, multiple: false });
    if (!res) return;
    await patchSettings({ [settingKey]: res as string } as Partial<Settings>);
    onChange();
  }

  async function reset() {
    await patchSettings({ [settingKey]: null } as Partial<Settings>);
    onChange();
  }

  return (
    <div
      className="rounded-xl border p-3"
      style={{
        borderColor: found ? "#34d39926" : "#f5a52426",
        background: found ? "#34d3990a" : "#f5a5240a",
      }}
    >
      <div className="flex items-start gap-2.5">
        {found ? (
          <CheckCircle2 size={16} className="mt-0.5 shrink-0 text-emerald-400" />
        ) : (
          <KeyRound size={16} className="mt-0.5 shrink-0 text-amber-400" />
        )}
        <div className="min-w-0 flex-1">
          <p className="text-[0.78rem] font-medium">{label}</p>
          {found ? (
            <p
              className="selectable mono mt-1 truncate text-[0.66rem] text-[var(--color-muted)]"
              title={path ?? ""}
            >
              {path}
            </p>
          ) : (
            <>
              <p className="mt-1 text-[0.68rem] leading-relaxed text-[var(--color-muted)]">{hint}</p>
              <p className="selectable mono mt-1 truncate text-[0.64rem] text-[var(--color-faint)]">
                Por defecto se busca en {fallback}
              </p>
            </>
          )}
        </div>
      </div>

      <div className="mt-2 flex flex-wrap gap-1.5">
        <button className="btn btn-ghost px-2.5 py-1 text-xs" onClick={pickFile}>
          <FolderSearch size={13} /> Elegir {file}
        </button>
        <button className="btn btn-quiet px-2.5 py-1 text-xs" onClick={pickFolder}>
          Elegir carpeta
        </button>
        {custom && (
          <button className="btn btn-quiet px-2 py-1 text-xs" onClick={reset} title="Volver a la ruta por defecto">
            <RotateCcw size={13} />
          </button>
        )}
      </div>
    </div>
  );
}
