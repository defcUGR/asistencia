// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use arboard::Clipboard;
use serde::{Deserialize, Deserializer, Serialize};
use serialport::{SerialPort, SerialPortInfo};
use tauri::{Manager, WindowEvent};
use ts_rs::TS;

static EMIT_IDS: AtomicBool = AtomicBool::new(true);
static mut SCAN_HANDLE: Mutex<Option<JoinHandle<Result<(), String>>>> = Mutex::new(None);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn paste() -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

#[tauri::command]
fn start_scan(app_handle: tauri::AppHandle, port_name: &str) -> Result<(), String> {
    let mut port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .map_err(|e| e.to_string())?;

    if unsafe { SCAN_HANDLE.lock().map_err(|e| e.to_string())?.is_some() } {
        return Err("scan already running".to_owned());
    }
    EMIT_IDS.store(true, Ordering::SeqCst);
    start_scanner(app_handle, port).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
    serialport::available_ports().map_err(|e| e.to_string())
}

fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

fn vec_deserialize<'de, D>(d: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    if &s == "[]" {
        return Ok(vec![]);
    }
    let v: Vec<String> = rem_first_and_last(&s)
        .split(',')
        .map(|s| rem_first_and_last(s).to_owned())
        .collect();
    Ok(v)
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
struct RawAttendant {
    #[serde(rename(deserialize = "Apellidos"))]
    last_name: String,
    #[serde(rename(deserialize = "Nombre"))]
    name: String,
    #[serde(rename(deserialize = "Nombre completo"))]
    full_name: String,
    #[serde(rename(deserialize = "Correo UGR"))]
    email: String,
    #[serde(rename(deserialize = "NIF"))]
    nif: Option<String>,
    #[serde(rename(deserialize = "TUI"))]
    tui: Option<String>,
    #[serde(rename(deserialize = "Grado"))]
    degree: Option<String>,
    #[serde(rename(deserialize = "Curso"))]
    course: Option<String>,
    #[serde(rename(deserialize = "Grupo"))]
    group: Option<String>,
    #[serde(rename(deserialize = "Delegadx"), deserialize_with = "vec_deserialize")]
    delegado: Vec<String>,
    #[serde(
        rename(deserialize = "Subdelegadx"),
        deserialize_with = "vec_deserialize"
    )]
    subdelegado: Vec<String>,
    #[serde(rename(deserialize = "Electo"), deserialize_with = "vec_deserialize")]
    electo: Vec<String>,
    #[serde(
        rename(deserialize = "Junta de Centro"),
        deserialize_with = "vec_deserialize"
    )]
    junta_de_centro: Vec<String>,
    #[serde(rename(deserialize = "Claustro"), deserialize_with = "vec_deserialize")]
    claustro: Vec<String>,

    #[serde(
        rename(deserialize = "V. Actividades"),
        deserialize_with = "vec_deserialize"
    )]
    v_actividades: Vec<String>,
    #[serde(
        rename(deserialize = "V. Comunicación"),
        deserialize_with = "vec_deserialize"
    )]
    v_comunicacion: Vec<String>,
    #[serde(
        rename(deserialize = "V. Extensión"),
        deserialize_with = "vec_deserialize"
    )]
    v_extension: Vec<String>,

    #[serde(rename(deserialize = "Pronombres"))]
    pronouns: Option<String>,
    #[serde(rename(deserialize = "Apodo"))]
    nickname: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
struct AttendantChecks {
    is_delegado: bool,
    is_subdelegado: bool,
    is_electo: bool,
    is_junta_de_centro: bool,
    is_claustro: bool,
    is_voluntario: bool,

    has_own_vote: bool,
}

impl AttendantChecks {
    fn from_attendant(att: &RawAttendant) -> Self {
        let is = |v: &Vec<String>| v.len() % 2 == 1;
        Self {
            is_delegado: is(&att.delegado),
            is_subdelegado: is(&att.subdelegado),
            is_electo: is(&att.electo),
            is_junta_de_centro: is(&att.junta_de_centro),
            is_claustro: is(&att.claustro),
            is_voluntario: is(&att.v_actividades)
                || is(&att.v_comunicacion)
                || is(&att.v_extension),

            has_own_vote: is(&att.claustro) || is(&att.electo) || is(&att.junta_de_centro),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
struct Attendant {
    raw: RawAttendant,
    checks: AttendantChecks,
}

#[tauri::command]
fn process_csv(path: &str) -> Result<Vec<Attendant>, String> {
    let mut vec = Vec::new();
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    for result in rdr.deserialize() {
        let record: RawAttendant = result.map_err(|e| e.to_string())?;
        let checks: AttendantChecks = AttendantChecks::from_attendant(&record);
        vec.push(Attendant {
            raw: record,
            checks,
        });
    }
    Ok(vec)
}

type CSVExportData = Vec<CSVExportItem>;

#[derive(Debug, Deserialize, Serialize, TS)]
struct CSVExportItem {
    new: bool,
    time: String,
    tui: String,
    full_name: String,
}

#[tauri::command]
fn export_csv(
    app_handle: tauri::AppHandle,
    file_path: &str,
    data: CSVExportData,
) -> Result<String, String> {
    // This gets blocked
    // let file_path = match tauri::api::dialog::blocking::FileDialogBuilder::new()
    //     .set_title("Guardar CSV")
    //     .save_file()
    // {
    //     Some(p) => p,
    //     None => {
    //         return Err("The user did not select a location for the file to be saved".to_owned())
    //     }
    // };

    // let mut file = std::fs::File::create(file_path);
    let mut wtr = csv::Writer::from_path(&file_path).map_err(|e| e.to_string())?;

    for r in data {
        wtr.serialize(r).map_err(|e| e.to_string())?;
        app_handle
            .emit_all("export_csv_progress", None::<()>)
            .map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;

    Ok(file_path.to_owned())
}

type LimeSurveyExportData = Vec<LimeSurveyExportItem>;

fn default_ok() -> String {
    String::from("OK")
}

fn default_es() -> String {
    String::from("es")
}

fn default_N() -> String {
    String::from("N")
}

fn default_0() -> u32 {
    0
}

fn default_1() -> u32 {
    1
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
struct LimeSurveyExportItem {
    tid: u32,
    firstname: String,
    lastname: String,
    email: String,
    #[serde(default = "default_ok")]
    emailstatus: String,
    token: Option<String>,
    #[serde(default = "default_es")]
    language: String,
    validfrom: Option<String>,
    validuntil: Option<String>,
    #[serde(default = "default_N")]
    invited: String,
    #[serde(default = "default_N")]
    reminded: String,
    #[serde(default = "default_0")]
    remindercount: u32,
    #[serde(default = "default_N")]
    completed: String,
    #[serde(default = "default_1")]
    usesleft: u32,
}

#[tauri::command]
fn export_lime_survey(
    app_handle: tauri::AppHandle,
    file_path: &str,
    data: LimeSurveyExportData,
) -> Result<String, String> {
    let mut wtr = csv::Writer::from_path(&file_path).map_err(|e| e.to_string())?;

    for r in data {
        wtr.serialize(r).map_err(|e| e.to_string())?;
        app_handle
            .emit_all("export_lime_survey_progress", None::<()>)
            .map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;

    Ok(file_path.to_owned())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            paste,
            start_scan,
            get_serial_ports,
            process_csv,
            export_csv,
            export_lime_survey
        ])
        .on_window_event(move |ev| match *ev.event() {
            WindowEvent::CloseRequested { .. } => EMIT_IDS.store(false, Ordering::SeqCst),
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_scanner(
    handle: tauri::AppHandle,
    mut port: Box<dyn SerialPort>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("called scan to start");
    unsafe {
        *(SCAN_HANDLE.lock().unwrap()) = Some(thread::spawn(move || {
            let close_listen_id = handle.listen_global("close_scan", |_| {
                EMIT_IDS.store(false, Ordering::SeqCst);
            });

            let mut serial_buf: Vec<u8> = vec![0; 37];
            while EMIT_IDS.load(Ordering::SeqCst) {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(_) => {
                        println!("Scanned: {:?}", String::from_utf8(serial_buf.clone()));
                        handle
                            .emit_all("id_scanned", String::from_utf8(serial_buf.clone()).unwrap())
                            .map_err(|e: tauri::Error| e.to_string())
                            .unwrap()
                    }
                    Err(_) => {}
                }
            }

            std::mem::drop(port);
            handle.unlisten(close_listen_id);
            handle
                .emit_all("scan_closed", true)
                .map_err(|e| e.to_string())
                .unwrap();
            *(SCAN_HANDLE.lock().unwrap()) = None;

            Ok(())
        }));
    }

    Ok(())
}
