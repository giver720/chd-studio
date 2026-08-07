export interface Ps3Entry {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  /** update | lang | media | core */
  kind: string;
  lang: string | null;
  suggested: boolean;
  protected: boolean;
  note: string | null;
}

export interface Ps3Scan {
  dir: string;
  total: number;
  valid: boolean;
  title: string | null;
  entries: Ps3Entry[];
}

export interface TrimResult {
  freed: number;
  removed: number;
  skipped: string[];
}

export const KIND_LABEL: Record<string, string> = {
  update: "Actualizador",
  lang: "Idioma",
  media: "Vídeo o audio",
  core: "Del juego",
};

export const KIND_COLOR: Record<string, string> = {
  update: "#f5a524",
  lang: "#22d3ee",
  media: "#a78bfa",
  core: "#8790a6",
};

/** Idiomas presentes en el juego, con cuánto ocupa cada uno. */
export function languageTotals(entries: Ps3Entry[]): { lang: string; size: number; count: number }[] {
  const acc = new Map<string, { size: number; count: number }>();
  for (const e of entries) {
    if (e.kind !== "lang" || !e.lang || e.is_dir) continue;
    const cur = acc.get(e.lang) ?? { size: 0, count: 0 };
    acc.set(e.lang, { size: cur.size + e.size, count: cur.count + 1 });
  }
  return [...acc.entries()]
    .map(([lang, v]) => ({ lang, ...v }))
    .sort((a, b) => b.size - a.size);
}
