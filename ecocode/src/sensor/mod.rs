//! Sensor module for monitoring system energy consumption.
//!
//! This module provides functions to read CPU and GPU energy data from the system.
//! It includes:
//! - `energy`: CPU energy monitoring using Intel RAPL interface
//! - `gpu`: GPU energy monitoring using NVIDIA Management Library (NVML)

pub mod cpu;
pub mod gpu;

pub const RAPL_PATH: &str = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";
