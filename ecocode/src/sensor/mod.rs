//! Sensor module for monitoring system energy consumption.
//!
//! This module provides functions to read CPU and GPU energy data from the system:
//! - [`cpu`] — CPU energy monitoring via the Intel RAPL sysfs interface
//! - [`gpu`] — GPU energy and per-process utilization via NVIDIA NVML

pub mod cpu;
pub mod gpu;

/// Path to the Intel RAPL energy counter (microjoules since boot).
///
/// Requires read permission — run `sudo chmod +r` on this file before use.
pub const RAPL_PATH: &str = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";
