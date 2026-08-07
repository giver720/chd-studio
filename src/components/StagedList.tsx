import { AnimatePresence, motion } from "framer-motion";
import { Disc3, HardDrive, Layers, X } from "lucide-react";
import { bytes } from "../lib/format";
import { GENERATIONS, systemById } from "../lib/profiles";
import { useStore } from "../store";
import type { StagedFile } from "../lib/types";

function ModeIcon({ mode }: { mode: string }) {
  if (mode === "createhd" || mode === "createraw") return <HardDrive size={16} />;
  if (mode === "createdvd") return <Layers size={16} />;
  return <Disc3 size={16} />;
}

function Row({ file }: { file: StagedFile }) {
  const { setStaged, removeStaged } = useStore();
  const sys = systemById(file.systemId);

  return (
    <motion.li
      layout
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, x: -12, transition: { duration: 0.14 } }}
      transition={{ type: "spring", stiffness: 420, damping: 34 }}
      className="glass group flex items-center gap-3 rounded-xl px-3 py-2.5"
    >
      <span
        className="grid h-9 w-9 shrink-0 place-items-center rounded-lg"
        style={{ background: `${sys.color}1f`, color: sys.color }}
      >
        <ModeIcon mode={file.mode} />
      </span>

      <span className="min-w-0 flex-1">
        <span className="block truncate text-[0.82rem] font-medium" title={file.path}>
          {file.name}
        </span>
        <span className="mt-0.5 flex items-center gap-1.5 text-[0.68rem] text-[var(--color-faint)]">
          <span className="uppercase">{file.ext}</span>
          <span>·</span>
          <span>{bytes(file.size)}</span>
        </span>
      </span>

      <select
        className="field no-drag w-[168px] shrink-0 py-1.5 text-xs"
        value={file.systemId}
        onChange={(e) => setStaged([file.path], { systemId: e.target.value })}
      >
        {GENERATIONS.map((g) => (
          <optgroup key={g.id} label={g.title}>
            {g.systems.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </optgroup>
        ))}
      </select>

      <button
        onClick={() => removeStaged(file.path)}
        className="btn btn-quiet btn-danger shrink-0 px-2 py-1 opacity-0 transition-opacity group-hover:opacity-100"
        aria-label="Quitar"
      >
        <X size={14} />
      </button>
    </motion.li>
  );
}

export function StagedList() {
  const staged = useStore((s) => s.staged);
  return (
    <ul className="flex flex-col gap-1.5">
      <AnimatePresence initial={false}>
        {staged.map((f) => (
          <Row key={f.path} file={f} />
        ))}
      </AnimatePresence>
    </ul>
  );
}
