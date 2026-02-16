//! Terminal exporter — prints measurement records as a formatted table to stdout.

use crate::exporter::{Exporter, ExporterType, Record};

/// Exports records to the terminal as a formatted table.
///
/// Records are both accumulated for a final summary (`export()`)
/// and streamed line-by-line during measurement (`export_line()`).
pub struct TerminalExporter {
    records: Vec<Record>,
    first_record: bool,
}

impl TerminalExporter {
    /// Creates a new terminal exporter.
    pub fn new() -> TerminalExporter {
        TerminalExporter {
            records: Vec::new(),
            first_record: true,
        }
    }
}

impl Exporter for TerminalExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Terminal
    }

    fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
        self.records.push(record);
        Ok(())
    }

    /// Prints all accumulated records as a formatted table.
    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n[TERMINAL EXPORT]");
        println!("\n{}", "=".repeat(80));
        println!(
            "{:<5} {:<8} {:<12} {:<10} {:<10} {:<12} {:<12}",
            "ID", "PID", "Timestamp", "CPU%", "CPU(W)", "GPU%", "GPU(W)"
        );
        println!("{}", "-".repeat(80));

        for record in &self.records {
            println!(
                "{:<5} {:<8} {:<12} {:<10.2} {:<10.2} {:<12.3} {:<12.3}",
                record.id,
                record.pid,
                record.timestamp,
                record.cpu_usage,
                record.cpu_energy,
                record.gpu_usage,
                record.gpu_energy
            );
        }
        println!("{}\n", "=".repeat(80));

        Ok(())
    }

    /// Prints the most recently added record as a single table row.
    fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let r = self.records.last().unwrap();

        // Print the table header once before the first row
        if self.first_record {
            println!("\n{}", "=".repeat(80));
            println!(
                "{:<5} {:<8} {:<12} {:<10} {:<10} {:<12} {:<12}",
                "ID", "PID", "Timestamp", "CPU%", "CPU(W)", "GPU%", "GPU(W)"
            );
            println!("{}", "-".repeat(80));
            self.first_record = false;
        }

        println!(
            "{:<5} {:<8} {:<12} {:<10.2} {:<10.2} {:<12.3} {:<12.3}",
            r.id, r.pid, r.timestamp, r.cpu_usage, r.cpu_energy, r.gpu_usage, r.gpu_energy
        );

        Ok(())
    }
}
