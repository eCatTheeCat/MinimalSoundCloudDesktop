// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  #[cfg(target_os = "macos")]
  {
    tauri_plugin_deep_link::prepare("com.minimalsoundcloud.desktop");
  }
  app_lib::run();
}
