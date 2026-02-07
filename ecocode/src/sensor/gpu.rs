//! GPU energy monitoring module using NVIDIA Management Library (NVML).
//!
//! This module provides functionality to monitor GPU energy consumption and utilization
//! using the NVIDIA Management Library (NVML) wrapper. It can retrieve both system-wide
//! and per-process GPU metrics.

use nvml_wrapper::{Device, Nvml};

/// Default NVIDIA GPU device index to monitor.
pub const DEFAULT_GPU_DEVICE_INDEX: u32 = 0;

pub fn get_gpu_info() -> Result<(), Box<dyn std::error::Error>> {
    let nvml = Nvml::init()?;
    let device = nvml.device_by_index(DEFAULT_GPU_DEVICE_INDEX)?;
    let name = device.name()?;
    println!("\n[GPU INFO]");
    println!("  GPU Name: {}", name);
    Ok(())
}

pub fn get_gpu_power() -> Result<u32, Box<dyn std::error::Error>> {
    let nvml = Nvml::init()?;
    let device = nvml.device_by_index(DEFAULT_GPU_DEVICE_INDEX)?;
    let power = device.power_usage()?; // in mW

    Ok(power)
}

pub fn get_gpu_energy(device: &Device) -> Result<f64, Box<dyn std::error::Error>> {
    //optimize: use nvml as input since reinit is expensive

    let energy_milij = device.total_energy_consumption()?; // in mJ

    Ok(energy_milij as f64)
}

pub fn get_gpu_energy_by_pid(
    device: &Device,
    pid: u32,
    energy_1: f64,
    energy_2: f64,
    interval_secs: u64,
) -> (f64, f64) {
    // Attempt to get the accounting stats for the process
    let stats = match device.accounting_stats_for(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error fetching accounting stats for PID {}: {}", pid, e);
            return (0.0, 0.0); // Return 0 if stats cannot be retrieved
        }
    };

    // Attempt to get GPU utilization
    let process_util = match stats.gpu_utilization {
        Some(util) => util as f64 / 100.0,
        None => {
            eprintln!("GPU utilization data is not available for PID {}.", pid);
            return (0.0, 0.0); // Return 0 if GPU utilization is unavailable
        }
    };

    let delta_energy_mj = (energy_2 - energy_1) as f64;

    let total_gpu_power_w = delta_energy_mj / 1000.0 / interval_secs as f64;

    let process_gpu_power_w = total_gpu_power_w * process_util;

    (process_gpu_power_w, process_util)
}
