# CHD Studio

Aplicación de escritorio para convertir imágenes de disco a **CHD** (Compressed Hunks of Data) usando
`chdman`, la herramienta oficial de MAME. Interfaz en español, arrastrar y soltar, cola por lotes y
progreso en tiempo real.

## Qué cubre

| Familia | Comando de chdman | Sistemas |
|---|---|---|
| CD-ROM (2352 B/sector) | `createcd` | PlayStation, Saturn, Dreamcast (GD-ROM), Mega-CD, PC Engine CD, Neo Geo CD, 3DO, CD-i, PC-FX, Amiga CD32 |
| DVD (2048 B/sector) | `createdvd` | PlayStation 2, PSP (UMD), Xbox, DVD-ROM de PC |
| Disco duro | `createhd` | Arcade con HDD, discos de MS-DOS/Win9x, Xbox HDD |
| Datos crudos | `createraw` | Volcados sin estructura |
| Arcade GD-ROM | `createcd` | NAOMI / NAOMI 2, Triforce, Chihiro, Atomiswave |

También hace el camino inverso (`extractcd`, `extractdvd`, `extracthd`, `extractraw`), verificación
(`verify`) e inspección de cabeceras (`info`).

### Formatos de entrada

- **Sí:** `.cue` + `.bin`, `.gdi`, `.toc`, `.nrg`, `.cdr`, `.iso`, `.img`, `.hdi`, `.vhd`, `.raw`
- **No:** `.cdi`, `.mdf/.mds`, `.ccd`, `.rvz/.wbfs/.nkit`, comprimidos. La app los detecta y avisa en
  vez de fallar en silencio.

## El motor: chdman

CHD Studio es la ventana; el trabajo lo hace `chdman.exe`, la herramienta oficial de MAME.

### Incluirlo en el instalador (recomendado)

Ejecuta una vez:

```bash
npm run chdman
```

Descarga el paquete oficial de binarios de MAME desde GitHub, comprueba su firma SHA256, extrae
únicamente `chdman.exe` (~4 MB) y lo deja en `src-tauri/binaries/`. A partir de ahí, cada
`npm run dist` lo empaqueta dentro del instalador y **el usuario final no tiene que instalar nada**.

Junto al binario se genera `LICENCIA-chdman.txt` con el aviso de GPL-2.0-or-later y el enlace al
código fuente de esa versión exacta, como exige la licencia de MAME. CHD Studio ejecuta chdman como
programa independiente, así que son obras separadas distribuidas juntas por comodidad.

### Sin incluirlo

La app sigue funcionando y busca `chdman` por orden en:

1. La ruta que hayas elegido en Ajustes
2. Su carpeta interna (`%APPDATA%\chd-studio\bin`)
3. La copia empaquetada (`resources/binaries`)
4. El `PATH` del sistema
5. Instalaciones típicas de MAME

Si no aparece, en **Ajustes → Motor chdman** tienes «Importar chdman.exe», que copia el ejecutable
dentro de la app para no depender de dónde lo dejaste.

Con MAME 0.255 o superior se habilita el códec **zstd**, que el preset «Máxima» aprovecha.

## Presets de compresión

| Preset | CD | DVD / HDD |
|---|---|---|
| Máxima | `cdzs, cdlz, cdzl, cdfl` | `zstd, lzma, huff, flac` |
| Equilibrada | `cdlz, cdzl, cdfl` | `lzma, zlib, huff, flac` |
| Rápida | `cdzs, cdfl` | `zstd, huff` |

Sin soporte de zstd se usan los equivalentes clásicos.

## Nintendo Switch

| Conversión | Herramienta | Notas |
|---|---|---|
| NSP → NSZ | `nsz` | Compresión zstd, nivel 1–22 (18 por defecto) |
| NSZ → NSP | `nsz` | Reconstrucción bit a bit |
| XCI → XCZ | `nsz` | Comprime el volcado de cartucho |
| XCZ → XCI | `nsz` | Reconstrucción bit a bit |
| XCI → NSP | `4NXCI` | Cartucho a instalable; puede generar varios NSP |

**Requiere tus propias `prod.keys`** en `%USERPROFILE%\.switch\prod.keys`. CHD Studio no las incluye
ni ayuda a obtenerlas: solo comprueba si el archivo existe y avisa si falta.

`nsz` se instala con pip dentro de un entorno de Python privado de la app (`%APPDATA%\chd-studio\pyenv`),
sin tocar el Python del sistema. `4NXCI` se descarga de su última release de GitHub. Ambas cosas se
hacen desde **Ajustes → Herramientas** con un botón.

## Nintendo 3DS

| Conversión | Herramienta | Claves que exige |
|---|---|---|
| Comprimir a Z3DS | `z3ds_compressor` | ninguna |
| CIA → CCI | `cia-to-cci` | `~/.3ds/aes_keys.txt` |
| CCI/.3ds → CIA | `3dsconv` | `~/.3ds/boot9.bin` |

**Z3DS** es el formato comprimido que [Azahar](https://github.com/azahar-emu/azahar) admite desde la
versión 2123: zstd *seekable*, pensado para descomprimir rápido y poder saltar a cualquier punto sin
extraer el archivo entero. Según la extensión de entrada el resultado es `.zcci`, `.zcia`, `.zcxi` o
`.z3dsx`. Un `.3ds` es el mismo contenedor que un `.cci`, así que se le fuerza la salida `.zcci` para
que el emulador lo reconozca.

La compresión es de ida: no hay descompresor porque el emulador lee el archivo comprimido tal cual.

Las claves son tuyas y salen de tu propia consola. CHD Studio comprueba si están y avisa, pero no las
incluye ni ayuda a obtenerlas.

## Dónde guardas tus claves

No hace falta dejarlas en la carpeta por defecto. En las vistas de Switch y 3DS puedes señalar
**el archivo concreto o la carpeta que lo contiene**, y la ruta queda guardada:

| Archivo | Ruta por defecto | Se le pasa a la herramienta como |
|---|---|---|
| `prod.keys` | `~/.switch/` | `nsz --keys`, `4nxci -k` |
| `aes_keys.txt` | `~/.3ds/` | `cia-to-cci --keys` |
| `boot9.bin` | `~/.3ds/` | `3dsconv --boot9=` |

Las cuatro herramientas aceptan una ruta explícita, así que no se copia ni se mueve nada: se les
indica dónde mirar.

## Xbox 360

| Conversión | Herramienta | Notas |
|---|---|---|
| ISO → GOD | `iso2god` | Games On Demand, el formato de la tienda de la consola |

GOD no es un archivo sino una carpeta con el juego troceado en partes de 1 GB, así que **cabe en
discos formateados en FAT32** (que no admiten archivos de más de 4 GB) y la consola lo reconoce sin
parchear nada.

La opción **«Recortar el espacio vacío»** (`--trim`, activada por defecto) es donde está casi todo el
ahorro: los discos de Xbox 360 llevan una zona de relleno que en muchos juegos son varios GB.

El botón **Analizar** ejecuta `--dry-run` para leer la ficha del juego sin convertir nada, útil para
comprobar que el ISO es válido antes de una conversión larga.

`iso2god` también acepta ISOs de Xbox original.

> **Sobre `.xex`:** no es un formato al que convertir, es el ejecutable que va *dentro* del ISO
> (`default.xex`). Lo que se convierte de verdad es a GOD. Para extraer el contenido de un ISO de
> Xbox 360 no hay una herramienta de línea de comandos mantenida: `extract-xiso` solo documenta
> soporte para Xbox original, así que no se incluye.

## En construcción

- **PlayStation 3** — adelgazar ISOs quitando packs de idioma y `PS3_UPDATE`, con
  `extractps3iso` / `makeps3iso`.
- **Xbox 360** — ISO → GOD (Games on Demand) con `iso2god`, e ISO → carpeta con `extract-xiso`.

## Desarrollo

```bash
npm install
npm run app      # tauri dev
npm run dist     # instalador NSIS en src-tauri/target/release/bundle
```

O ejecuta `CREAR_EXE.bat` para generar el instalador de una tacada.

## Estructura

```
src/                 interfaz React + Tailwind
  lib/profiles.ts    catálogo de sistemas por generación y códecs
  store.ts           estado global (zustand)
src-tauri/src/
  chdman.rs          localización y sondeo del ejecutable
  jobs.rs            cola, ejecución y parseo de progreso
  settings.rs        preferencias persistentes
  lib.rs             comandos expuestos al frontend
```
