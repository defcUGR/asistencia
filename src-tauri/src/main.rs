// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use arboard::Clipboard;
use serialport::{SerialPort, SerialPortInfo};
use tauri::{Manager, WindowEvent};

static EMIT_IDS: AtomicBool = AtomicBool::new(true);
static mut SCAN_HANDLE: Mutex<Option<JoinHandle<Result<(), String>>>> = Mutex::new(None);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn copy(contents: &str) -> Result<&str, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(contents).map_err(|e| e.to_string())?;
    Ok(contents)
}

#[tauri::command]
fn start_scan(app_handle: tauri::AppHandle, port_name: &str) -> Result<(), String> {
    let mut port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    if unsafe { SCAN_HANDLE.lock().unwrap().is_some() } {
        return Err("scan already running".to_owned());
    }
    EMIT_IDS.store(true, Ordering::SeqCst);
    start_scanner(app_handle, port).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
    serialport::available_ports().map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![copy, start_scan, get_serial_ports])
        .on_window_event(move |ev| match ev.event() {
            &WindowEvent::CloseRequested { .. } => EMIT_IDS.store(false, Ordering::SeqCst),
            &_ => {}
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
                println!("scanning");
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(_) => handle
                        .emit_all("id_scanned", String::from_utf8(serial_buf.clone()).unwrap())
                        .map_err(|e: tauri::Error| e.to_string())
                        .unwrap(),
                    Err(_) => println!("Found no data!"),
                }
            }

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
