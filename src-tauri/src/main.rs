// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod model;

use notify_debouncer_mini::{
    new_debouncer_opt, notify::*, Config as NotifyDebouncerConfig, Debouncer,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::{mpsc::channel, Arc, Mutex},
    time::Duration,
};
use tauri::{CustomMenuItem, Manager};
// use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
struct Id {
    id: String,
}

pub struct FileLineReader {
    byte_count: u64,
    sock: File,
    model: Arc<Mutex<model::Model>>,
}

impl FileLineReader {
    pub fn new(model: Arc<Mutex<model::Model>>, fp: &str) -> Self {
        let mut s = File::open(fp).expect("can't open file to watch");
        s.seek(SeekFrom::End(0)).expect("can't seek to file end");
        let size = s.metadata().expect("can't get file metadata").len();
        FileLineReader {
            byte_count: size,
            sock: s,
            model,
        }
    }

    pub fn process_new_content(&mut self) {
        let new_size = self
            .sock
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
            .read_to_string(&mut contents)
            .expect("can't read new content");
        let mut lock = self.model.lock().unwrap();
        contents.lines().for_each(|l| {
            // add log
            let _ = lock.try_add(l);
        });
    }
}

fn main() {
    let (tx, rx) = channel();
    let debouncer_config = NotifyDebouncerConfig::default()
        .with_batch_mode(true)
        .with_timeout(Duration::from_millis(300));
    let mut debouncer: Debouncer<RecommendedWatcher> =
        new_debouncer_opt(debouncer_config, tx).unwrap();

    // let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    // let tray_menu = SystemTrayMenu::new()
    //     .add_item(quit)
    //     .add_native_item(SystemTrayMenuItem::Separator);
    // let tray = SystemTray::new().with_menu(tray_menu);

    let model = Arc::new(Mutex::new(model::Model::new()));
    debouncer
        .watcher()
        .watch(Path::new("poe_log_mock.txt"), RecursiveMode::NonRecursive)
        .unwrap();

    let mut file_line_reader = FileLineReader::new(Arc::clone(&model), "poe_log_mock.txt");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(move |app| {
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
            Ok(())
        })
        // .system_tray(tray)
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
