//! GPU energy monitoring module using NVIDIA Management Library (NVML).
//!
//! This module provides functionality to monitor GPU energy consumption and utilization
//! using the NVIDIA Management Library (NVML) wrapper. It can retrieve both system-wide
//! and per-process GPU metrics.

use nvml_wrapper::Device;

/// Default NVIDIA GPU device index to monitor.
pub const DEFAULT_GPU_DEVICE_INDEX: u32 = 0;

// pub fn get_gpu_info() -> Result<(), Box<dyn std::error::Error>> {
//     let nvml = Nvml::init()?;
//     let device = nvml.device_by_index(DEFAULT_GPU_DEVICE_INDEX)?;
//     let name = device.name()?;
//     println!("\n[GPU INFO]");
//     println!("  GPU Name: {}", name);
//     Ok(())
// }

// pub fn get_gpu_power(device: &Device) -> Result<u32, Box<dyn std::error::Error>> {

//     let power = device.power_usage()?; // in milliWatt

//     Ok(power)
// }

// pub fn get_gpu_energy(device: &Device) -> Result<f64, Box<dyn std::error::Error>> {
//     //optimize: use nvml as input since reinit is expensive

//     let energy_milij = device.total_energy_consumption()?; // in mJ

//     Ok(energy_milij as f64)
// }

/// Returns the cumulative GPU energy consumption in **millijoules (mJ)**.
///
/// This value is cumulative since the last NVIDIA driver reload.
/// Take two readings and compute the delta to get interval energy.
///
/// # Errors
///
/// Returns an error if the NVML query fails (e.g. unsupported GPU, device lost).
pub fn get_gpu_energy(device: &Device) -> Result<f64, Box<dyn std::error::Error>> {
    let energy_mj = device.total_energy_consumption()?;
    Ok(energy_mj as f64)
}

/// Computes per-process GPU power and utilization for a given measurement interval.
///
/// This function combines two data sources:
/// 1. **Energy counter delta** — the difference between two cumulative `total_energy_consumption()`
///    readings (`energy_1` and `energy_2`), converted to average power over the interval.
/// 2. **Process utilization samples** — high-frequency SM utilization snapshots from the NVIDIA
///    driver's internal circular buffer, filtered for the target `pid`.
///
/// The per-process power is estimated as: `total_gpu_power_w × avg_process_utilization`.
///
/// # Arguments
///
/// * `device` — The NVML device handle.
/// * `pid` — The PID of the target process.
/// * `energy_1` — Cumulative GPU energy (mJ) at the **start** of the interval.
/// * `energy_2` — Cumulative GPU energy (mJ) at the **end** of the interval.
/// * `timestamp` — NVML timestamp (µs) from the last successful sample query.
///                  Pass `0` on the first call to fetch all buffered samples.
/// * `interval_secs` — Elapsed wall-clock time for this interval, in seconds.
///
/// # Returns
///
/// A tuple of `(process_gpu_power_w, process_gpu_util_percent, next_timestamp)`:
/// - `process_gpu_power_w` — Estimated per-process GPU power draw in Watts.
/// - `process_gpu_util_percent` — Average SM utilization for this process (0–100%).
/// - `next_timestamp` — The latest sample timestamp to pass into the next call.
pub fn get_gpu_energy_by_pid(
    device: &Device,
    pid: u32,
    energy_1: f64,
    energy_2: f64,
    timestamp: u64,
    interval_secs: f64,
) -> (f64, f64, u64) {
    // Fetch per-process utilization samples since the last timestamp.
    // On failure (e.g. process exited the GPU), return zeroes and keep
    // the same timestamp so the next call can retry the missed window.
    let stats = match device.process_utilization_stats(timestamp) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error fetching process utilization for PID {}: {}. \
                 Process may have stopped running on the GPU.",
                pid, e
            );
            return (0.0, 0.0, timestamp);
        }
    };

    // Advance the timestamp cursor to the most recent sample so the next
    // call only retrieves new samples (avoids double-counting).
    let next_timestamp = stats.iter().map(|s| s.timestamp).max().unwrap_or(timestamp); // next timestamp the gpu stats will use for next iteration

    // Filter samples for the target PID and compute average SM utilization.
    let pid_samples: Vec<_> = stats.iter().filter(|s| s.pid == pid).collect();

    let process_util = if pid_samples.is_empty() {
        0.0
    } else {
        let sum: u64 = pid_samples.iter().map(|s| s.sm_util as u64).sum();
        (sum as f64 / pid_samples.len() as f64) / 100.0 // Normalize % → 0.0–1.0
    };

    // Compute total GPU power from energy counter delta.
    // Handle the rare case where the counter wraps or resets.
    let delta_energy_mj = if energy_2 >= energy_1 {
        energy_2 - energy_1
    } else {
        eprintln!("Warning: GPU energy counter wrapped or reset.");
        energy_2 // Conservative: use only the post-reset value
    };

    // mJ → J (÷1000), then J / s = Watts
    let total_gpu_power_w = delta_energy_mj / 1000.0 / interval_secs;

    // Attribute a fraction of total GPU power to this process
    let process_gpu_power_w = total_gpu_power_w * process_util;

    (process_gpu_power_w, process_util * 100.0, next_timestamp)
}
