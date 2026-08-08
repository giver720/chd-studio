/** Conversiones disponibles para archivos de Nintendo 3DS. */
export interface ThreeDsOp {
  id: string;
  label: string;
  accepts: string[];
  /** Herramientas que hacen falta para esta operación */
  tools: string[];
  desc: string;
  shrinks: boolean;
  /** Qué archivo de claves necesita, si es que necesita alguno */
  needs?: "boot9" | "aes_keys";
}

export const THREEDS_OPS: ThreeDsOp[] = [
  {
    id: "z3dscompress",
    label: "Comprimir a Z3DS",
    accepts: ["cci", "3ds", "cia", "cxi", "3dsx"],
    tools: ["z3ds"],
    desc: "Formato comprimido que Azahar carga directamente, sin descomprimir antes.",
    shrinks: true,
  },
  {
    id: "cia2cci",
    label: "CIA → CCI",
    accepts: ["cia"],
    tools: ["ctrtool", "3dstool", "makerom"],
    desc: "Descifra el CIA y lo remonta como CCI, para cargarlo en Azahar sin instalarlo. Los juegos de eShop necesitan además seeddb.bin.",
    shrinks: false,
    needs: "boot9",
  },
  {
    id: "cci2cia",
    label: "CCI → CIA",
    accepts: ["cci", "3ds"],
    tools: ["3dsconv"],
    desc: "Convierte el volcado de cartucho en un instalable para consola con CFW.",
    shrinks: false,
    needs: "boot9",
  },
];

export const THREEDS_EXT = ["cci", "3ds", "cia", "cxi", "3dsx"];

/** Extensión Z3DS que corresponde a cada formato de entrada. */
export function z3dsExt(ext: string): string {
  if (ext === "cia") return "zcia";
  if (ext === "cxi") return "zcxi";
  if (ext === "3dsx") return "z3dsx";
  return "zcci";
}

export function opsFor(ext: string): ThreeDsOp[] {
  return THREEDS_OPS.filter((o) => o.accepts.includes(ext));
}

export function defaultOp(ext: string): string {
  return ext === "cia" ? "cia2cci" : "z3dscompress";
}
