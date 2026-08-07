import { motion } from "framer-motion";
import type { ReactNode } from "react";

export function Segmented<T extends string>({
  value,
  onChange,
  options,
  layoutId,
}: {
  value: T;
  onChange: (v: T) => void;
  options: { id: T; label: string; hint?: string }[];
  layoutId: string;
}) {
  return (
    <div className="flex gap-1 rounded-xl border border-[var(--color-edge)] bg-white/[0.04] p-1">
      {options.map((o) => (
        <button
          key={o.id}
          onClick={() => onChange(o.id)}
          title={o.hint}
          className="relative flex-1 rounded-lg px-3 py-1.5 text-[0.8rem] font-medium transition-colors"
          style={{ color: value === o.id ? "#0a0c12" : "var(--color-muted)" }}
        >
          {value === o.id && (
            <motion.span
              layoutId={layoutId}
              className="absolute inset-0 rounded-lg"
              style={{ background: "linear-gradient(100deg, var(--accent), var(--accent-2))" }}
              transition={{ type: "spring", stiffness: 420, damping: 34 }}
            />
          )}
          <span className="relative z-10">{o.label}</span>
        </button>
      ))}
    </div>
  );
}

export function Toggle({
  checked,
  onChange,
  label,
  hint,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
  hint?: string;
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className="flex w-full items-start gap-3 rounded-xl px-3 py-2.5 text-left transition-colors hover:bg-white/[0.04]"
    >
      <span
        className="mt-0.5 flex h-[22px] w-[38px] shrink-0 items-center rounded-full p-[3px] transition-colors"
        style={{
          background: checked
            ? "linear-gradient(100deg, var(--accent), var(--accent-2))"
            : "#ffffff1a",
        }}
      >
        <motion.span
          layout
          transition={{ type: "spring", stiffness: 600, damping: 36 }}
          className="h-4 w-4 rounded-full bg-white shadow"
          style={{ marginLeft: checked ? 16 : 0 }}
        />
      </span>
      <span className="min-w-0">
        <span className="block text-sm font-medium">{label}</span>
        {hint && <span className="block text-xs leading-snug text-[var(--color-muted)]">{hint}</span>}
      </span>
    </button>
  );
}

export function Card({
  title,
  desc,
  children,
  right,
}: {
  title?: string;
  desc?: string;
  children: ReactNode;
  right?: ReactNode;
}) {
  return (
    <section className="glass rounded-2xl p-4">
      {(title || right) && (
        <header className="mb-3 flex items-start justify-between gap-3">
          <div>
            {title && <h3 className="text-sm font-semibold">{title}</h3>}
            {desc && <p className="mt-0.5 text-xs text-[var(--color-muted)]">{desc}</p>}
          </div>
          {right}
        </header>
      )}
      {children}
    </section>
  );
}

export function Empty({
  icon,
  title,
  desc,
  action,
}: {
  icon: ReactNode;
  title: string;
  desc: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <div className="grid h-14 w-14 place-items-center rounded-2xl border border-[var(--color-edge)] bg-white/[0.04] text-[var(--color-muted)]">
        {icon}
      </div>
      <div>
        <p className="text-sm font-semibold">{title}</p>
        <p className="mx-auto mt-1 max-w-sm text-xs leading-relaxed text-[var(--color-muted)]">{desc}</p>
      </div>
      {action}
    </div>
  );
}
