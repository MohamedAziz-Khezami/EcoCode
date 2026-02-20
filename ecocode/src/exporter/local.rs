// SQLite exporter — persists measurement records into a SQLite database.

use async_trait::async_trait;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::exporter::{Exporter, ExporterType, Record};

/// Exports records to a SQLite database file.
///
/// The `records` table is created automatically if it does not exist.
pub struct SqliteExporter {
    db: SqlitePool,
}

impl SqliteExporter {
    pub async fn new() -> Result<SqliteExporter, Box<dyn std::error::Error>> {
        let db_url = "sqlite://ecocodeDB.db"; // You can change this to a custom path if needed

        let connect_options: SqliteConnectOptions = db_url.parse()?;
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options.create_if_missing(true))
            .await?;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE
            );
            CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY,
                name TEXT,
                project_id INTEGER,
                FOREIGN KEY (project_id) REFERENCES projects(id)
            );
            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY,
                pid INTEGER,
                timestamp TEXT,
                cpu_usage REAL,
                cpu_energy REAL,
                gpu_usage REAL,
                gpu_energy REAL,
                mem_usage REAL,
                mem_energy REAL,
                igpu_usage REAL,
                igpu_energy REAL
            )",
        )
        .execute(&db)
        .await?;

        Ok(SqliteExporter { db })
    }
}

#[async_trait(?Send)]
impl Exporter for SqliteExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Local
    }

    async fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "INSERT INTO records (pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy, mem_usage, mem_energy, igpu_usage, igpu_energy)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record.pid as i64)
        .bind(record.timestamp)
        .bind(record.cpu_usage)
        .bind(record.cpu_energy)
        .bind(record.gpu_usage)
        .bind(record.gpu_energy)
        .bind(record.mem_usage)
        .bind(record.mem_energy)
        .bind(record.igpu_usage)
        .bind(record.igpu_energy)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    async fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

