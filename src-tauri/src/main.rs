// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod model;
mod settings;

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
use tauri::{Manager, PhysicalPosition, State};
use log::{debug, error};
// use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
struct Id {
    id: String,
}

pub struct FileLineReader {
    byte_count: u64,
    sock: Option<File>,
    model: Arc<Mutex<model::Model>>,
}

impl FileLineReader {
    pub fn new(model: Arc<Mutex<model::Model>>, fp: &str) -> Self {
        let s = File::open(fp).ok();
        let size;
        if s.is_some() {
            s.as_ref()
                .unwrap()
                .seek(SeekFrom::End(0))
                .expect("can't seek to file end");
            size = s
                .as_ref()
                .unwrap()
                .metadata()
                .expect("can't get file metadata")
                .len();
        } else {
            size = 0;
        }

        FileLineReader {
            byte_count: size,
            sock: s,
            model,
        }
    }

    pub fn process_new_content(&mut self) {
        if self.sock.is_none() {
            return;
        }

        let new_size = self
            .sock
            .as_ref()
            .unwrap()
            .metadata()
            .expect("can't get file metadata in new content")
            .len();
        if new_size <= self.byte_count {
            // probably file truncated or nothing new added, just update byte_count and wait for new call
            self.byte_count = new_size;
            return;
        }
        self.byte_count = new_size;

        let mut contents = String::new();
        self.sock
            .as_ref()
            .unwrap()
            .read_to_string(&mut contents)
            .expect("can't read new content");
        let mut lock = self.model.lock().unwrap();
        contents.lines().for_each(|l| {
            // add log
            let _ = lock.try_add(l);
        });
    }
}

fn subscribe_new_trades(
    app: tauri::AppHandle,
    model: Arc<Mutex<model::Model>>,
    mut file_line_reader: FileLineReader,
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
                Ok(_) => file_line_reader.process_new_content(),
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
) -> (FileLineReader, Debouncer<INotifyWatcher>) {
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

    let file_line_reader = FileLineReader::new(Arc::clone(&model), &stx.logpath);

    app.manage(AppState {
        stx: Mutex::new(stx),
        cfg_path: cfg_path.to_string(),
    });

    (file_line_reader, debouncer)
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

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(move |app| {
            let (file_line_reader, debouncer) = init_config(app, tx, Arc::clone(&model));
            subscribe_new_trades(app.app_handle(), model, file_line_reader, rx);
            let _ = app.manage(debouncer);
            Ok(())
        })
        // .system_tray(tray)
        .invoke_handler(tauri::generate_handler![update_position_stx, update_logpath_stx])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/*Object

event: "tauri://move"

id: 4151762018

payload: r {type: "Physical", x: 0, y: 61}

windowLabel: "incoming"

Object Prototype */
