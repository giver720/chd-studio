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
    /// Hay que conseguirla aparte: el proyecto no publica en GitHub. La app la
    /// busca en las rutas donde suele instalarse y deja senalarla a mano.
    External { site: &'static str },
    /// Se descarga de una pagina web corriente, buscando en ella el primer
    /// enlace que contenga cierto texto. Para proyectos que publican en su
    /// propio sitio en vez de en GitHub.
    Web {
        /// Pagina con la lista de descargas
        page: &'static str,
        /// Raiz para los enlaces relativos
        base: &'static str,
        /// Fragmento que debe contener el enlace
        contains: &'static str,
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
        id: "3dstool",
        name: "3dstool",
        exe: "3dstool",
        kind: ToolKind::Github {
            repo: "dnasdw/3dstool",
            asset: "3dstool.zip",
            tag: "",
        },
        purpose: "Reconstruye el contenido de 3DS ya descifrado",
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
    ToolSpec {
        id: "xiso",
        name: "extract-xiso",
        exe: "extract-xiso",
        kind: ToolKind::Github {
            repo: "XboxDev/extract-xiso",
            asset: "Win64_Release",
            tag: "",
        },
        purpose: "Extrae ISOs de Xbox 360 a carpeta con default.xex",
        family: "xbox360",
        license: "Ver repositorio",
    },
    ToolSpec {
        id: "ps3iso",
        name: "ps3iso-utils",
        exe: "extractps3iso",
        kind: ToolKind::Github {
            repo: "bucanero/ps3iso-utils",
            asset: "Win64",
            tag: "",
        },
        purpose: "Extrae, reconstruye y parte ISOs de PS3",
        family: "ps3",
        license: "GPL-3.0",
    },
    ToolSpec {
        id: "maxcso",
        name: "maxcso",
        exe: "maxcso",
        kind: ToolKind::Github {
            repo: "unknownbrackets/maxcso",
            // "windows.7z" distingue el de 64 bits del "windows-32bit.7z"
            asset: "windows.7z",
            tag: "",
        },
        purpose: "Comprime ISOs de PSP a CSO, ZSO o DAX",
        family: "psp",
        license: "ISC",
    },
    ToolSpec {
        id: "dolphintool",
        name: "DolphinTool",
        exe: "DolphinTool",
        kind: ToolKind::External {
            site: "https://dolphin-emu.org/download/",
        },
        purpose: "Convierte discos de Wii y GameCube a RVZ",
        family: "wii",
        license: "GPL-2.0 (Dolphin)",
    },
    ToolSpec {
        id: "wit",
        name: "Wiimms ISO Tools",
        exe: "wit",
        kind: ToolKind::Web {
            page: "https://wit.wiimm.de/download.html",
            base: "https://wit.wiimm.de",
            // La pagina lista las versiones de la mas nueva a la mas vieja
            contains: "cygwin64.zip",
        },
        purpose: "Crea WBFS para cargadores USB de Wii",
        family: "wii",
        license: "GPL-2.0",
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

    // Las externas se buscan donde suele instalarlas su propio programa
    if matches!(spec.kind, ToolKind::External { .. }) {
        for d in external_dirs(id) {
            let c = d.join(&name);
            if c.is_file() {
                return Some((c, "sistema".into()));
            }
        }
    }

    None
}

/// Carpetas donde buscar una herramienta que viene dentro de otro programa.
fn external_dirs(id: &str) -> Vec<PathBuf> {
    let prefijo = match id {
        "dolphintool" => "dolphin",
        _ => return vec![],
    };

    // Estos programas se instalan en sitios muy variados, asi que se rastrean
    // las raices buscando carpetas cuyo nombre empiece por el del programa
    let mut dirs = vec![];
    let mut raices: Vec<PathBuf> = vec![];

    #[cfg(windows)]
    {
        for letra in ["C", "D", "E", "F"] {
            let raiz = PathBuf::from(format!("{letra}:\\"));
            if raiz.is_dir() {
                raices.push(raiz.clone());
                // Un nivel de carpetas contenedoras: emuladores, Emulators...
                if let Ok(rd) = std::fs::read_dir(&raiz) {
                    for e in rd.flatten().take(60) {
                        if e.path().is_dir() {
                            raices.push(e.path());
                        }
                    }
                }
            }
        }
        for p in ["Program Files", "Program Files (x86)"] {
            raices.push(PathBuf::from(format!("C:\\{p}")));
        }
        if let Some(home) = dirs::home_dir() {
            raices.push(home.join("Downloads"));
            raices.push(home.join("AppData").join("Local").join("Programs"));
        }
    }

    for raiz in raices {
        let Ok(rd) = std::fs::read_dir(&raiz) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            let n = e.file_name().to_string_lossy().to_lowercase();
            if n.starts_with(prefijo) {
                // wit reparte sus programas en una subcarpeta bin
                dirs.push(p.join("bin"));
                dirs.push(p);
            }
        }
    }

    dirs
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
    // Las de PS3 entran en modo interactivo si no reconocen el argumento, asi
    // que hay que darles justo el que documentan
    let arg = match id {
        "z3ds" | "4nxci" | "ps3iso" => "--help",
        "dolphintool" => "--help",
        _ => "--version",
    };
    // Cinco segundos de sobra para que una herramienta diga su version
    let (_, text) = chdman::run_capture_timeout(path, &[arg.to_string()], 5)
        .await
        .ok()?;
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
        installable: !matches!(
            spec.kind,
            ToolKind::Bundled | ToolKind::External { .. }
        ),
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

    let lower = asset.name.to_lowercase();
    if lower.ends_with(".zip") {
        let cursor = std::io::Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(cursor)?;
        zip.extract(&dest)?;
        // Algunos proyectos meten un tar.gz dentro del zip (ps3iso-utils)
        unpack_nested_tarballs(&dest)?;
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let dec = flate2::read::GzDecoder::new(std::io::Cursor::new(bytes));
        tar::Archive::new(dec).unpack(&dest)?;
    } else if lower.ends_with(".7z") {
        extract_7z(&bytes, &asset.name, &dest).await?;
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

/// Extrae un .7z apoyandose en el `tar` del sistema.
///
/// El tar que trae Windows 10 en adelante es libarchive, que sabe leer 7-Zip,
/// asi que no hace falta que el usuario instale nada ni meter un descompresor
/// entero en la aplicacion.
async fn extract_7z(bytes: &[u8], name: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let tmp = std::env::temp_dir().join(format!("chd-studio-{name}"));
    std::fs::write(&tmp, bytes)?;

    // El tar de Git Bash confunde "C:" con un host remoto; usamos el del sistema
    let tar_exe = if cfg!(windows) {
        std::path::PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into()))
            .join("System32")
            .join("tar.exe")
    } else {
        std::path::PathBuf::from("tar")
    };

    let args = vec![
        "-xf".to_string(),
        tmp.to_string_lossy().to_string(),
        "-C".to_string(),
        dest.to_string_lossy().to_string(),
    ];
    let res = chdman::run_capture(&tar_exe, &args).await;
    let _ = std::fs::remove_file(&tmp);

    match res {
        Ok((true, _)) => Ok(()),
        Ok((false, out)) => anyhow::bail!("No se pudo descomprimir el .7z: {}", out.trim()),
        Err(e) => anyhow::bail!("No se encontro tar para abrir el .7z: {e}"),
    }
}

/// Desempaqueta los .tar.gz que hayan quedado sueltos tras extraer un zip.
fn unpack_nested_tarballs(dir: &std::path::Path) -> anyhow::Result<()> {
    let entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .map(|n| {
                        let n = n.to_string_lossy().to_lowercase();
                        n.ends_with(".tar.gz") || n.ends_with(".tgz")
                    })
                    .unwrap_or(false)
        })
        .collect();

    for tarball in entries {
        let f = std::fs::File::open(&tarball)?;
        let dec = flate2::read::GzDecoder::new(f);
        tar::Archive::new(dec).unpack(dir)?;
        let _ = std::fs::remove_file(&tarball);
    }
    Ok(())
}

/// Busca otro ejecutable dentro de la carpeta ya descargada de una herramienta.
///
/// ps3iso-utils trae cuatro programas en la misma descarga, cada uno en su
/// subcarpeta, asi que se registra uno solo y los demas se localizan al lado.
pub fn locate_sibling(id: &str, exe: &str) -> Option<PathBuf> {
    let name = exe_name(exe);
    let base = tools_dir().join(id);
    let direct = base.join(&name);
    if direct.is_file() {
        return Some(direct);
    }
    if let Some(found) = find_in(&base, &name, 0) {
        return Some(found);
    }
    // Tambien puede venir dentro del instalador
    for d in chdman::bundled_dirs() {
        let c = d.join(&name);
        if c.is_file() {
            return Some(c);
        }
    }
    which::which(exe).ok()
}

/// Descarga una herramienta desde una pagina web corriente.
///
/// Se busca en el HTML el primer enlace que contenga `contains`. Los sitios que
/// listan sus versiones lo hacen de la mas nueva a la mas vieja, asi que el
/// primero es el que interesa. Si el sitio cambia de formato esto dejara de
/// encontrarlo, y por eso el mensaje de error remite a la pagina.
pub async fn install_web_tool(
    id: &str,
    page: &str,
    base: &str,
    contains: &str,
) -> anyhow::Result<String> {
    let client = reqwest::Client::builder().user_agent("chd-studio").build()?;
    let html = client
        .get(page)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Busqueda simple de href="..." sin meter un analizador de HTML entero
    let enlace = html
        .split("href=\"")
        .skip(1)
        .filter_map(|resto| resto.split('"').next())
        .find(|h| h.contains(contains))
        .ok_or_else(|| {
            anyhow::anyhow!("No se encontro ningun enlace con «{contains}» en {page}")
        })?;

    let url = if enlace.starts_with("http") {
        enlace.to_string()
    } else if enlace.starts_with('/') {
        format!("{base}{enlace}")
    } else {
        format!("{base}/{enlace}")
    };

    let bytes = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let dest = tools_dir().join(id);
    let _ = std::fs::remove_dir_all(&dest);
    std::fs::create_dir_all(&dest)?;

    let lower = url.to_lowercase();
    if lower.ends_with(".zip") {
        zip::ZipArchive::new(std::io::Cursor::new(bytes))?.extract(&dest)?;
        unpack_nested_tarballs(&dest)?;
    } else if lower.ends_with(".7z") {
        let nombre = url.rsplit('/').next().unwrap_or("descarga.7z");
        extract_7z(&bytes, nombre, &dest).await?;
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let dec = flate2::read::GzDecoder::new(std::io::Cursor::new(bytes));
        tar::Archive::new(dec).unpack(&dest)?;
    } else {
        let nombre = spec(id)
            .map(|s| exe_name(s.exe))
            .unwrap_or_else(|| "descarga.bin".into());
        std::fs::write(dest.join(nombre), &bytes)?;
    }

    // El nombre del archivo lleva la version, que es lo que se muestra
    Ok(url.rsplit('/').next().unwrap_or("descargado").to_string())
}

/// Punto de entrada unico que usa la interfaz para el boton «Instalar».
pub async fn install(id: &str) -> anyhow::Result<String> {
    let spec = spec(id).ok_or_else(|| anyhow::anyhow!("Herramienta desconocida: {id}"))?;
    match spec.kind {
        ToolKind::Bundled => anyhow::bail!("{} ya viene incluida con CHD Studio", spec.name),
        ToolKind::External { site } => anyhow::bail!(
            "{} viene dentro de otro programa y hay que instalarlo aparte: {site}",
            spec.name
        ),
        ToolKind::Python { package } => {
            install_python_package(package).await?;
            Ok(format!("{} instalado con pip", spec.name))
        }
        ToolKind::Github { repo, asset, tag } => {
            let found = install_github_tool(id, repo, asset, tag).await?;
            Ok(format!("{} {found} descargado", spec.name))
        }
        ToolKind::Web {
            page,
            base,
            contains,
        } => {
            let archivo = install_web_tool(id, page, base, contains).await?;
            Ok(format!("{} descargado ({archivo})", spec.name))
        }
    }
}
