mod model;
mod ui;

mod hardware_detect;

use eframe::NativeOptions;
use egui::ViewportBuilder;
use puffin_http::Server;
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use ui::app::App;

pub const APP_ID: &str = "dpm-simulator";
const APP_NAME: &str = concat!("DPM Simulator v", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppConfig {
    profile_server_port: Option<u16>,
}

fn main() -> eframe::Result<()> {
    pretty_env_logger::init();
    
    if let Ok(backend) = std::env::var("WGPU_BACKEND") {
        println!("[Info] Forcing graphics backend: {}", backend);
    }
    
    // --- Hardware Detection (Run once at startup) ---
    let gpu_mode = hardware_detect::detect_hardware();
    println!("--------------------------------------------------");
    println!("[Hardware Detection] Result: {}", gpu_mode);
    println!("--------------------------------------------------");
    // ------------------------------------------------
    
    let app_config = envy::prefixed("DPM_SIM_")
        .from_env::<AppConfig>()
        .unwrap_or_default();
    let _handles = app_config.profile_server_port.map(start_puffin_server);
    eframe::run_native(
        APP_NAME,
        NativeOptions {
            viewport: ViewportBuilder::default()
                .with_app_id(APP_ID)
                .with_inner_size([1280.0, 800.0])
                .with_min_inner_size([1024.0, 768.0]),
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                power_preference: eframe::wgpu::PowerPreference::LowPower, // Prefer integrated graphics to avoid NVIDIA EGL/Wayland bugs on Linux
                supported_backends: eframe::wgpu::util::backend_bits_from_env()
                    .unwrap_or(eframe::wgpu::Backends::PRIMARY),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(App::new(cc, gpu_mode)))),
    )
}

fn start_puffin_server(port: u16) -> (Child, Server) {
    puffin::set_scopes_on(true);
    let addr = format!("127.0.0.1:{port}");
    Server::new(&addr)
        .ok()
        .and_then(|server| {
            Command::new("puffin_viewer")
                .arg("--url")
                .arg(&addr)
                .spawn()
                .ok()
                .map(|child| (child, server))
        })
        .unwrap_or_else(|| panic!("Failed to start puffin http server at {addr}"))
}
