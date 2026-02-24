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
pub mod local;
pub mod online;
pub mod terminal;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Represents a single measurement record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: u32,
    pub run_id: i64,
    pub pid: u32,
    pub timestamp: String, // RFC3339 UTC timestamp
    pub cpu_usage: f64,    // percentage (0-100)
    pub cpu_energy: f64,   // watts
    pub gpu_usage: f64,    // percentage (0-100)
    pub gpu_energy: f64,   // watts
    pub mem_usage: f64,    // percentage (0-100) Dram
    pub mem_energy: f64,   // watts
    pub igpu_usage: f64,   //igpu
    pub igpu_energy: f64,
}

impl Record {
    pub fn new(
        id: u32,
        run_id: i64,
        pid: u32,
        timestamp: String,
        cpu_usage: f64,
        cpu_energy: f64,
        gpu_usage: f64,
        gpu_energy: f64,
        mem_usage: Option<f64>,
        mem_energy: Option<f64>,
        igpu_usage: Option<f64>,
        igpu_energy: Option<f64>,
    ) -> Record {
        Record {
            id,
            run_id,
            pid,
            timestamp,
            cpu_usage,
            cpu_energy,
            gpu_usage,
            gpu_energy,
            mem_usage: mem_usage.unwrap_or(0.0),
            mem_energy: mem_energy.unwrap_or(0.0),
            igpu_usage: igpu_usage.unwrap_or(0.0),
            igpu_energy: igpu_energy.unwrap_or(0.0),
        }
    }
    pub fn to_vec(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.run_id.to_string(),
            self.pid.to_string(),
            self.timestamp.clone(),
            self.cpu_usage.to_string(),
            self.cpu_energy.to_string(),
            self.gpu_usage.to_string(),
            self.gpu_energy.to_string(),
            self.mem_usage.to_string(),
            self.mem_energy.to_string(),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExporterType {
    Terminal,
    Csv,
    Json,
    Local,
    Online,
    Prometheus,
}

/// Trait for different export formats
#[async_trait(?Send)]
pub trait Exporter {
    fn exporter_type(&self) -> ExporterType; // Returns "terminal", "csv", "json", etc.
    async fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>>;
    async fn export(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn project_exists(&mut self, project_name: &str) -> Result<i64, Box<dyn std::error::Error>>;
    async fn create_project(&mut self, project_name: &str) -> Result<i64, Box<dyn std::error::Error>>;
    async fn create_run(&mut self, run_name: &str, project_id: i64) -> Result<i64, Box<dyn std::error::Error>>;
}
