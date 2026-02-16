//! Exporter module — defines the output backends for measurement records.
//!
//! Available exporters:
//! - [`terminal`] — pretty-printed table to stdout
//! - [`csv`] — comma-separated values file
//! - [`json`] — JSON array file
//! - [`sqlite`] — SQLite database
//! - [`prometheus`] — Prometheus metrics endpoint via HTTP

pub mod csv;
pub mod json;
pub mod prometheus;
pub mod sqlite;
pub mod terminal;

use serde::{Deserialize, Serialize};

/// A single measurement record captured during one interval.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    /// Monotonically increasing iteration counter.
    pub id: u32,
    /// PID of the monitored process.
    pub pid: u32,
    /// Unix timestamp in milliseconds.
    pub timestamp: i64,
    /// Normalized CPU usage as a percentage (0–100%).
    pub cpu_usage: f64,
    /// Per-process CPU power consumption in Watts.
    pub cpu_energy: f64,
    /// Per-process GPU SM utilization as a percentage (0–100%).
    pub gpu_usage: f64,
    /// Per-process GPU power consumption in Watts.
    pub gpu_energy: f64,
}

impl Record {
    /// Creates a new measurement record.
    pub fn new(
        id: u32,
        pid: u32,
        timestamp: i64,
        cpu_usage: f64,
        cpu_energy: f64,
        gpu_usage: f64,
        gpu_energy: f64,
    ) -> Record {
        Record {
            id,
            pid,
            timestamp,
            cpu_usage,
            cpu_energy,
            gpu_usage,
            gpu_energy,
        }
    }

    /// Converts all fields to strings for tabular output (CSV, terminal).
    pub fn to_vec(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.pid.to_string(),
            self.timestamp.to_string(),
            self.cpu_usage.to_string(),
            self.cpu_energy.to_string(),
            self.gpu_usage.to_string(),
            self.gpu_energy.to_string(),
        ]
    }
}

/// Supported exporter backends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExporterType {
    Terminal,
    Csv,
    Json,
    Sqlite,
    Prometheus,
}

/// Trait that all export backends must implement.
pub trait Exporter {
    /// Returns the type of this exporter.
    fn exporter_type(&self) -> ExporterType;

    /// Appends a measurement record to the exporter's buffer or output.
    fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>>;

    /// Finalizes and flushes all buffered data (called once at shutdown).
    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Incrementally outputs the most recent record (for live streaming).
    fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
