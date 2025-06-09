use std::fs::File;

pub struct CsvRecords {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvRecords {
    pub fn new(file_path: &str) -> Result<Self, String> {
        let file = match File::open(&file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!(
                    "File is not found in {}.\n{}",
                    file_path,
                    e.to_string()
                ));
            }
        };
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(file);

        let headers: Vec<String> = match reader.headers() {
            Ok(headers) => headers.iter().map(|header| header.to_string()).collect(),
            Err(e) => {
                return Err(format!(
                    "Fail to read headers in {}.\n{}",
                    file_path,
                    e.to_string()
                ));
            }
        };

        let mut records: Vec<Vec<String>> = vec![];
        for record in reader.records() {
            match record {
                Ok(record) => records.push(record.iter().map(|value| value.to_string()).collect()),
                Err(e) => {
                    return Err(format!(
                        "Fail to read a record in {}.\n{}",
                        file_path,
                        e.to_string()
                    ));
                }
            }
        }

        if !records.iter().all(|record| record.len().eq(&headers.len())) {
            return Err(format!(
                "The number of a record does not matcdh the number of header.",
            ));
        }

        Ok(CsvRecords {
            headers: headers,
            records: records,
        })
    }
}
