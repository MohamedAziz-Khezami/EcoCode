//! Sensor module for monitoring system energy consumption.
//!
//! This module provides functions to read CPU and GPU energy data from the system.
//! It includes:
//! - `energy`: CPU energy monitoring using Intel RAPL interface
//! - `gpu`: GPU energy monitoring using NVIDIA Management Library (NVML)

use std::path::Path;

pub mod gpu;
pub mod rapl;

//TODO: Will be changed for multi-socket support
pub const RAPL_PATH: &str = "/sys/class/powercap/intel-rapl/";
//inside this you find energy_uj of the whole cpu socket including igpu and dram
// you find intel-rapl:0:0/energy_uj for cpu cores
// you find intel-rapl:0:1/energy_uj for igpu
// you find intel-rapl:0:2/energy_uj for dram
