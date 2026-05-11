# DPM Simulator

**DPM Simulator** (Continuum Interaction Particle Dynamics Simulator) is a high-performance particle physics and mesh simulation engine with a native graphical user interface. It leverages hardware-accelerated computation and robust computational geometry libraries to simulate particle interactions and structural stresses.

## Features

- **High-Performance Physics Engine**: Built to handle complex nodal forces, body force modeling (e.g., gravity), and continuum interactions.
- **Hardware-Accelerated Rendering & Compute**: Uses `wgpu` to leverage the host's GPU (Vulkan, Metal, DirectX 12) for rendering and compute tasks, with a fallback to CPU. Includes automatic hardware detection on startup.
- **Advanced Computational Geometry**: Integrates the CGAL (Computational Geometry Algorithms Library) for reliable mesh and spatial operations.
- **Interactive GUI & Analytics Dashboard**: Built natively with `egui`. Includes real-time viewport rendering, stress plots, dynamic camera tracking, and performance profiling via `puffin`.
- **Cross-Platform Availability**: Automated CI/CD pipelines generate platform-specific release binaries (`.deb` for Linux, `.dmg` for macOS, `.nsis` for Windows) via `cargo-packager`.

## Project Structure

The project is structured as a Cargo workspace with several key crates:

- **`simulator/`**: The main application and GUI entry point. Manages the window (`eframe`), user interface, analytics dashboard, viewport, and overall application state.
- **`cpd/` & `cpd-wgpu/`**: Core modules for compute. Includes definitions and implementations for the physics interactions, body forces, and GPU compute shaders.
- **`mesh/`**: Utilities for handling simulation meshes and object geometries.
- **`cgal/` & `cgal-sys/`**: Rust CXX bindings to the C++ CGAL library.
- **`nalgebra-ext/`**: Extensions to the `nalgebra` crate for mathematical constructs.
- **`function/`**: General-purpose function and utility structures.

## System Requirements & Dependencies

To build the simulator from source, you need a Rust toolchain (`cargo`) and the following system-level dependencies:

### Core / Physics Dependencies
- CGAL (`libcgal-dev`)
- Boost (`libboost-all-dev`)
- GMP (`libgmp-dev`)
- MPFR (`libmpfr-dev`)

### Graphics & GUI Dependencies (Linux)
- GTK 3 (`libgtk-3-dev`)
- ALSA (`libasound2-dev`)
- Vulkan (`libvulkan-dev`) (Recommended for GPU acceleration)
- Wayland / XKB (`libwayland-dev`, `libxkbcommon-dev`)

> *Note: For comprehensive Linux-specific setup instructions, please see [README_LINUX.md](./README_LINUX.md).*

## Building and Running

Ensure that you have all necessary system libraries installed. Then, use Cargo to build and run the simulator.

### Running the Simulator

To run the simulator natively during development:

```bash
cargo run --release -p simulator
```

### Profiling
The simulator includes support for the `puffin` profiler. To run with profiling enabled, you can provide the `DPM_SIM_PROFILE_SERVER_PORT` environment variable (e.g., `8585`), which will start the puffin HTTP server and launch the `puffin_viewer` tool if installed.

## Graphics Configuration

If you experience rendering issues, you can explicitly set the `wgpu` backend via environment variables:

- **Force Vulkan (Linux/Windows)**: `WGPU_BACKEND=vulkan cargo run --release`
- **Force Metal (macOS)**: `WGPU_BACKEND=metal cargo run --release`
- **Disable GPU (CPU Fallback)**: `WGPU_BACKEND=cpu cargo run --release`

## Packaging and Distribution

The project uses `cargo-packager` to create distribution bundles. The CI pipeline automatically handles these builds for multiple platforms.

Supported formats include:
- `.dmg` (macOS 10.15+)
- `nsis` (Windows)
- `.deb` (Debian/Ubuntu Linux)

To package manually (assuming `cargo-packager` is installed):
```bash
cargo packager -p simulator
```

## License

Please see the [LICENSE](./LICENSE) file for more information.
