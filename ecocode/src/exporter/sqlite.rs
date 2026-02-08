use rusqlite::Connection;

use crate::exporter::{Exporter, ExporterType};

pub struct SqliteExporter {
    db: Connection,
}

impl SqliteExporter {
    pub fn new(db_path: String) -> SqliteExporter {
        SqliteExporter {
            db: Connection::open(db_path).unwrap(), //open or create a db
        }
    }
}

impl Exporter for SqliteExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Sqlite
    }

    fn add_record(&mut self, record: super::Record) -> Result<(), Box<dyn std::error::Error>> {
        let sql = "CREATE TABLE IF NOT EXISTS records (
            id INTEGER PRIMARY KEY,
            pid INTEGER,
            timestamp INTEGER,
            cpu_usage REAL,
            cpu_energy REAL,
            gpu_usage REAL,
            gpu_energy REAL
        )";

        self.db.execute(sql, ())?;

        let sql = "INSERT INTO records (pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy) VALUES (?, ?, ?, ?, ?, ?)";
        self.db.execute(
            sql,
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
