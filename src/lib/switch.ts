/** Conversiones disponibles para archivos de Nintendo Switch. */
export interface SwitchOp {
  id: string;
  label: string;
  from: string;
  to: string;
  tool: string;
  desc: string;
  /** true si el resultado ocupa menos que el original */
  shrinks: boolean;
}

export const SWITCH_OPS: SwitchOp[] = [
  {
    id: "nsp2nsz",
    label: "NSP → NSZ",
    from: "nsp",
    to: "nsz",
    tool: "nsz",
    desc: "Comprime el instalable con zstd. Suele quitar entre un 20 % y un 50 %.",
    shrinks: true,
  },
  {
    id: "nsz2nsp",
    label: "NSZ → NSP",
    from: "nsz",
    to: "nsp",
    tool: "nsz",
    desc: "Devuelve el NSZ a su forma original, byte por byte.",
    shrinks: false,
  },
  {
    id: "xci2xcz",
    label: "XCI → XCZ",
    from: "xci",
    to: "xcz",
    tool: "nsz",
    desc: "Comprime el volcado de cartucho manteniendo su estructura.",
    shrinks: true,
  },
  {
    id: "xcz2xci",
    label: "XCZ → XCI",
    from: "xcz",
    to: "xci",
    tool: "nsz",
    desc: "Reconstruye el cartucho original desde el XCZ.",
    shrinks: false,
  },
  {
    id: "xci2nsp",
    label: "XCI → NSP",
    from: "xci",
    to: "nsp",
    tool: "4nxci",
    desc: "Convierte un volcado de cartucho en un instalable. Puede generar varios NSP.",
    shrinks: false,
  },
];

export const SWITCH_EXT = ["nsp", "nsz", "xci", "xcz"];

/** Operaciones que aceptan un archivo con esa extensión. */
export function opsFor(ext: string): SwitchOp[] {
  return SWITCH_OPS.filter((o) => o.from === ext);
}

export function isCompressed(ext: string): boolean {
  return ext === "nsz" || ext === "xcz";
}

export function defaultOp(ext: string): string {
  const first = SWITCH_OPS.find((o) => o.from === ext);
  return first?.id ?? "nsp2nsz";
}
