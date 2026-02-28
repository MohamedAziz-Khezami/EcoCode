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
// TODO: Do the run calculations after the run and save them in database table
impl SqliteExporter {
    pub async fn new() -> Result<SqliteExporter, Box<dyn std::error::Error>> {
        let db_url = "sqlite://ecocodeDB.db"; // You can change this to a custom path if needed

        let connect_options: SqliteConnectOptions = db_url.parse()?;
        let connect_options = connect_options.create_if_missing(true).foreign_keys(true);
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await?;

        sqlx::query( // Enable WAL(write-ahead log) mode for one writer and multiple readers, and create tables if they don't exist
            // Since sqlite don't have notification channels like PostgreSQL.
            "
            PRAGMA journal_mode=WAL;
            
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
                run_id INTEGER,
                pid INTEGER,
                timestamp TEXT,
                cpu_usage REAL,
                cpu_energy REAL,
                gpu_usage REAL,
                gpu_energy REAL,
                mem_usage REAL,
                mem_energy REAL,
                igpu_usage REAL,
                igpu_energy REAL,
                FOREIGN KEY (run_id) REFERENCES runs(id)
            )",
        )
        .execute(&db)
        .await?;

        Ok(SqliteExporter { db })
    }




}

    // check_if_project_exists
    // create_project
    // get_project_id
    // create_run(project_id)
    // get_run_id
    // add_record(run_id, record)

    // create a project -> get project id -> create a run with project id -> get run id -> add record with run id



#[async_trait(?Send)]
impl Exporter for SqliteExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Local
    }



     async fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
        let result = sqlx::query(
            "INSERT INTO records (run_id, pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy, mem_usage, mem_energy, igpu_usage, igpu_energy)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record.run_id)
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

        // Bump user_version so polling readers can detect DB updates.
        let version = result
            .last_insert_rowid()
            .clamp(i32::MIN as i64, i32::MAX as i64);
        let pragma = format!("PRAGMA user_version = {version}");
        sqlx::query(&pragma).execute(&self.db).await?;

        Ok(())
    }



    // async fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
    //     sqlx::query(
    //         "INSERT INTO records (pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy, mem_usage, mem_energy, igpu_usage, igpu_energy)
    //          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    //     )
    //     .bind(record.pid as i64)
    //     .bind(record.timestamp)
    //     .bind(record.cpu_usage)
    //     .bind(record.cpu_energy)
    //     .bind(record.gpu_usage)
    //     .bind(record.gpu_energy)
    //     .bind(record.mem_usage)
    //     .bind(record.mem_energy)
    //     .bind(record.igpu_usage)
    //     .bind(record.igpu_energy)
    //     .execute(&self.db)
    //     .await?;

    //     Ok(())
    // }

    async fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    async fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }


     async fn project_exists(
        &mut self,
        project_name: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {

        // Try to fetch existing id
        let existing_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM projects WHERE name = ?"
        )
        .bind(project_name)
        .fetch_optional(&self.db)
        .await?;

        if let Some(id) = existing_id {
            return Ok(id);
        }

        // If not found, insert it
        let result = sqlx::query(
            "INSERT INTO projects (name) VALUES (?)"
        )
        .bind(project_name)
        .execute(&self.db)
        .await?;

        let new_id = result.last_insert_rowid();

        Ok(new_id)
    }

     async fn create_project(&mut self, project_name: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let result = sqlx::query("INSERT INTO projects (name) VALUES (?)")
            .bind(project_name)
            .execute(&self.db)
            .await?;

        Ok(result.last_insert_rowid())
    }


     async fn create_run(&mut self, run_name: &str, project_id: i64) -> Result<i64, Box<dyn std::error::Error>> {
        let result = sqlx::query("INSERT INTO runs (name, project_id) VALUES (?, ?)")
            .bind(run_name)
            .bind(project_id)
            .execute(&self.db)
            .await?;

        Ok(result.last_insert_rowid())
    }
}
