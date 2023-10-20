// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod model;

use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread::{sleep, spawn},
    time::Duration,
};
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem};
// use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
struct Id {
    id: String,
}

#[derive(Clone)]
struct AppState {
    model: Arc<Mutex<model::Model>>,
}

#[tauri::command(async)]
fn spawn_outgoing_trade(state: tauri::State<'_, AppState>, msg: String) {
    println!("outgoing tradechat command: {}", msg);
    let r = state.model.lock().unwrap().try_add(&msg).ok();
    println!("try add result: {:?}", r);
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator);
    let tray = SystemTray::new().with_menu(tray_menu);

    let model = Arc::new(Mutex::new(model::Model::new()));
    let model2 = Arc::clone(&model);
    tauri::Builder::default()
        .manage(AppState { model })
        .setup(move |app| {
            let app = app.app_handle();
            model2.lock().unwrap().outgoing_subscribe(move |og| {
                app.emit_all("new-outgoing-trade", og).unwrap();
            });
            Ok(())
        })
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![spawn_outgoing_trade])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
