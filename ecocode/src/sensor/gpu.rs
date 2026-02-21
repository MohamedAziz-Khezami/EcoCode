//! GPU energy monitoring module using NVIDIA Management Library (NVML).
//!
//! This module provides functionality to monitor GPU energy consumption and utilization
//! using the NVIDIA Management Library (NVML) wrapper. It can retrieve both system-wide
//! and per-process GPU metrics.

use nvml_wrapper::{Device, Nvml};

/// Default NVIDIA GPU device index to monitor.
pub const DEFAULT_GPU_DEVICE_INDEX: u32 = 0;

pub fn _get_gpu_info() -> Result<(), Box<dyn std::error::Error>> {
    let nvml = Nvml::init()?;
    let device = nvml.device_by_index(DEFAULT_GPU_DEVICE_INDEX)?;
    let name = device.name()?;
    println!("\n[GPU INFO]");
    println!("  GPU Name: {}", name);
    Ok(())
}

pub fn _get_gpu_power() -> Result<u32, Box<dyn std::error::Error>> {
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

// 1. What are these samples?
// The NVIDIA driver doesn't just keep a single "current utilization" number for each process.
// Instead, it has an internal circular buffer where it records "snapshots" of activity at high frequency (usually every few milliseconds).
// When you call device.process_utilization_stats(timestamp), you are asking the driver: "Give me every snapshot you've recorded since this specific point in time."
// 2. What does the function actually return?
// It returns a Vec<ProcessUtilizationSample>. If your code sleeps for 1 second and the driver takes a snapshot every 100ms, that vector will contain about 10 samples for your process.
// Each ProcessUtilizationSample looks like this:
// pid
// : The process ID.
// timestamp: The exact microsecond this specific snapshot was taken.
// sm_util: The GPU utilization (%) during that tiny slice of time.
// mem_util: Video memory utilization during that slice.
// enc_util / dec_util: Video encoder/decoder activity.

pub fn get_gpu_energy_by_pid(
    device: &Device,
    pid: u32,
    energy_1: f64,
    energy_2: f64,
    timestamp: u64,
    interval_secs: f64,
) -> (f64, f64, u64) {
    let stats = match device.process_utilization_stats(timestamp) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error fetching process utilization for PID {}: {}, maybe it stopped running on the GPU.",
                pid, e
            );
            return (0.0, 0.0, timestamp); // Return current timestamp to retry next time
        }
    };

    //If the function fails once and we return the same timestamp, the next successful call will return samples for a 2-second window (the failed 1s + the current 1s).

    // However, in
    // main.rs
    // we are still calculating total_gpu_power_w using only the 1-second delta of the physical hardware energy counters (energy_2 - energy_1).

    // The result: You are applying the average utilization of the last 2 seconds to the total power consumed in the last 1 second. It's an approximation that says: "I don't know exactly what happened in the last second, so I'll use the average of everything I've seen since my last successful check."

    // Is there a better way?

    // If you want to perfectly "give up" on the failed period and stay strictly within the current 1-second window, you could return the current system timestamp instead.

    // But there's a risk: If you do that, the activity that happened during that 1-second failure is permanently lost. By retrying with the old timestamp, we at least allow the driver to give us those samples later, ensuring our average utilization is informed by the "missing" time.

    // Get the latest timestamp from samples or keep the current one
    let next_timestamp = stats.iter().map(|s| s.timestamp).max().unwrap_or(timestamp);

    // println!("{:?}", stats);

    // Filter samples for this PID
    let pid_samples: Vec<_> = stats.iter().filter(|s| s.pid == pid).collect();

    let process_util = if pid_samples.is_empty() {
        0.0
    } else {
        // Average the utilization across all samples in this period
        let sum: u64 = pid_samples.iter().map(|s| s.sm_util as u64).sum();
        (sum as f64 / pid_samples.len() as f64) / 100.0
    };

    let delta_energy_mj = if energy_2 >= energy_1 {
        energy_2 - energy_1
    } else {
        // Handle wraparound. NVML total energy is usually 64-bit, but some drivers
        // might report 32-bit values or reset.
        println!("Warning: GPU energy counter wrapped or reset.");
        energy_2 // Minimum delta we can assume is the new value
    };
    //The code detects 200 < 1,000,000 and enters the else block:

    // Delta: 200 mJ (It assumes the 200 mJ used since the reset is the safest delta).
    // Power: 200 / 1000 / 1.0 = 0.2 Watts.
    // EcoCode UI: Displays a very low power usage for that single second, keeping your data clean and physically accurate.

    let total_gpu_power_w = delta_energy_mj / 1000.0 / interval_secs;
    let process_gpu_power_w = total_gpu_power_w * process_util;

    (
        process_gpu_power_w,
        (process_util * 100.0).round_ties_even(),
        next_timestamp,
    )
}

//TODO: Make it modular
