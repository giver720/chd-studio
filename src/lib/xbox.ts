export interface XboxOp {
  id: string;
  label: string;
  tool: string;
  desc: string;
  /** true si el resultado es una carpeta y no un archivo */
  toFolder: boolean;
}

export const XBOX_OPS: XboxOp[] = [
  {
    id: "iso2god",
    label: "ISO → GOD",
    tool: "iso2god",
    desc: "El formato de la tienda de la consola. Se trocea en 1 GB, así que vale para discos en FAT32.",
    toFolder: true,
  },
  {
    id: "iso2folder",
    label: "ISO → carpeta (XEX)",
    tool: "xiso",
    desc: "Extrae el juego con su default.xex y todos sus archivos, que es lo que piden Aurora y Freestyle Dash. Úsalo cuando el juego no arranca en GOD.",
    toFolder: true,
  },
];

/** Operación que reconstruye un ISO desde una carpeta ya extraída. */
export const XBOX_BUILD: XboxOp = {
  id: "folder2iso",
  label: "Carpeta → ISO",
  tool: "xiso",
  desc: "Vuelve a montar el ISO desde una carpeta extraída.",
  toFolder: false,
};

export function opById(id: string): XboxOp {
  return XBOX_OPS.find((o) => o.id === id) ?? XBOX_BUILD;
}
