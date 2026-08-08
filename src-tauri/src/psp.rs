//! Compresion de imagenes de PSP con `maxcso`.
//!
//! Un UMD volcado a ISO se puede guardar comprimido sin perder nada, y tanto la
//! consola con CFW como PPSSPP lo leen tal cual:
//!
//!   * **CSO**  - el de toda la vida, el que entiende todo el mundo.
//!   * **ZSO**  - comprimido con zstd: descomprime mucho mas rapido, que en una
//!                consola de verdad significa menos espera y menos bateria.
//!   * **DAX**  - formato antiguo, solo para CFW viejo.
//!
//! maxcso marca ZSO y DAX como experimentales; CSO es la opcion segura.

use crate::settings::Settings;

/// (modo, extension de salida, formato para --format)
pub const MODES: &[(&str, &str, &str)] = &[
    ("iso2cso", "cso", "cso1"),
    ("iso2zso", "zso", "zso"),
    ("iso2dax", "dax", "dax"),
    ("cso2iso", "iso", ""),
];

pub fn is_mode(mode: &str) -> bool {
    MODES.iter().any(|m| m.0 == mode)
}

pub fn output_ext(mode: &str) -> Option<&'static str> {
    MODES.iter().find(|m| m.0 == mode).map(|m| m.1)
}

fn formato(mode: &str) -> Option<&'static str> {
    MODES
        .iter()
        .find(|m| m.0 == mode)
        .map(|m| m.2)
        .filter(|f| !f.is_empty())
}

pub fn is_psp_ext(ext: &str) -> bool {
    matches!(ext, "iso" | "cso" | "zso" | "dax")
}

/// Modo sugerido segun lo que se suelte: un ISO se comprime, lo demas se abre.
pub fn suggest_mode(ext: &str) -> &'static str {
    if ext == "iso" {
        "iso2cso"
    } else {
        "cso2iso"
    }
}

/// `maxcso [--format=X] [--fast|--use-zopfli] [--threads=N] <entrada> -o <salida>`
pub fn args(mode: &str, input: &str, output: &str, s: &Settings) -> Vec<String> {
    let mut a: Vec<String> = vec![];

    if mode == "cso2iso" {
        a.push("--decompress".into());
    } else if let Some(f) = formato(mode) {
        a.push(format!("--format={f}"));

        // El preset general de la app decide cuanto esfuerzo se le mete.
        //
        // Medido sobre un UMD de 1,47 GB:
        //   --fast                        33,3 %   en   8 s
        //   por defecto                   33,1 %   en  99 s
        //   con 7zdeflate y libdeflate    33,1 %   en  98 s
        //   anadiendo zopfli                 ?     en ~25 min
        //
        // Por eso zopfli no se usa: multiplica por veinte el tiempo para ganar,
        // como mucho, una fraccion de punto.
        match s.preset.as_str() {
            "fast" => a.push("--fast".into()),
            "max" => {
                a.push("--use-7zdeflate".into());
                a.push("--use-libdeflate".into());
            }
            _ => {}
        }
    }

    if s.threads > 0 {
        a.push(format!("--threads={}", s.threads));
    }

    a.push(input.to_string());
    a.push("-o".into());
    a.push(output.to_string());
    a
}

// No hay funcion para `--measure`: maxcso solo escribe su resultado cuando la
// salida es una consola de verdad. Desde la aplicacion nunca devuelve nada, ni
// siquiera redirigiendolo a un archivo, asi que la opcion se retiro.
