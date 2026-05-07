use std::env::consts::{ARCH, OS};
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GpuAccelerationMode {
    /// Apple Silicon: ARM64 + macOS
    /// Use: Neural Engine (CoreML) for inference -> MPS (Metal GPU) for training/ops
    NeuralEngineCoreMl,
    
    /// Intel Mac: x86_64 + macOS
    /// Use: Metal GPU via PyTorch MPS. Do NOT attempt Neural Engine.
    IntelMacMps,
    
    /// Linux/Windows + NVIDIA: CUDA
    /// For Linux multi-GPU: pick GPU with most free VRAM.
    Cuda { gpu_index: usize, free_vram_mb: u64 },
    
    /// Linux + AMD: ROCm
    Rocm,
    
    /// Linux + Intel iGPU: IPEX or OpenCL
    IpexOpenCl,
    
    /// Universal Fallback: CPU only
    #[default]
    CpuOnly,
}

impl std::fmt::Display for GpuAccelerationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeuralEngineCoreMl => write!(f, "Apple Silicon (CoreML + MPS)"),
            Self::IntelMacMps => write!(f, "Intel Mac (MPS)"),
            Self::Cuda { gpu_index, free_vram_mb } => 
                write!(f, "NVIDIA CUDA (GPU {}, {} MB Free)", gpu_index, free_vram_mb),
            Self::Rocm => write!(f, "AMD ROCm"),
            Self::IpexOpenCl => write!(f, "Intel iGPU (IPEX/OpenCL)"),
            Self::CpuOnly => write!(f, "CPU (Fallback)"),
        }
    }
}

pub fn detect_hardware() -> GpuAccelerationMode {
    match OS {
        "macos" => {
            if ARCH == "aarch64" {
                GpuAccelerationMode::NeuralEngineCoreMl
            } else {
                // Intel Mac: x86_64 or other non-arm flags
                GpuAccelerationMode::IntelMacMps
            }
        }
        "linux" => {
            if let Some((idx, vram)) = detect_nvidia_gpu() {
                GpuAccelerationMode::Cuda { gpu_index: idx, free_vram_mb: vram }
            } else if command_exists("rocm-smi") {
                GpuAccelerationMode::Rocm
            } else if has_intel_gpu() {
                GpuAccelerationMode::IpexOpenCl
            } else {
                GpuAccelerationMode::CpuOnly
            }
        }
        "windows" => {
            if let Some((idx, vram)) = detect_nvidia_gpu() {
                GpuAccelerationMode::Cuda { gpu_index: idx, free_vram_mb: vram }
            } else {
                GpuAccelerationMode::CpuOnly
            }
        }
        _ => GpuAccelerationMode::CpuOnly,
    }
}

fn command_exists(cmd: &str) -> bool {
    let check_cmd = if OS == "windows" { format!("{}.exe", cmd) } else { cmd.to_string() };
    Command::new(check_cmd)
        .arg("--version")
        .output()
        .is_ok()
}

fn detect_nvidia_gpu() -> Option<(usize, u64)> {
    if !command_exists("nvidia-smi") {
        return None;
    }

    // Query free memory for all GPUs
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=memory.free", "--format=csv,noheader,nounits"])
        .output();

    if let Ok(output) = output {
        let s = String::from_utf8_lossy(&output.stdout);
        let mut best_gpu = None;
        let mut max_free = 0;

        for (idx, line) in s.lines().enumerate() {
            if let Ok(free) = line.trim().parse::<u64>() {
                if free > max_free {
                    max_free = free;
                    best_gpu = Some((idx, free));
                }
            }
        }
        best_gpu.or(Some((0, 0))) // Fallback to index 0 if parsing fails
    } else {
        Some((0, 0)) // Found binary but query failed
    }
}

fn has_intel_gpu() -> bool {
    // Linux Intel check: Look for i915 or presence of Intel in clinfo
    if command_exists("clinfo") {
        let output = Command::new("clinfo").output();
        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
            return s.contains("intel");
        }
    }
    false
}
