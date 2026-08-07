export function bytes(n: number): string {
  if (!n || n < 0) return "—";
  const u = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < u.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v >= 100 || i === 0 ? 0 : 1)} ${u[i]}`;
}

export function duration(ms: number): string {
  if (ms < 1000) return "0s";
  const s = Math.floor(ms / 1000);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const r = s % 60;
  if (h) return `${h}h ${m}m`;
  if (m) return `${m}m ${r}s`;
  return `${r}s`;
}

/** Cuánto se ahorró: 1 GB → 400 MB es un 60 % menos. */
export function savings(input: number, output: number): string | null {
  if (!input || !output) return null;
  const pct = (1 - output / input) * 100;
  if (!isFinite(pct)) return null;
  return `${pct > 0 ? "−" : "+"}${Math.abs(pct).toFixed(0)} %`;
}

export function baseName(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

export function dirName(p: string): string {
  const parts = p.split(/[\\/]/);
  parts.pop();
  return parts.join("\\");
}
