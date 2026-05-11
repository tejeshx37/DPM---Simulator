# DPM Simulator - Linux Guide

This document provides instructions for setting up and running the DPM Simulator on Linux.

## Quick Setup (Debian/Ubuntu)

We provide a setup script to install all required libraries automatically:

```bash
sudo ./scripts/setup_linux.sh
```

## Manual Installation

If you prefer to install dependencies manually instead of using the script, run the following command to install all physics and GUI libraries:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libcgal-dev libboost-all-dev libgmp-dev libmpfr-dev libgtk-3-dev libasound2-dev libvulkan-dev libwayland-dev libxkbcommon-dev cmake
```

## Building from Source

Once dependencies are installed, you can build the simulator using Cargo:

```bash
cargo build --release -p simulator
```

The executable will be located at `target/release/simulator`.

## Graphics Configuration

The simulator uses `wgpu` for rendering. On Linux, you can control the graphics backend using environment variables if you encounter issues:

- **Force Vulkan**: `WGPU_BACKEND=vulkan ./target/release/simulator`
- **Force GLES**: `WGPU_BACKEND=gl ./target/release/simulator`
- **Disable GPU (CPU Fallback)**: `WGPU_BACKEND=cpu ./target/release/simulator`

### Wayland Support
If you are using Wayland and the window decorations look incorrect, try forcing the X11 backend for the GUI:
```bash
WINIT_UNIX_BACKEND=x11 ./target/release/simulator
```

## Packaged Distribution (.deb)

If you have a `.deb` file, you can install it using:
```bash
sudo apt install ./dpm-simulator.deb
```
**All system dependencies will be installed automatically by the package manager.**
