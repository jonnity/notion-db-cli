use std::{collections::HashMap, fs::File};

pub struct CsvRecords {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
    current_index: usize,
}

impl CsvRecords {
    pub fn new(file_path: &str) -> Result<Self, String> {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!("File is not found in {}.\n{}", file_path, e));
            }
        };
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(file);

        let headers: Vec<String> = match reader.headers() {
            Ok(headers) => headers.iter().map(|header| header.to_string()).collect(),
            Err(e) => {
                return Err(format!("Fail to read headers in {}.\n{}", file_path, e));
            }
        };

        let mut records: Vec<Vec<String>> = vec![];
        for record in reader.records() {
            match record {
                Ok(record) => records.push(record.iter().map(|value| value.to_string()).collect()),
                Err(e) => {
                    return Err(format!("Fail to read a record in {}.\n{}", file_path, e));
                }
            }
        }

        if !records.iter().all(|record| record.len().eq(&headers.len())) {
            return Err("The number of a record does not matcdh the number of header.".to_string());
        }

        Ok(CsvRecords {
            headers,
            records,
            current_index: 0,
        })
    }
}

impl Iterator for CsvRecords {
    type Item = HashMap<String, String>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.records.get(self.current_index) {
            let mut header_record_pair = HashMap::<String, String>::new();
            for (i, _) in current.iter().enumerate().take(self.headers.len()) {
                header_record_pair.insert(self.headers[i].clone(), current[i].clone());
            }
            self.current_index += 1;
            Some(header_record_pair)
        } else {
            self.current_index = 0;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CsvRecords;
    #[test]
    fn read_csv() {
        let mut records = CsvRecords::new("./test_files/test_csv_2_rows").unwrap();
        let first = records.next();
        assert!(first.is_some());
        let second = records.next();
        assert!(second.is_some());
        let third = records.next();
        assert!(third.is_none());
    }

    #[test]
    fn invalid_csv() {
        let records = CsvRecords::new("./test_files/test_invalid");
        assert!(records.is_err());
    }
}
