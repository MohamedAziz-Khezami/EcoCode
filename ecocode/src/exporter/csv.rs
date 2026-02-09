use std::fs::File;

use crate::exporter::{Exporter, ExporterType, Record};
use csv::Writer;

pub struct CsvExporter {
    file_path: String,
    writer: Writer<File>,
    pub first_record: bool,
}

impl CsvExporter {
    pub fn new(file_path: String) -> Result<CsvExporter, Box<dyn std::error::Error>> {
        let file = File::create(&file_path)?;
        let writer = csv::Writer::from_writer(file);
        Ok(CsvExporter {
            file_path,
            writer,
            first_record: true,
        })
    }
}

impl Exporter for CsvExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Csv
    }
    fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
        if self.first_record {
            self.writer
                .write_record(vec![
                    "ID",
                    "PID",
                    "Timestamp",
                    "CPU%",
                    "CPU(W)",
                    "GPU%",
                    "GPU(W)",
                ])
                .unwrap();
        }
        self.first_record = false;
        self.writer.write_record(&record.to_vec()).unwrap();
        self.writer.flush().unwrap();
        Ok(())
    }

    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n[CSV EXPORT]");
        println!("Records found in  File: {}", self.file_path);
        Ok(())
    }
    fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
