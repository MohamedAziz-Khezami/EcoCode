// This will be the exporter for online platform EcoCode.com to save run informations online.
// It will push the data to TimeScaleDB to be stored on the cloud.

use sqlx::{Error, postgres::PgPool};

use dotenv::dotenv;
use std::env;

use crate::exporter::{Exporter, ExporterType, Record};
use async_trait::async_trait;

pub struct OnlineExporter {
    db: PgPool,
}

impl OnlineExporter {
    pub async fn new() -> Result<OnlineExporter, Error> {
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

        let db = PgPool::connect(&db_url).await?; // that's why it needs async

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS records (
                id BIGSERIAL PRIMARY KEY,
                pid BIGINT,
                timestamp TIMESTAMPTZ,
                cpu_usage DOUBLE PRECISION,
                cpu_energy DOUBLE PRECISION,
                gpu_usage DOUBLE PRECISION,
                gpu_energy DOUBLE PRECISION
            )",
        )
        .execute(&db)
        .await?;

        Ok(OnlineExporter { db })
    }
}

#[async_trait(?Send)]
impl Exporter for OnlineExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Online
    }

    async fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "INSERT INTO records (pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy)
            VALUES ($1, $2::timestamptz, $3, $4, $5, $6)",
        )
        .bind(record.pid as i64)
        .bind(record.timestamp)
        .bind(record.cpu_usage)
        .bind(record.cpu_energy)
        .bind(record.gpu_usage)
        .bind(record.gpu_energy)
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
