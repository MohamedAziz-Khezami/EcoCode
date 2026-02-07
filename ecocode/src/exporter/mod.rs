pub mod terminal;
pub mod csv;
pub mod json;

/// Represents a single measurement record
#[derive(Clone, Debug)]
pub struct Record {
    pub id: u32,
    pub pid: u32,
    pub timestamp: u64,  // milliseconds since epoch
    pub cpu_usage: f64,  // percentage (0-100)
    pub gpu_usage: f64,  // percentage (0-100)
    pub cpu_energy: f64, // watts
    pub gpu_energy: f64, // watts
}

impl Record {
    pub fn new(
        id: u32,
        pid: u32,
        timestamp: u64,
        cpu_usage: f64,
        gpu_usage: f64,
        cpu_energy: f64,
        gpu_energy: f64,
    ) -> Record {
        Record {
            id,
            pid,
            timestamp,
            cpu_usage,
            gpu_usage,
            cpu_energy,
            gpu_energy,
        }
    }
    pub fn to_vec(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.pid.to_string(),
            self.timestamp.to_string(),
            self.cpu_usage.to_string(),
            self.gpu_usage.to_string(),
            self.cpu_energy.to_string(),
            self.gpu_energy.to_string(),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExporterType {
    Terminal,
    Csv,
    Json,
}

/// Trait for different export formats
pub trait Exporter {
    fn exporter_type(&self) -> ExporterType; // Returns "terminal", "csv", "json", etc.
    fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>>;
    fn export(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn export_line(&self) -> Result<(), Box<dyn std::error::Error>>;
}
