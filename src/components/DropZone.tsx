import { open } from "@tauri-apps/plugin-dialog";
import { motion } from "framer-motion";
import { FolderOpen, UploadCloud } from "lucide-react";

export function DropZone({
  dragging,
  compact,
  title,
  hint,
  extensions,
  onPaths,
}: {
  dragging: boolean;
  compact?: boolean;
  title: string;
  hint: string;
  extensions: string[];
  onPaths: (paths: string[]) => void;
}) {
  async function pickFiles() {
    const res = await open({
      multiple: true,
      filters: [{ name: "Imágenes de disco", extensions }],
    });
    if (res) onPaths(Array.isArray(res) ? res : [res]);
  }

  async function pickFolder() {
    const res = await open({ directory: true, multiple: false });
    if (res) onPaths([res as string]);
  }

  return (
    <motion.div
      animate={{
        borderColor: dragging ? "var(--accent)" : "#ffffff1a",
        backgroundColor: dragging ? "var(--accent-soft)" : "rgba(255,255,255,0.025)",
      }}
      className="rounded-2xl border border-dashed p-5 text-center"
    >
      <motion.div
        animate={{ scale: dragging ? 1.06 : 1, y: dragging ? -2 : 0 }}
        transition={{ type: "spring", stiffness: 320, damping: 20 }}
        className="flex flex-col items-center gap-3"
      >
        {!compact && (
          <div
            className="grid place-items-center rounded-2xl border border-[var(--color-edge)] bg-white/[0.05]"
            style={{ width: 56, height: 56, color: dragging ? "var(--accent)" : "var(--color-faint)" }}
          >
            <UploadCloud size={26} />
          </div>
        )}
        <div>
          <p className="text-sm font-semibold">{dragging ? "Suelta aquí" : title}</p>
          <p className="mx-auto mt-1 max-w-md text-xs leading-relaxed text-[var(--color-muted)]">{hint}</p>
        </div>
        <div className="flex gap-2">
          <button className="btn btn-ghost" onClick={pickFiles}>
            <UploadCloud size={15} /> Elegir archivos
          </button>
          <button className="btn btn-quiet" onClick={pickFolder}>
            <FolderOpen size={15} /> Carpeta entera
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
}
