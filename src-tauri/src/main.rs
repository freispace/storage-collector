// Suppress the Windows console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    freispace_storage_collector_lib::run();
}
