//! Conversion de ISOs de Xbox 360 (y Xbox original) al formato GOD.
//!
//! GOD son las siglas de Games On Demand: el mismo formato que usa la consola
//! para los juegos descargados de la tienda. En vez de un archivo unico, es una
//! carpeta con el contenido troceado en partes de 1 GB, asi que tambien vale
//! para copiarlo a un disco formateado en FAT32.
//!
//! Lo hace `iso2god`, que ademas sabe recortar el espacio vacio del ISO: los
//! discos de Xbox 360 llevan una zona de relleno enorme que no hace falta.

use crate::settings::Settings;

/// (modo, herramienta)
pub const MODES: &[(&str, &str)] = &[
    ("iso2god", "iso2god"),
    // Extraer a carpeta deja el default.xex y todos los archivos del juego,
    // que es lo que piden los lanzadores tipo Aurora o Freestyle Dash. Hace
    // falta porque no todos los juegos arrancan en formato GOD.
    ("iso2folder", "xiso"),
    ("folder2iso", "xiso"),
];

pub fn is_mode(mode: &str) -> bool {
    MODES.iter().any(|m| m.0 == mode)
}

pub fn tool_for(mode: &str) -> Option<&'static str> {
    MODES.iter().find(|m| m.0 == mode).map(|m| m.1)
}

/// `iso2god [--trim] [-j N] <iso> <carpeta destino>`
pub fn args(input: &str, dest_dir: &str, s: &Settings) -> Vec<String> {
    let mut a: Vec<String> = vec![];

    if s.xbox_trim {
        a.push("--trim".into());
    }
    if s.threads > 0 {
        a.push("-j".into());
        a.push(s.threads.to_string());
    }

    a.push(input.to_string());
    a.push(dest_dir.to_string());
    a
}

/// Argumentos para leer la ficha del juego sin convertir nada.
pub fn probe_args(input: &str) -> Vec<String> {
    vec!["--dry-run".into(), input.to_string(), ".".into()]
}

/// `extract-xiso -x [-s] -d <carpeta> <iso>`
///
/// extract-xiso reconoce los tres formatos de disco de Xbox (XGD1, XGD2 y
/// XGD3), asi que sirve tanto para Xbox original como para 360.
pub fn extract_args(input: &str, dest: &str, s: &Settings) -> Vec<String> {
    let mut a = vec!["-x".to_string()];
    if s.xbox_skip_update {
        // $SystemUpdate es el actualizador del disco: no hace falta para jugar
        a.push("-s".into());
    }
    a.push("-d".into());
    a.push(dest.to_string());
    a.push(input.to_string());
    a
}

/// `extract-xiso -c <carpeta> <iso destino>`
pub fn build_args(dir: &str, output: &str) -> Vec<String> {
    vec!["-c".into(), dir.to_string(), output.to_string()]
}
