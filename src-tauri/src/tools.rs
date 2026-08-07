//! Registro de las herramientas externas que CHD Studio orquesta.
//!
//! Cada consola necesita su propio ejecutable y cada uno se consigue de forma
//! distinta: chdman viaja dentro del instalador, `nsz` es un paquete de Python
//! y `4nxci` es un binario que se publica en GitHub. Este modulo unifica la
//! deteccion y la instalacion para que la interfaz las trate a todas igual.

use crate::chdman;
use crate::settings::{self, Settings};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolKind {
    /// Viaja dentro del instalador de CHD Studio.
    Bundled,
    /// Se instala con pip dentro del entorno privado de la app.
    Python { package: &'static str },
    /// Se descarga de la ultima release de un repositorio de GitHub.
    Github {
        repo: &'static str,
        /// Fragmento que debe contener el nombre del asset.
        asset: &'static str,
        /// Prefijo de la etiqueta, para repos que publican varias herramientas
        /// por separado (Project_CTR usa `ctrtool-v1.3.0`, `makerom-v0.19.0`).
        /// Vacio = usar simplemente la ultima release.
        tag: &'static str,
    },
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ToolSpec {
    pub id: &'static str,
    pub name: &'static str,
    /// Nombre del ejecutable, sin extension.
    pub exe: &'static str,
    pub kind: ToolKind,
    /// Para que sirve, en una linea.
    pub purpose: &'static str,
    /// Plataforma a la que pertenece: chd | switch | 3ds | ps3
    pub family: &'static str,
    /// Licencia, para dejarlo claro en la interfaz.
    pub license: &'static str,
}

pub const TOOLS: &[ToolSpec] = &[
    ToolSpec {
        id: "chdman",
        name: "chdman",
        exe: "chdman",
        kind: ToolKind::Bundled,
        purpose: "Crea y extrae archivos CHD",
        family: "chd",
        license: "GPL-2.0-or-later (MAME)",
    },
    ToolSpec {
        id: "nsz",
        name: "nsz",
        exe: "nsz",
        kind: ToolKind::Python { package: "nsz" },
        purpose: "Comprime y descomprime NSP/NSZ y XCI/XCZ",
        family: "switch",
        license: "MIT",
    },
    ToolSpec {
        id: "4nxci",
        name: "4NXCI",
        exe: "4nxci",
        kind: ToolKind::Github {
            repo: "tetj/4NXCI-2026",
            // El unico asset se llama `4nxci.exe`, sin la plataforma en el nombre
            asset: ".exe",
            tag: "",
        },
        purpose: "Convierte cartuchos XCI a NSP",
        family: "switch",
        license: "ISC",
    },
    ToolSpec {
        id: "z3ds",
        name: "z3ds_compressor",
        exe: "z3ds_compressor",
        kind: ToolKind::Github {
            repo: "energeticokay/z3ds_compress",
            asset: "windows",
            tag: "",
        },
        purpose: "Comprime ROMs al formato Z3DS que lee Azahar",
        family: "3ds",
        license: "GPL-2.0",
    },
    ToolSpec {
        id: "3dsconv",
        name: "3dsconv",
        exe: "3dsconv",
        kind: ToolKind::Github {
            repo: "ihaveamac/3dsconv",
            asset: ".exe",
            tag: "",
        },
        purpose: "Convierte CCI/.3ds a CIA instalable",
        family: "3ds",
        license: "MIT",
    },
    ToolSpec {
        id: "ctrtool",
        name: "ctrtool",
        exe: "ctrtool",
        kind: ToolKind::Github {
            repo: "3DSGuy/Project_CTR",
            asset: "win_x64",
            tag: "ctrtool",
        },
        purpose: "Extrae el contenido de un CIA (paso 1 de CIA → CCI)",
        family: "3ds",
        license: "MIT",
    },
    ToolSpec {
        id: "makerom",
        name: "makerom",
        exe: "makerom",
        kind: ToolKind::Github {
            repo: "3DSGuy/Project_CTR",
            asset: "win_x86_64",
            tag: "makerom",
        },
        purpose: "Reconstruye el CCI desde el contenido (paso 2 de CIA → CCI)",
        family: "3ds",
        license: "MIT",
    },
    ToolSpec {
        id: "iso2god",
        name: "iso2god",
        exe: "iso2god",
        kind: ToolKind::Github {
            repo: "iliazeus/iso2god-rs",
            asset: "windows",
            tag: "",
        },
        purpose: "Convierte ISOs de Xbox 360 y Xbox al formato GOD",
        family: "xbox360",
        license: "MIT",
    },
];

pub fn spec(id: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().find(|t| t.id == id)
}

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

/// Entorno de Python privado de la app, para no tocar el del sistema.
pub fn venv_dir() -> PathBuf {
    settings::config_dir().join("pyenv")
}

fn venv_bin() -> PathBuf {
    if cfg!(windows) {
        venv_dir().join("Scripts")
    } else {
        venv_dir().join("bin")
    }
}

/// Carpeta donde se dejan los binarios descargados de GitHub.
pub fn tools_dir() -> PathBuf {
    settings::config_dir().join("tools")
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatus {
    pub id: &'static str,
    pub name: &'static str,
    pub purpose: &'static str,
    pub family: &'static str,
    pub license: &'static str,
    pub kind: ToolKind,
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    /// bundled | manual | app | venv | tools | path
    pub source: Option<String>,
    /// true si CHD Studio puede instalarla sola
    pub installable: bool,
}

/// Busca el ejecutable de una herramienta por orden de preferencia.
pub fn locate(id: &str, s: &Settings) -> Option<(PathBuf, String)> {
    let spec = spec(id)?;

    // chdman conserva su propia busqueda, que ademas mira instalaciones de MAME
    if id == "chdman" {
        return chdman::locate(s.chdman_path.as_deref());
    }

    if let Some(p) = s.tool_paths.get(id) {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some((p, "manual".into()));
        }
    }

    let name = exe_name(spec.exe);

    let venv = venv_bin().join(&name);
    if venv.is_file() {
        return Some((venv, "venv".into()));
    }

    // Las que viajan dentro del instalador
    for d in chdman::bundled_dirs() {
        let c = d.join(&name);
        if c.is_file() {
            return Some((c, "bundled".into()));
        }
    }

    let downloaded = tools_dir().join(id).join(&name);
    if downloaded.is_file() {
        return Some((downloaded, "tools".into()));
    }
    // Algunos zips traen el binario dentro de una subcarpeta
    if let Some(found) = find_in(&tools_dir().join(id), &name, 0) {
        return Some((found, "tools".into()));
    }

    if let Ok(p) = which::which(spec.exe) {
        return Some((p, "path".into()));
    }

    None
}

fn find_in(dir: &std::path::Path, name: &str, depth: usize) -> Option<PathBuf> {
    if depth > 3 {
        return None;
    }
    let rd = std::fs::read_dir(dir).ok()?;
    let mut dirs = vec![];
    for e in rd.flatten() {
        let p = e.path();
        if p.is_file() {
            if p.file_name().map(|f| f.eq_ignore_ascii_case(name)).unwrap_or(false) {
                return Some(p);
            }
        } else if p.is_dir() {
            dirs.push(p);
        }
    }
    dirs.into_iter().find_map(|d| find_in(&d, name, depth + 1))
}

/// Pregunta la version ejecutando la herramienta; si falla, deja el campo vacio.
async fn probe_version(id: &str, path: &std::path::Path) -> Option<String> {
    let arg = match id {
        "z3ds" | "4nxci" => "--help",
        _ => "--version",
    };
    let (_, text) = chdman::run_capture(path, &[arg.to_string()]).await.ok()?;
    let line = text.lines().map(|l| l.trim()).find(|l| !l.is_empty())?;
    // Varias herramientas no tienen bandera de version y escupen el uso;
    // preferimos no mostrar nada antes que un "usage: ..." sin sentido.
    if line.to_lowercase().starts_with("usage") {
        return None;
    }
    Some(line.chars().take(90).collect())
}

pub async fn status_of(id: &str, s: &Settings) -> ToolStatus {
    let spec = spec(id).expect("id de herramienta desconocido");
    let located = locate(id, s);

    let (found, path, source, version) = match located {
        Some((p, src)) => {
            let v = probe_version(id, &p).await;
            (true, Some(p.to_string_lossy().to_string()), Some(src), v)
        }
        None => (false, None, None, None),
    };

    ToolStatus {
        id: spec.id,
        name: spec.name,
        purpose: spec.purpose,
        family: spec.family,
        license: spec.license,
        kind: spec.kind,
        found,
        path,
        source,
        version,
        installable: !matches!(spec.kind, ToolKind::Bundled),
    }
}

pub async fn status_all(s: &Settings) -> Vec<ToolStatus> {
    let mut out = vec![];
    for t in TOOLS {
        out.push(status_of(t.id, s).await);
    }
    out
}

// ---------------------------------------------------------------- Python

#[derive(Debug, Clone, Serialize)]
pub struct PythonStatus {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub venv_ready: bool,
}

/// Busca un Python utilizable: primero `python`, luego el lanzador `py`.
pub fn find_python() -> Option<PathBuf> {
    for name in ["python3", "python"] {
        if let Ok(p) = which::which(name) {
            // En Windows hay un alias de la Store que no sirve para nada
            if !p.to_string_lossy().contains("WindowsApps") {
                return Some(p);
            }
        }
    }
    which::which("py").ok()
}

pub async fn python_status() -> PythonStatus {
    let path = find_python();
    let version = match &path {
        Some(p) => chdman::run_capture(p, &["--version".to_string()])
            .await
            .ok()
            .map(|(_, t)| t.trim().to_string()),
        None => None,
    };
    PythonStatus {
        found: path.is_some(),
        path: path.map(|p| p.to_string_lossy().to_string()),
        version,
        venv_ready: venv_bin().join(exe_name("pip")).is_file(),
    }
}

/// Crea el entorno virtual si hace falta y devuelve la ruta de pip.
async fn ensure_venv() -> anyhow::Result<PathBuf> {
    let pip = venv_bin().join(exe_name("pip"));
    if pip.is_file() {
        return Ok(pip);
    }

    let python = find_python().ok_or_else(|| {
        anyhow::anyhow!("No hay Python instalado. Instalalo desde python.org y vuelve a intentarlo.")
    })?;

    std::fs::create_dir_all(settings::config_dir())?;
    let (ok, out) = chdman::run_capture(
        &python,
        &[
            "-m".into(),
            "venv".into(),
            venv_dir().to_string_lossy().to_string(),
        ],
    )
    .await?;

    if !ok || !pip.is_file() {
        anyhow::bail!("No se pudo crear el entorno de Python: {}", out.trim());
    }
    Ok(pip)
}

/// Instala (o actualiza) un paquete de Python dentro del entorno de la app.
pub async fn install_python_package(package: &str) -> anyhow::Result<String> {
    let pip = ensure_venv().await?;
    let (ok, out) = chdman::run_capture(
        &pip,
        &[
            "install".into(),
            "--upgrade".into(),
            "--disable-pip-version-check".into(),
            package.into(),
        ],
    )
    .await?;

    if !ok {
        let tail: Vec<&str> = out.lines().rev().take(4).collect();
        anyhow::bail!("pip fallo: {}", tail.into_iter().rev().collect::<Vec<_>>().join(" "));
    }
    Ok(out)
}

// ---------------------------------------------------------------- GitHub

#[derive(serde::Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(serde::Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

/// Descarga el asset de una release que contenga `contains` en su nombre.
///
/// Con `tag_prefix` se coge la release mas reciente cuya etiqueta empiece por
/// ese texto; sin el, simplemente la ultima.
pub async fn install_github_tool(
    id: &str,
    repo: &str,
    contains: &str,
    tag_prefix: &str,
) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("chd-studio")
        .build()?;

    let rel: GhRelease = if tag_prefix.is_empty() {
        client
            .get(format!("https://api.github.com/repos/{repo}/releases/latest"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?
    } else {
        let all: Vec<GhRelease> = client
            .get(format!(
                "https://api.github.com/repos/{repo}/releases?per_page=100"
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        // La API devuelve las releases de la mas nueva a la mas vieja
        all.into_iter()
            .find(|r| r.tag_name.starts_with(tag_prefix))
            .ok_or_else(|| {
                anyhow::anyhow!("{repo} no tiene ninguna release que empiece por «{tag_prefix}»")
            })?
    };

    let needle = contains.to_lowercase();
    let asset = rel
        .assets
        .iter()
        .find(|a| a.name.to_lowercase().contains(&needle))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "La release {} de {repo} no trae ningun archivo que contenga «{contains}»",
                rel.tag_name
            )
        })?;

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let dest = tools_dir().join(id);
    let _ = std::fs::remove_dir_all(&dest);
    std::fs::create_dir_all(&dest)?;

    if asset.name.to_lowercase().ends_with(".zip") {
        let cursor = std::io::Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(cursor)?;
        zip.extract(&dest)?;
    } else {
        // Los binarios sueltos suelen venir con el nombre de la plataforma
        // pegado (iso2god-x86_64-windows.exe); los dejamos con el nombre que
        // luego busca `locate`.
        let name = spec(id)
            .map(|s| exe_name(s.exe))
            .unwrap_or_else(|| asset.name.clone());
        std::fs::write(dest.join(name), &bytes)?;
    }

    Ok(rel.tag_name)
}

/// Punto de entrada unico que usa la interfaz para el boton «Instalar».
pub async fn install(id: &str) -> anyhow::Result<String> {
    let spec = spec(id).ok_or_else(|| anyhow::anyhow!("Herramienta desconocida: {id}"))?;
    match spec.kind {
        ToolKind::Bundled => anyhow::bail!("{} ya viene incluida con CHD Studio", spec.name),
        ToolKind::Python { package } => {
            install_python_package(package).await?;
            Ok(format!("{} instalado con pip", spec.name))
        }
        ToolKind::Github { repo, asset, tag } => {
            let found = install_github_tool(id, repo, asset, tag).await?;
            Ok(format!("{} {found} descargado", spec.name))
        }
    }
}
