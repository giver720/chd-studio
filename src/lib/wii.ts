export interface WiiOp {
  id: string;
  label: string;
  accepts: string[];
  to: string;
  desc: string;
  /** true si el resultado ocupa menos que el original */
  shrinks: boolean;
  recommended?: boolean;
  /** Esta la hace `wit`, no DolphinTool */
  needsWit?: boolean;
}

export const WII_OPS: WiiOp[] = [
  {
    id: "iso2rvz",
    label: "ISO → RVZ",
    accepts: ["iso", "gcm"],
    to: "rvz",
    desc: "El formato propio de Dolphin. Con el relleno quitado, un juego de 4,7 GB puede quedarse en 1-2 GB.",
    shrinks: true,
    recommended: true,
  },
  {
    id: "iso2wia",
    label: "ISO → WIA",
    accepts: ["iso", "gcm"],
    to: "wia",
    desc: "El antecesor de RVZ. Úsalo solo si tu emulador es viejo y no lee RVZ.",
    shrinks: true,
  },
  {
    id: "iso2gcz",
    label: "ISO → GCZ",
    accepts: ["iso", "gcm"],
    to: "gcz",
    desc: "Formato antiguo, sobre todo de GameCube. Comprime bastante menos que RVZ.",
    shrinks: true,
  },
  {
    id: "iso2wbfs",
    label: "ISO → WBFS",
    accepts: ["iso", "gcm"],
    to: "wbfs",
    desc: "Para jugar en una Wii de verdad con un cargador USB. No es para Dolphin.",
    shrinks: true,
    needsWit: true,
  },
  {
    id: "rvz2iso",
    label: "Volver a ISO",
    accepts: ["rvz", "wia", "gcz", "wbfs", "ciso"],
    to: "iso",
    desc: "Reconstruye el ISO. Ojo: si lo comprimiste con el relleno quitado, no vuelve idéntico al original.",
    shrinks: false,
  },
];

export const WII_EXT = ["iso", "gcm", "rvz", "wia", "gcz", "wbfs", "ciso"];

export function opsFor(ext: string): WiiOp[] {
  return WII_OPS.filter((o) => o.accepts.includes(ext));
}

export function defaultOp(ext: string): string {
  return ext === "iso" || ext === "gcm" ? "iso2rvz" : "rvz2iso";
}
