use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::exporter::{Exporter, ExporterType, Record};

pub struct JsonExporter {
    pub file_path: String,
    pub writer: BufWriter<File>,
    pub first_record: bool,
}

impl JsonExporter {
    pub fn new(file_path: String) -> Result<JsonExporter, Box<dyn Error>> {
        let file = File::create(&file_path)?;
        let writer = BufWriter::new(file); // Wrap the file in a buffered writer
        Ok(JsonExporter {
            file_path,
            writer,
            first_record: true,
        })
    }
}

impl Exporter for JsonExporter {
    fn exporter_type(&self) -> ExporterType {
        ExporterType::Json
    }

    fn add_record(&mut self, record: Record) -> Result<(), Box<dyn Error>> {
        // Write a comma if it's not the first record
        if !self.first_record {
            self.writer.write_all(b",")?; // Add a comma between records
        } else {
            // If this is the first record, write the opening bracket for the JSON array
            self.writer.write_all(b"[")?; // Begin the JSON array
            self.first_record = false; // Mark that the first record is processed
        }

        // Serialize the record to JSON and write it
        let json = serde_json::to_string(&record)?;
        self.writer.write_all(json.as_bytes())?;

        Ok(())
    }
    fn export(&mut self) -> Result<(), Box<dyn Error>> {
        // Write the closing bracket for the JSON array
        self.writer.write_all(b"]")?;
        self.writer.flush()?;

        println!("\n[JSON EXPORT]");
        println!("Records found in  File: {}", self.file_path);
        Ok(())
    }
    fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
