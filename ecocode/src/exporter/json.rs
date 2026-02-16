//! JSON exporter — writes measurement records as a JSON array to a file.

use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::exporter::{Exporter, ExporterType, Record};

/// Exports records to a JSON file as a top-level array `[{...}, {...}, ...]`.
///
/// Records are streamed incrementally: the opening `[` is written on the first
/// record, commas are inserted between records, and the closing `]` is written
/// during `export()`.
pub struct JsonExporter {
    file_path: String,
    writer: BufWriter<File>,
    first_record: bool,
}

impl JsonExporter {
    /// Creates a new JSON exporter that writes to the given file path.
    pub fn new(file_path: String) -> Result<JsonExporter, Box<dyn Error>> {
        let file = File::create(&file_path)?;
        let writer = BufWriter::new(file);
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
        if !self.first_record {
            self.writer.write_all(b",")?;
        } else {
            // Begin the JSON array on the first record
            self.writer.write_all(b"[")?;
            self.first_record = false;
        }

        let json = serde_json::to_string(&record)?;
        self.writer.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Closes the JSON array and flushes the file.
    fn export(&mut self) -> Result<(), Box<dyn Error>> {
        self.writer.write_all(b"]")?;
        self.writer.flush()?;

        println!("\n[JSON EXPORT]");
        println!("Records saved to: {}", self.file_path);
        Ok(())
    }

    fn export_line(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
