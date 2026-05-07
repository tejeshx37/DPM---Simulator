fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        // Note: In a real environment, you'd provide a path to a .ico file here.
        // For now, we set the metadata.
        res.set_icon_with_id("src/assets/simulator-icon.ico", "1");
        res.set("ProductName", "DPM Simulator");
        res.set("CompanyName", "DPM Team");
        res.set("LegalCopyright", "Copyright (c) 2026 DPM Team");
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resources: {}", e);
            std::process::exit(1);
        }
    }
}
