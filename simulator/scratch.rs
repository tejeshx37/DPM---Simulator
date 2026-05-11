fn main() {
    let _ = eframe::egui_wgpu::WgpuConfiguration {
        supported_backends: eframe::wgpu::Backends::VULKAN,
        ..Default::default()
    };
}
