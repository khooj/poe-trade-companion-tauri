// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod model;

use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    thread::{sleep, spawn},
    time::Duration,
};
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem};
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
struct Id {
    id: String,
}

#[derive(Clone)]
struct AppState {
    model: Arc<Mutex<model::Model>>,
}

#[tauri::command]
async fn spawn_outgoing_trade(state: tauri::State<'_, AppState>, msg: String) -> Result<(), ()> {
    println!("outgoing tradechat command: {}", msg);
    let r = state.model.lock().await.try_add(&msg).ok();
    println!("try add result: {:?}", r);
    Ok(())
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
        .setup(|app| {
            let app = app.app_handle();
            tauri::async_runtime::spawn(async move {
                let model = model2;

                loop {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    let outgoing_trades = model.lock().await.get_new_outgoing();
                    for i in outgoing_trades {
                        app.emit_all("new-outgoing-trade", i).unwrap();
                    }
                }
            });
            // let id = app.listen_global("outgoing-trade-chat", |event| {
            //     println!("got outgoing event {:?}", event.payload());
            // });
            // app.unlisten(id);
            Ok(())
        })
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![spawn_outgoing_trade])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
