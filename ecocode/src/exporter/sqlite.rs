//! SQLite exporter — persists measurement records into a SQLite database.

use rusqlite::Connection;

use crate::exporter::{Exporter, ExporterType};

/// Exports records to a SQLite database file.
///
/// The `records` table is created automatically if it does not exist.
pub struct SqliteExporter {
    db: Connection,
}

impl SqliteExporter {
    /// Creates a new SQLite exporter that writes to the given database path.
    /// The file is created if it does not already exist.
    pub fn new(db_path: String) -> SqliteExporter {
        SqliteExporter {
            db: Connection::open(db_path).unwrap(),
        }
    }
}

impl Exporter for SqliteExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Sqlite
    }

    fn add_record(&mut self, record: super::Record) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure the table exists (idempotent)
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY,
                pid INTEGER,
                timestamp INTEGER,
                cpu_usage REAL,
                cpu_energy REAL,
                gpu_usage REAL,
                gpu_energy REAL
            )",
            (),
        )?;

        // Insert the measurement record
        self.db.execute(
            "INSERT INTO records (pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy)
             VALUES (?, ?, ?, ?, ?, ?)",
            (
                record.pid,
                record.timestamp,
                record.cpu_usage,
                record.cpu_energy,
                record.gpu_usage,
                record.gpu_energy,
            ),
        )?;

        Ok(())
    }

    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
