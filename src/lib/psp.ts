export interface PspOp {
  id: string;
  label: string;
  accepts: string[];
  to: string;
  desc: string;
  /** Advertencia de la propia herramienta */
  experimental?: boolean;
}

export const PSP_OPS: PspOp[] = [
  {
    id: "iso2cso",
    label: "ISO → CSO",
    accepts: ["iso"],
    to: "cso",
    desc: "El formato de siempre. Lo leen tanto la PSP con CFW como PPSSPP.",
  },
  {
    id: "iso2zso",
    label: "ISO → ZSO",
    accepts: ["iso"],
    to: "zso",
    desc: "Con zstd: descomprime mucho más rápido, así que en consola real carga antes y gasta menos batería.",
    experimental: true,
  },
  {
    id: "iso2dax",
    label: "ISO → DAX",
    accepts: ["iso"],
    to: "dax",
    desc: "Formato antiguo. Solo si tienes un CFW viejo que no lea CSO.",
    experimental: true,
  },
  {
    id: "cso2iso",
    label: "Volver a ISO",
    accepts: ["cso", "zso", "dax"],
    to: "iso",
    desc: "Descomprime al ISO original, idéntico byte a byte.",
  },
];

export const PSP_EXT = ["iso", "cso", "zso", "dax"];

export function opsFor(ext: string): PspOp[] {
  return PSP_OPS.filter((o) => o.accepts.includes(ext));
}

export function defaultOp(ext: string): string {
  return ext === "iso" ? "iso2cso" : "cso2iso";
}

/**
 * maxcso escribe el resultado de --measure en una línea con el tamaño y el
 * porcentaje. Se saca el porcentaje para enseñar el ahorro.
 */
export function parseMeasure(text: string): string | null {
  const pct = [...text.matchAll(/([\d.]+)\s*%/g)].pop();
  if (pct) return `quedaría en el ${pct[1]} %`;
  const line = text
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean)
    .pop();
  return line ?? null;
}
