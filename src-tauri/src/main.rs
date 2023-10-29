// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_line_reader;
mod model;
mod settings;
#[cfg(test)]
mod test_utilities;

use file_line_reader::FileLineReader;
use log::{debug, error};
use notify_debouncer_mini::{
    new_debouncer_opt, notify::*, Config as NotifyDebouncerConfig, DebouncedEvent, Debouncer,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{
    CustomMenuItem, Manager, PhysicalPosition, State, SystemTray, SystemTrayMenu,
    SystemTrayMenuItem,
};
// use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
struct Id {
    id: String,
}

fn subscribe_new_trades(
    app: tauri::AppHandle,
    model: Arc<Mutex<model::Model>>,
    mut file_line_reader: FileLineReader<File>,
    rx: Receiver<Result<Vec<DebouncedEvent>>>,
) {
    let apph = app.app_handle();
    model.lock().unwrap().outgoing_subscribe(move |og| {
        apph.emit_all("new-outgoing-trade", og).unwrap();
    });
    let apph = app.app_handle();
    model.lock().unwrap().incoming_subscribe(move |ig| {
        apph.emit_all("new-incoming-trade", ig).unwrap();
    });

    tauri::async_runtime::spawn(async move {
        for res in rx {
            match res {
                Ok(_) => {
                    let _ = file_line_reader.process_new_content();
                }
                Err(e) => panic!("file notify events fail: {:?}", e),
            }
        }
    });
}

struct AppState {
    stx: Mutex<settings::Settings>,
    cfg_path: String,
}

fn init_config(
    app: &mut tauri::App,
    tx: Sender<Result<Vec<DebouncedEvent>>>,
    model: Arc<Mutex<model::Model>>,
) -> (FileLineReader<File>, Debouncer<RecommendedWatcher>) {
    let base = app.path_resolver().app_config_dir().unwrap_or(
        app.path_resolver()
            .app_data_dir()
            .expect("can't get app data dir"),
    );
    let app_data = base.join("config.json");
    let cfg_path = app_data.as_os_str().to_str().unwrap();

    let stx = settings::Settings::new(cfg_path).unwrap_or(settings::Settings {
        logpath: String::new(),
        ..Default::default()
    });

    app.get_window("incoming")
        .unwrap()
        .set_position(PhysicalPosition::new(
            stx.incoming_position.0,
            stx.incoming_position.1,
        ))
        .expect("can't set incoming window position");

    app.get_window("outgoing")
        .unwrap()
        .set_position(PhysicalPosition::new(
            stx.outgoing_position.0,
            stx.outgoing_position.1,
        ))
        .expect("can't set outgoing window position");

    let debouncer_config = NotifyDebouncerConfig::default()
        .with_batch_mode(true)
        .with_timeout(Duration::from_millis(300));
    let mut debouncer: Debouncer<RecommendedWatcher> =
        new_debouncer_opt(debouncer_config, tx).unwrap();
    debouncer
        .watcher()
        .watch(Path::new(&stx.logpath), RecursiveMode::NonRecursive)
        .unwrap();

    let file_line_reader = FileLineReader::<File>::with_file(Arc::clone(&model), &stx.logpath).unwrap();

    app.manage(AppState {
        stx: Mutex::new(stx),
        cfg_path: cfg_path.to_string(),
    });

    (file_line_reader, debouncer)
}

fn setup_systemtray(app: &mut tauri::App) {}

#[tauri::command]
fn update_position_stx(stx: State<AppState>, position: (i32, i32), window: String) {
    let mut s = stx.stx.lock().unwrap();
    if window == "incoming" {
        s.incoming_position = position;
    } else {
        s.outgoing_position = position;
    }
    let r = s.save(&stx.cfg_path);
    if r.is_err() {
        error!("can't save stx: {}", r.unwrap_err());
    }
    debug!("called update_position_stx {:?} {}", position, window);
}

#[tauri::command]
fn update_logpath_stx(stx: State<AppState>, logpath: String) {
    let mut s = stx.stx.lock().unwrap();
    s.logpath = logpath;
    let r = s.save(&stx.cfg_path);
    if r.is_err() {
        error!("can't save stx: {}", r.unwrap_err());
    }
    debug!("called update_logpath_stx {}", s.logpath);
}

fn main() {
    let (tx, rx) = channel();

    let model = Arc::new(Mutex::new(model::Model::new()));

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(quit);
    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(move |app| {
            let (file_line_reader, debouncer) = init_config(app, tx, Arc::clone(&model));
            subscribe_new_trades(app.app_handle(), model, file_line_reader, rx);
            let _ = app.manage(debouncer);
            Ok(())
        })
        // .system_tray(tray)
        .invoke_handler(tauri::generate_handler![
            update_position_stx,
            update_logpath_stx
        ])
        .system_tray(tray)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
