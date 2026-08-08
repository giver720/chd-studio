use crate::chdman;
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncReadExt, BufReader};

pub const EV_JOB: &str = "job://update";
pub const EV_TOAST: &str = "app://toast";

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub input: String,
    pub input_name: String,
    pub output: String,
    /// Segunda salida (el .bin que acompana a un .cue al extraer)
    pub output_extra: Option<String>,
    /// Herramienta que ejecuta el trabajo: chdman | nsz | 4nxci
    pub tool: String,
    /// Accion concreta dentro de esa herramienta
    pub mode: String,
    /// Id del perfil de sistema, solo para mostrarlo en la interfaz
    pub system: String,
    pub codecs: Vec<String>,
    pub hunk_size: Option<u32>,
    pub unit_size: Option<u32>,
    /// queued | running | done | error | canceled
    pub status: String,
    pub progress: f32,
    pub phase: String,
    pub ratio: Option<f32>,
    pub message: Option<String>,
    pub log: Vec<String>,
    pub input_size: u64,
    pub output_size: u64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

impl Job {
    pub fn new(input: String, output: String, tool: String, mode: String, system: String) -> Self {
        let input_name = Path::new(&input)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| input.clone());
        let input_size = std::fs::metadata(&input).map(|m| m.len()).unwrap_or(0);
        Self {
            id: format!("{}-{}", now_ms(), fastrand_hex()),
            input,
            input_name,
            output,
            output_extra: None,
            tool,
            mode,
            system,
            codecs: vec![],
            hunk_size: None,
            unit_size: None,
            status: "queued".into(),
            progress: 0.0,
            phase: "En cola".into(),
            ratio: None,
            message: None,
            log: vec![],
            input_size,
            output_size: 0,
            started_at: None,
            finished_at: None,
        }
    }
}

/// Identificador corto y unico sin dependencias extra.
fn fastrand_hex() -> String {
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    format!("{:x}", n.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ t)
}

#[derive(Default)]
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub jobs: Mutex<Vec<Job>>,
    pub cancels: Mutex<std::collections::HashMap<String, Arc<AtomicBool>>>,
    pub children: Mutex<std::collections::HashMap<String, u32>>,
    pub pumping: AtomicBool,
}

impl AppState {
    pub fn snapshot(&self) -> Vec<Job> {
        self.jobs.lock().unwrap().clone()
    }

    pub fn update<F: FnOnce(&mut Job)>(&self, id: &str, f: F) -> Option<Job> {
        let mut jobs = self.jobs.lock().unwrap();
        let job = jobs.iter_mut().find(|j| j.id == id)?;
        f(job);
        Some(job.clone())
    }
}

fn emit_job(app: &AppHandle, job: &Job) {
    let _ = app.emit(EV_JOB, job);
}

#[allow(dead_code)]
pub fn toast(app: &AppHandle, kind: &str, text: &str) {
    let _ = app.emit(EV_TOAST, serde_json::json!({ "kind": kind, "text": text }));
}

/// Reparte la construccion de argumentos segun la herramienta del trabajo.
fn build_args(job: &Job, s: &Settings) -> Vec<String> {
    match job.tool.as_str() {
        "nsz" => {
            let dir = out_dir_of(job);
            crate::switch::nsz_args(&job.mode, &job.input, &dir, s)
        }
        "4nxci" => crate::switch::nxci_args(&job.input, &out_dir_of(job), s),
        "z3ds" => crate::threeds::z3ds_args(&job.input, &job.output),
        "3dsconv" => crate::threeds::conv_args(&job.input, &out_dir_of(job), s),
        "iso2god" => crate::xbox360::args(&job.input, &job.output, s),
        "xiso" => match job.mode.as_str() {
            "folder2iso" => crate::xbox360::build_args(&job.input, &job.output),
            _ => crate::xbox360::extract_args(&job.input, &job.output, s),
        },
        "maxcso" => crate::psp::args(&job.mode, &job.input, &job.output, s),
        "wit" => crate::wii::wbfs_args(&job.input, &job.output, s),
        "dolphintool" => {
            // Carpeta propia para los temporales de Dolphin
            let user = crate::settings::config_dir().join("dolphin");
            let _ = std::fs::create_dir_all(&user);
            let user = user.to_string_lossy().to_string();
            if job.mode == "wiiverify" {
                crate::wii::verify_args(&job.input, &user)
            } else {
                crate::wii::convert_args(&job.mode, &job.input, &job.output, &user, s)
            }
        }
        "ps3iso" => match job.mode.as_str() {
            "ps3build" => crate::ps3::build_args(&job.input, &job.output, s.ps3_split_fat32),
            "ps3split" => crate::ps3::split_args(&job.input),
            // Al extraer no se parte nada: si luego se reconstruye el ISO, los
            // trozos sueltos confundirian a makeps3iso.
            _ => crate::ps3::extract_args(&job.input, &job.output, false),
        },
        _ => chdman_args(job, s),
    }
}

/// Los modos de comprobacion no generan archivo, asi que no hay nada que limpiar.
fn is_verify(mode: &str) -> bool {
    mode == "verify"
}

/// Algunos modos producen una carpeta en vez de un archivo suelto.
fn writes_directory(tool: &str) -> bool {
    tool == "iso2god"
}

fn mode_writes_directory(job: &Job) -> bool {
    writes_directory(&job.tool) || job.mode == "ps3extract" || job.mode == "iso2folder"
}

/// Suma recursiva del contenido de una carpeta, para poder informar del tamaño.
fn dir_size(path: &Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(path) else {
        return 0;
    };
    rd.flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => dir_size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

/// Borra el resultado a medias, sea archivo o carpeta.
fn remove_output(job: &Job) {
    if mode_writes_directory(job) {
        let _ = std::fs::remove_dir_all(&job.output);
    } else {
        let _ = std::fs::remove_file(&job.output);
    }
}

/// Tamaño del resultado, sea un archivo suelto o una carpeta entera.
fn output_size_of(job: &Job) -> u64 {
    let p = Path::new(&job.output);
    if mode_writes_directory(job) {
        dir_size(p)
    } else {
        std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
    }
}

/// Varias herramientas escriben en una carpeta, no en un archivo concreto.
fn out_dir_of(job: &Job) -> String {
    Path::new(&job.output)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Construye la linea de argumentos de chdman para un trabajo.
fn chdman_args(job: &Job, s: &Settings) -> Vec<String> {
    let mut a: Vec<String> = vec![job.mode.clone()];
    a.push("-i".into());
    a.push(job.input.clone());

    if job.mode != "verify" {
        a.push("-o".into());
        a.push(job.output.clone());
        if let Some(extra) = &job.output_extra {
            a.push("-ob".into());
            a.push(extra.clone());
        }
        if s.overwrite {
            a.push("-f".into());
        }
    }

    if job.mode.starts_with("create") {
        if !job.codecs.is_empty() {
            a.push("-c".into());
            a.push(job.codecs.join(","));
        }
        if let Some(hs) = job.hunk_size {
            a.push("-hs".into());
            a.push(hs.to_string());
        }
        if job.mode == "createraw" {
            a.push("-us".into());
            a.push(job.unit_size.unwrap_or(512).to_string());
        }
        if s.threads > 0 {
            a.push("-np".into());
            a.push(s.threads.to_string());
        }
    }

    a
}

/// Lee el porcentaje de una linea de progreso.
///
/// chdman escribe "Compressing, 43.2% complete... (ratio=51.0%)" mientras que
/// nsz usa una barra estilo tqdm con el porcentaje suelto, asi que se admiten
/// las dos formas.
fn parse_progress(line: &str) -> Option<(f32, Option<f32>, String)> {
    let pct: f32 = match line.find("% complete") {
        Some(idx) => number_before(&line[..idx])?,
        None => percent_token(line)?,
    };
    if !(0.0..=100.0).contains(&pct) {
        return None;
    }

    let ratio = line.find("ratio=").and_then(|i| {
        line[i + 6..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>()
            .parse::<f32>()
            .ok()
    });

    let head = line.split(',').next().unwrap_or("").trim();
    let phase = if head.contains("Compress") {
        "Comprimiendo"
    } else if head.contains("Extract") || head.contains("Decompress") {
        "Extrayendo"
    } else if head.contains("Verif") {
        "Verificando"
    } else if head.contains("Analyz") {
        "Analizando"
    } else {
        "Procesando"
    };

    Some((pct, ratio, phase.to_string()))
}

/// Numero pegado al final de un fragmento: "Compressing, 43.2" -> 43.2
fn number_before(head: &str) -> Option<f32> {
    let num: String = head
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    num.parse().ok()
}

/// Ultimo token con forma de porcentaje de la linea: "[ 43%|####]" -> 43
fn percent_token(line: &str) -> Option<f32> {
    let mut best = None;
    for (i, c) in line.char_indices() {
        if c == '%' {
            if let Some(v) = number_before(&line[..i]) {
                best = Some(v);
            }
        }
    }
    best
}

/// Lee un pipe partiendo por \r y \n, ya que chdman reescribe la misma linea.
async fn pump_pipe<R>(reader: R, tx: tokio::sync::mpsc::UnboundedSender<String>)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let mut reader = BufReader::new(reader);
    let mut buf = [0u8; 1024];
    let mut acc = String::new();
    loop {
        match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                acc.push_str(&String::from_utf8_lossy(&buf[..n]));
                while let Some(pos) = acc.find(['\r', '\n']) {
                    let line = acc[..pos].trim().to_string();
                    acc.drain(..pos + 1);
                    if !line.is_empty() {
                        let _ = tx.send(line);
                    }
                }
            }
        }
    }
    let rest = acc.trim().to_string();
    if !rest.is_empty() {
        let _ = tx.send(rest);
    }
}

async fn kill_pid(pid: u32) {
    #[cfg(windows)]
    {
        let mut c = tokio::process::Command::new("taskkill");
        c.args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        chdman::hide_console(&mut c);
        let _ = c.status().await;
    }
    #[cfg(not(windows))]
    {
        let _ = tokio::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .await;
    }
}

/// CIA -> CCI descifrado.
///
/// No basta con sacar el contenido y volver a empaquetarlo: dentro del CIA cada
/// particion es un NCCH con su propio cifrado, y Azahar no sabe descifrar. Hay
/// que deshacerlo aqui, y para eso hacen falta tres herramientas:
///
///   1. `ctrtool --contents`  saca las particiones (aun cifradas)
///   2. `3dstool --header`    copia la cabecera NCCH, que va en claro
///   3. `ctrtool --exefs...`  descifra las secciones con boot9 y la seeddb
///   4. `3dstool -cvtf`       las vuelve a montar marcadas como no cifradas
///   5. `makerom -f cci`      junta las particiones en el CCI final
///
/// Los intermedios se van borrando segun dejan de hacer falta: un juego grande
/// llegaria a ocupar cuatro veces su tamano si se guardaran todos a la vez.
async fn run_cia2cci(app: AppHandle, id: String, job: Job, s: Settings) {
    let state = app.state::<AppState>();

    let fail = |msg: String| {
        let st = app.state::<AppState>();
        if let Some(j) = st.update(&id, |j| {
            j.status = "error".into();
            j.phase = "Error".into();
            j.message = Some(msg.clone());
            j.finished_at = Some(now_ms());
        }) {
            emit_job(&app, &j);
        }
    };

    let paso = |texto: &str, pct: f32| {
        let st = app.state::<AppState>();
        if let Some(j) = st.update(&id, |j| {
            j.phase = texto.to_string();
            j.progress = pct;
        }) {
            emit_job(&app, &j);
        }
    };

    let Some(ctrtool) = crate::tools::locate("ctrtool", &s).map(|(p, _)| p) else {
        return fail("Falta ctrtool. Instalalo desde Ajustes -> Herramientas.".into());
    };
    let Some(tresdstool) = crate::tools::locate("3dstool", &s).map(|(p, _)| p) else {
        return fail("Falta 3dstool. Instalalo desde Ajustes -> Herramientas.".into());
    };
    let Some(makerom) = crate::tools::locate("makerom", &s).map(|(p, _)| p) else {
        return fail("Falta makerom. Instalalo desde Ajustes -> Herramientas.".into());
    };

    let work = std::env::temp_dir().join(format!("chd-studio-cia-{}", id));
    if let Err(e) = std::fs::create_dir_all(&work) {
        return fail(format!("No se pudo crear la carpeta temporal: {e}"));
    }

    // ctrtool solo lee boot9 de <HOME>/.3ds, asi que se le monta uno temporal
    let mut env: Vec<(&str, String)> = vec![];
    if let Some(home) = crate::threeds::prepare_ctrtool_home(&s, &work) {
        let h = home.to_string_lossy().to_string();
        env.push(("HOME", h.clone()));
        env.push(("USERPROFILE", h));
    }

    paso("Extrayendo el CIA", 5.0);
    let args = crate::threeds::ctrtool_args(&job.input, &work);
    match chdman::run_capture_env(&ctrtool, &args, &env).await {
        Ok((false, out)) => {
            let _ = std::fs::remove_dir_all(&work);
            let tail = out.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("ctrtool fallo");
            return fail(format!("ctrtool: {tail}"));
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&work);
            return fail(format!("No se pudo ejecutar ctrtool: {e}"));
        }
        Ok((true, _)) => {}
    }

    let partes = crate::threeds::collect_contents(&work);
    if partes.is_empty() {
        let _ = std::fs::remove_dir_all(&work);
        return fail("ctrtool no extrajo ningun contenido del CIA.".into());
    }

    // Cada particion se descifra y se vuelve a montar por separado
    let total = partes.len().max(1);
    let mut descifradas: Vec<(String, u32)> = vec![];

    for (n, (nombre, idx)) in partes.iter().enumerate() {
        let base = 10.0 + (n as f32 / total as f32) * 75.0;
        let tipo = crate::threeds::tipo_particion(*idx);
        let ncch = work.join(nombre);
        let sec = work.join(format!("p{idx}"));
        if let Err(e) = std::fs::create_dir_all(&sec) {
            let _ = std::fs::remove_dir_all(&work);
            return fail(format!("No se pudo crear la carpeta de trabajo: {e}"));
        }

        paso(&format!("Leyendo la particion {}", idx + 1), base);
        let args = crate::threeds::header_args(
            tipo,
            &ncch.to_string_lossy(),
            &sec.join("ncch.bin").to_string_lossy(),
        );
        if let Ok((false, out)) = chdman::run_capture(&tresdstool, &args).await {
            let _ = std::fs::remove_dir_all(&work);
            let tail = out.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("");
            return fail(format!("3dstool no pudo leer la cabecera: {tail}"));
        }

        paso(
            &format!("Descifrando la particion {}", idx + 1),
            base + 20.0 / total as f32,
        );
        let args = crate::threeds::split_args(&ncch.to_string_lossy(), &sec, &s);
        match chdman::run_capture_env(&ctrtool, &args, &env).await {
            Ok((_, out)) if out.to_lowercase().contains("unable to decrypt") => {
                let _ = std::fs::remove_dir_all(&work);
                let falta_seed = out.to_lowercase().contains("seed");
                return fail(if falta_seed {
                    "Este juego usa cifrado por semilla y no se encontro seeddb.bin. \
                     Indicale su ruta en esta misma pantalla."
                        .into()
                } else {
                    "No se pudo descifrar el contenido. Revisa que boot9.bin sea correcto."
                        .to_string()
                });
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&work);
                return fail(format!("No se pudo descifrar: {e}"));
            }
            Ok(_) => {}
        }

        // El NCCH cifrado ya no hace falta y ocupa lo mismo que el juego
        let _ = std::fs::remove_file(&ncch);

        paso(
            &format!("Rearmando la particion {}", idx + 1),
            base + 45.0 / total as f32,
        );
        let salida = work.join(format!("dec{idx}.{tipo}"));
        let args = crate::threeds::rebuild_args(tipo, &salida.to_string_lossy(), &sec);
        match chdman::run_capture(&tresdstool, &args).await {
            Ok((false, out)) => {
                let _ = std::fs::remove_dir_all(&work);
                let tail = out.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("");
                return fail(format!("3dstool no pudo rearmar la particion: {tail}"));
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&work);
                return fail(format!("No se pudo rearmar la particion: {e}"));
            }
            Ok((true, _)) => {}
        }

        let _ = std::fs::remove_dir_all(&sec);
        descifradas.push((
            salida.file_name().unwrap().to_string_lossy().to_string(),
            *idx,
        ));
    }

    paso("Montando el CCI", 88.0);
    if let Some(parent) = Path::new(&job.output).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let args = crate::threeds::makerom_args(&job.output, &descifradas);
    let resultado = chdman::run_capture_in(&makerom, &args, &[], Some(&work)).await;
    let _ = std::fs::remove_dir_all(&work);

    match resultado {
        Ok((true, _)) => {}
        Ok((false, out)) => {
            let tail = out
                .lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("makerom fallo");
            return fail(format!("makerom: {tail}"));
        }
        Err(e) => return fail(format!("No se pudo ejecutar makerom: {e}")),
    }

    let out_size = std::fs::metadata(&job.output).map(|m| m.len()).unwrap_or(0);
    if out_size == 0 {
        return fail("makerom no genero ningun archivo".into());
    }

    if let Some(j) = state.update(&id, |j| {
        j.status = "done".into();
        j.phase = "Listo".into();
        j.progress = 100.0;
        j.output_size = out_size;
        j.finished_at = Some(now_ms());
    }) {
        emit_job(&app, &j);
    }
}


/// Ejecuta un trabajo de principio a fin, emitiendo progreso al frontend.
async fn run_job(app: AppHandle, id: String) {
    let state = app.state::<AppState>();
    let (job, settings, exe) = {
        let s = state.settings.lock().unwrap().clone();
        let job = match state.jobs.lock().unwrap().iter().find(|j| j.id == id) {
            Some(j) => j.clone(),
            None => return,
        };
        // ps3iso-utils son cuatro programas en una sola descarga: hay que coger
        // el que toque segun el modo.
        let exe = if job.tool == "ps3iso" {
            crate::tools::locate_sibling("ps3iso", crate::ps3::exe_for(&job.mode))
        } else {
            crate::tools::locate(&job.tool, &s).map(|(p, _)| p)
        };
        (job, s, exe)
    };

    // Este modo encadena dos herramientas, asi que sigue su propio camino
    if job.mode == "cia2cci" {
        if let Some(j) = state.update(&id, |j| {
            j.status = "running".into();
            j.phase = "Iniciando".into();
            j.started_at = Some(now_ms());
        }) {
            emit_job(&app, &j);
        }
        return run_cia2cci(app.clone(), id, job, settings).await;
    }

    let Some(exe) = exe else {
        let tool = job.tool.clone();
        if let Some(j) = state.update(&id, |j| {
            j.status = "error".into();
            j.message = Some(format!(
                "No se encontro «{tool}». Instalalo desde Ajustes → Herramientas."
            ));
            j.finished_at = Some(now_ms());
        }) {
            emit_job(&app, &j);
        }
        return;
    };

    let cancel = Arc::new(AtomicBool::new(false));
    state
        .cancels
        .lock()
        .unwrap()
        .insert(id.clone(), cancel.clone());

    if let Some(j) = state.update(&id, |j| {
        j.status = "running".into();
        j.phase = "Iniciando".into();
        j.started_at = Some(now_ms());
        j.progress = 0.0;
    }) {
        emit_job(&app, &j);
    }

    // Asegura que exista la carpeta de destino. Si la salida es en si una
    // carpeta (GOD), hay que crearla entera y no solo la que la contiene.
    if writes_directory(&job.tool) {
        let _ = std::fs::create_dir_all(&job.output);
    } else if let Some(parent) = Path::new(&job.output).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let args = build_args(&job, &settings);
    let mut cmd = tokio::process::Command::new(&exe);
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    chdman::hide_console(&mut cmd);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            if let Some(j) = state.update(&id, |j| {
                j.status = "error".into();
                j.message = Some(format!("No se pudo iniciar chdman: {e}"));
                j.finished_at = Some(now_ms());
            }) {
                emit_job(&app, &j);
            }
            state.cancels.lock().unwrap().remove(&id);
            return;
        }
    };

    if let Some(pid) = child.id() {
        state.children.lock().unwrap().insert(id.clone(), pid);
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    if let Some(out) = child.stdout.take() {
        tauri::async_runtime::spawn(pump_pipe(out, tx.clone()));
    }
    if let Some(err) = child.stderr.take() {
        tauri::async_runtime::spawn(pump_pipe(err, tx.clone()));
    }
    drop(tx);

    // Varias herramientas (maxcso, extract-xiso, las de PS3...) callan cuando
    // su salida va a una tuberia en vez de a una consola, asi que nunca llega
    // un porcentaje. Sin esto el trabajo parece colgado aunque este avanzando.
    // El latido mira cuanto ha escrito ya y lo va contando.
    let saw_progress = Arc::new(AtomicBool::new(false));
    let done_flag = Arc::new(AtomicBool::new(false));
    {
        let app = app.clone();
        let id = id.clone();
        let job_out = job.output.clone();
        let is_dir = mode_writes_directory(&job);
        let saw = saw_progress.clone();
        let done = done_flag.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1200)).await;
                if done.load(Ordering::Relaxed) {
                    break;
                }
                if saw.load(Ordering::Relaxed) {
                    continue;
                }
                let p = Path::new(&job_out);
                let size = if is_dir {
                    dir_size(p)
                } else {
                    std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
                };
                let st = app.state::<AppState>();
                if let Some(j) = st.update(&id, |j| {
                    j.output_size = size;
                    if j.phase == "Iniciando" {
                        j.phase = "Procesando".into();
                    }
                }) {
                    emit_job(&app, &j);
                }
            }
        });
    }

    let mut last_emit = Instant::now() - Duration::from_secs(1);
    let mut last_pct = -1.0f32;

    while let Some(line) = rx.recv().await {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        if let Some((pct, ratio, phase)) = parse_progress(&line) {
            saw_progress.store(true, Ordering::Relaxed);
            let changed = (pct - last_pct).abs() >= 0.2;
            if changed && last_emit.elapsed() >= Duration::from_millis(120) {
                last_pct = pct;
                last_emit = Instant::now();
                if let Some(j) = state.update(&id, |j| {
                    j.progress = pct;
                    j.phase = phase.clone();
                    if ratio.is_some() {
                        j.ratio = ratio;
                    }
                }) {
                    emit_job(&app, &j);
                }
            }
        } else {
            let final_ratio = line.find("final ratio =").and_then(|i| {
                line[i + 13..]
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect::<String>()
                    .parse::<f32>()
                    .ok()
            });
            if let Some(j) = state.update(&id, |j| {
                if let Some(r) = final_ratio {
                    j.ratio = Some(r);
                }
                j.log.push(line.clone());
                if j.log.len() > 400 {
                    j.log.remove(0);
                }
            }) {
                emit_job(&app, &j);
            }
        }
    }

    if cancel.load(Ordering::Relaxed) {
        // El guard del mutex se suelta aqui, antes del await
        let pid = state.children.lock().unwrap().get(&id).copied();
        if let Some(pid) = pid {
            kill_pid(pid).await;
        }
        let _ = child.start_kill();
    }

    done_flag.store(true, Ordering::Relaxed);

    let ok = match child.wait().await {
        Ok(st) => st.success(),
        Err(_) => false,
    };

    state.children.lock().unwrap().remove(&id);
    state.cancels.lock().unwrap().remove(&id);

    let canceled = cancel.load(Ordering::Relaxed);
    let out_size = output_size_of(&job);

    if canceled {
        // Limpia lo que la herramienta dejo a medias
        if !is_verify(&job.mode) {
            remove_output(&job);
            if let Some(e) = &job.output_extra {
                let _ = std::fs::remove_file(e);
            }
        }
        if let Some(j) = state.update(&id, |j| {
            j.status = "canceled".into();
            j.phase = "Cancelado".into();
            j.message = Some("Cancelado por el usuario".into());
            j.finished_at = Some(now_ms());
        }) {
            emit_job(&app, &j);
        }
        return;
    }

    if !ok {
        let msg = {
            let jobs = state.jobs.lock().unwrap();
            jobs.iter()
                .find(|j| j.id == id)
                .and_then(|j| {
                    j.log
                        .iter()
                        .rev()
                        .find(|l| {
                            let l = l.to_lowercase();
                            l.contains("error") || l.contains("unable") || l.contains("fatal")
                        })
                        .cloned()
                })
                .unwrap_or_else(|| "chdman termino con error".into())
        };
        if !is_verify(&job.mode) && out_size == 0 {
            remove_output(&job);
        }
        if let Some(j) = state.update(&id, |j| {
            j.status = "error".into();
            j.phase = "Error".into();
            j.message = Some(msg.clone());
            j.finished_at = Some(now_ms());
        }) {
            emit_job(&app, &j);
        }
        return;
    }

    // Verificacion opcional despues de crear (solo aplica a chdman)
    if settings.verify_after && job.tool == "chdman" && job.mode.starts_with("create") {
        if let Some(j) = state.update(&id, |j| {
            j.phase = "Verificando".into();
            j.progress = 99.0;
        }) {
            emit_job(&app, &j);
        }
        let args = vec!["verify".to_string(), "-i".to_string(), job.output.clone()];
        match chdman::run_capture(&exe, &args).await {
            Ok((true, _)) => {}
            Ok((false, text)) => {
                if let Some(j) = state.update(&id, |j| {
                    j.status = "error".into();
                    j.phase = "Verificacion fallida".into();
                    j.message = Some(text.lines().last().unwrap_or("Verificacion fallida").into());
                    j.finished_at = Some(now_ms());
                }) {
                    emit_job(&app, &j);
                }
                return;
            }
            Err(e) => {
                if let Some(j) = state.update(&id, |j| {
                    j.message = Some(format!("No se pudo verificar: {e}"));
                }) {
                    emit_job(&app, &j);
                }
            }
        }
    }

    // Solo se borra el origen si de verdad se genero algo nuevo
    let produced = job.mode.starts_with("create")
        || matches!(
            job.mode.as_str(),
            "nsp2nsz" | "xci2xcz" | "nsz2nsp" | "xcz2xci" | "xci2nsp"
        );
    if settings.delete_source && produced && out_size > 0 {
        delete_source_set(&job.input);
    }

    if let Some(j) = state.update(&id, |j| {
        j.status = "done".into();
        j.phase = "Listo".into();
        j.progress = 100.0;
        j.output_size = out_size;
        j.finished_at = Some(now_ms());
    }) {
        emit_job(&app, &j);
    }
}

/// Al borrar el origen de un .cue/.gdi hay que borrar tambien sus pistas.
fn delete_source_set(input: &str) {
    let p = PathBuf::from(input);
    let ext = p
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if ext == "cue" || ext == "gdi" || ext == "toc" {
        if let Ok(text) = std::fs::read_to_string(&p) {
            if let Some(dir) = p.parent() {
                for line in text.lines() {
                    // Nombres de pista entre comillas (cue) o sueltos (gdi)
                    let candidates: Vec<String> = if let Some(a) = line.find('"') {
                        line[a + 1..]
                            .find('"')
                            .map(|b| vec![line[a + 1..a + 1 + b].to_string()])
                            .unwrap_or_default()
                    } else {
                        line.split_whitespace()
                            .filter(|t| {
                                let t = t.to_lowercase();
                                t.ends_with(".bin") || t.ends_with(".raw") || t.ends_with(".iso")
                            })
                            .map(|s| s.to_string())
                            .collect()
                    };
                    for c in candidates {
                        let f = dir.join(c);
                        if f.is_file() {
                            let _ = std::fs::remove_file(f);
                        }
                    }
                }
            }
        }
    }
    let _ = std::fs::remove_file(&p);
}

/// Bucle que va sacando trabajos de la cola respetando el limite de paralelismo.
pub fn start_pump(app: AppHandle) {
    let state = app.state::<AppState>();
    if state.pumping.swap(true, Ordering::SeqCst) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            let state = app.state::<AppState>();
            let parallel = state.settings.lock().unwrap().parallel.max(1);
            let (running, next) = {
                let jobs = state.jobs.lock().unwrap();
                let running = jobs.iter().filter(|j| j.status == "running").count();
                let next = jobs
                    .iter()
                    .find(|j| j.status == "queued")
                    .map(|j| j.id.clone());
                (running, next)
            };
            if running < parallel {
                if let Some(id) = next {
                    // Marcado inmediato para que el siguiente tick no lo tome dos veces
                    let _ = state.update(&id, |j| j.status = "running".into());
                    let app2 = app.clone();
                    tauri::async_runtime::spawn(async move { run_job(app2, id).await });
                }
            }
        }
    });
}

pub fn cancel(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    if let Some(flag) = state.cancels.lock().unwrap().get(id) {
        flag.store(true, Ordering::Relaxed);
    }
    let pid = state.children.lock().unwrap().get(id).copied();
    if let Some(pid) = pid {
        tauri::async_runtime::spawn(async move { kill_pid(pid).await });
    } else if let Some(j) = state.update(id, |j| {
        if j.status == "queued" {
            j.status = "canceled".into();
            j.phase = "Cancelado".into();
        }
    }) {
        emit_job(app, &j);
    }
}
