// This will be the exporter for online platform EcoCode.com to save run informations online.
// It will push the data to TimeScaleDB to be stored on the cloud.

use sqlx::{postgres::PgPool, Error};

use async_trait::async_trait;
use crate::exporter::{Exporter, ExporterType, Record};

pub struct OnlineExporter {
    db: PgPool
}


impl OnlineExporter {
   pub async fn new(db_url: String) -> Result<OnlineExporter, Error> {
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

        // Migrate legacy schemas that stored epoch millis as INTEGER/BIGINT.
        sqlx::query(
            r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = current_schema()
                      AND table_name = 'records'
                      AND column_name = 'timestamp'
                      AND data_type IN ('integer', 'bigint')
                ) THEN
                    ALTER TABLE records
                    ALTER COLUMN timestamp TYPE TIMESTAMPTZ
                    USING to_timestamp(timestamp::double precision / 1000.0);
                END IF;
            END
            $$;
            "#,
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
