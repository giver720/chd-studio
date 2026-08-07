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

pub const MODE: &str = "iso2god";

pub fn is_mode(mode: &str) -> bool {
    mode == MODE
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
