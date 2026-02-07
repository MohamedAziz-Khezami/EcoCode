use std::fs::File;

use crate::exporter::{Exporter, ExporterType, Record};
use csv::Writer;

pub struct CsvExporter {
    file_path: String,
    writer: Writer<File>,
}

impl<'a> CsvExporter{
    pub fn new(file_path: String) -> Result<CsvExporter, Box<dyn std::error::Error>> {
        let file = File::create(&file_path)?;
        let writer = csv::Writer::from_writer(file);
        Ok(CsvExporter {
            file_path,
            writer,
        })
    }
}

impl Exporter for CsvExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Csv
    }
    fn add_record(&mut self, record: Record) -> Result<(), Box<dyn std::error::Error>> {
        if record.id == 1 {
            self.writer.write_record(vec!["ID", "PID", "Timestamp", "CPU%", "GPU%", "CPU(W)", "GPU(W)"]).unwrap();
        }
        self.writer.write_record(&record.to_vec()).unwrap();
        self.writer.flush().unwrap();
        Ok(())
    }

    fn export(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n[CSV EXPORT]");
        println!("Records found in  File: {}", self.file_path);
        Ok(())
    }
    fn export_line(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        Ok(())
    }
}