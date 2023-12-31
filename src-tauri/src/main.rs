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
use std::{
    fs::File,
    path::Path,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{
    CustomMenuItem, Manager, PhysicalPosition, State, SystemTray, SystemTrayEvent, SystemTrayMenu,
};

struct AppState {
    stx: Mutex<settings::Settings>,
    cfg_path: String,
    file_line_reader: Mutex<Option<FileLineReader<File>>>,
    model: Arc<Mutex<model::Model>>,
    debouncer: Mutex<Debouncer<RecommendedWatcher>>,
}

fn subscribe_new_trades(
    app: tauri::AppHandle,
    model: Arc<Mutex<model::Model>>,
    rx: Receiver<Result<Vec<DebouncedEvent>>>,
) {
    let apph = app.app_handle();
    model.lock().unwrap().outgoing_subscribe(move |og| {
        debug!("trigger new outgoing trade: {:?}", og);
        apph.emit_all("new-outgoing-trade", og).unwrap();
    });
    let apph = app.app_handle();
    model.lock().unwrap().incoming_subscribe(move |ig| {
        debug!("trigger new incoming trade: {:?}", ig);
        apph.emit_all("new-incoming-trade", ig).unwrap();
    });

    let apph = app.app_handle();
    tauri::async_runtime::spawn(async move {
        for res in rx {
            match res {
                Ok(_) => {
                    debug!("start processing notify event");
                    let appstate = apph.state::<AppState>();
                    let mut flr = if let Ok(r) = appstate.file_line_reader.lock() {
                        r
                    } else {
                        return;
                    };
                    if flr.is_none() {
                        debug!("file_line_reader not initialized");
                        return;
                    }

                    let r = flr.as_mut().unwrap().process_new_content();
                    debug!("end processing notify event: {:?}", r);
                }
                Err(e) => panic!("file notify events fail: {:?}", e),
            }
        }
    });
}

fn init_config(
    app: &mut tauri::App,
    tx: Sender<Result<Vec<DebouncedEvent>>>,
    model: Arc<Mutex<model::Model>>,
) {
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

    let file_line_reader = FileLineReader::<File>::with_file(Arc::clone(&model), &stx.logpath).ok();

    app.manage(AppState {
        stx: Mutex::new(stx),
        cfg_path: cfg_path.to_string(),
        file_line_reader: Mutex::new(file_line_reader),
        model,
        debouncer: Mutex::new(debouncer),
    });
}

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
    let oldpath = std::mem::replace(&mut s.logpath, logpath);
    let r = s.save(&stx.cfg_path);
    if r.is_err() {
        error!("can't save stx: {}", r.unwrap_err());
    }
    debug!("called update_logpath_stx {}", s.logpath);
    let file_line_reader =
        FileLineReader::<File>::with_file(Arc::clone(&stx.model), &s.logpath).ok();
    *stx.file_line_reader.lock().unwrap() = file_line_reader;
    let mut db = stx.debouncer.lock().unwrap();
    db.watcher().unwatch(Path::new(&oldpath)).unwrap();
    db.watcher()
        .watch(Path::new(&s.logpath), RecursiveMode::NonRecursive)
        .unwrap();
}

#[tauri::command]
fn trade_close(stx: State<AppState>, id: String) {
    let mut m = stx.model.lock().unwrap();
    m.remove_trade(id);
}

fn system_tray_event_handler(app: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => {
                app.exit(0);
            }
            "settings" => {
                let window = app.get_window("settings").unwrap();
                window.show().unwrap();
            }
            _ => {}
        },
        _ => {}
    }
}

fn main() {
    let (tx, rx) = channel();

    let model = Arc::new(Mutex::new(model::Model::new()));

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let tray_menu = SystemTrayMenu::new().add_item(quit).add_item(settings);
    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(move |app| {
            init_config(app, tx, Arc::clone(&model));
            subscribe_new_trades(app.app_handle(), model, rx);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            update_position_stx,
            update_logpath_stx,
            trade_close,
        ])
        .system_tray(tray)
        .on_system_tray_event(system_tray_event_handler)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        })
}
