mod commands;
mod csv_reader;
mod operations;

use clap::Parser;
use commands::{CliArgs, Commands};
use operations::NotionClient;
use std::{process, vec};

#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();
    let client = match NotionClient::new(cli.token) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    match &cli.command {
        Commands::DbList => match client.list_database().await {
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
            Ok(databases) => {
                println!("The list of databases ({{title}}: {{id}}):");
                for database in databases {
                    println!("{}: {}", database.title, database.id);
                }
            }
        },
        Commands::DbView(args) => match client.get_database_properties(&args.id).await {
            Err(e) => {
                eprintln!("Fail to retrieve the databases information.");
                eprintln!("{}", e);
                process::exit(1);
            }
            Ok(properties) => {
                if let Some(file_path) = &args.file {
                    let mut property_keys: Vec<String> = vec![];
                    let mut property_examples: Vec<String> = vec![];
                    for property in properties {
                        property_keys.push(property.key);
                        property_examples.push(property.example);
                    }
                    let property_keys_csv = property_keys.join(", ");
                    let property_example_csv = property_examples.join(", ");
                    let content = format!("{}\n{}", property_keys_csv, property_example_csv);
                    match std::fs::write(file_path, content) {
                        Ok(_) => println!("Successfully wrote to {}", file_path),
                        Err(e) => {
                            eprintln!("Failed to write to file: {}", e);
                            process::exit(1);
                        }
                    }
                } else {
                    println!("The structure and columns of the database are as follows:");
                    let (keys, properties_list) = properties
                        .iter()
                        .map(|property| (property.key.clone(), property.r#type.clone()))
                        .collect::<(Vec<String>, Vec<String>)>();
                    let properties_list = vec![properties_list];
                    match make_aligned_string(keys, properties_list) {
                        Ok(str) => println!("{}", str),
                        Err(e) => {
                            eprintln!("fail to diplay properties. {}", e);
                            process::exit(1);
                        }
                    }
                }
            }
        },
        Commands::DbAdd(args) => {
            let csv_records = match csv_reader::CsvRecords::new(&args.file) {
                Ok(csv_records) => csv_records,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };

            for properties in csv_records {
                match client.add_item_to_database(&args.id, &properties).await {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!(
                            "Error has occured during creating new database item in a record of csv."
                        );
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };
            }
        }
        Commands::DbQuery(args) => {
            let result = match client.query_database(&args.id, None).await {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Fail to query database");
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            if result.has_more {
                println!("Results are more than 100, and not all results can be displayed.");
            }

            match make_aligned_string(result.keys, result.properties_list) {
                Ok(str) => println!("{}", str),
                Err(e) => {
                    eprintln!("fail to diplay properties. {}", e);
                    process::exit(1);
                }
            };
        }
    }
}

fn make_aligned_string(
    keys: Vec<String>,
    properties_list: Vec<Vec<String>>,
) -> Result<String, String> {
    if !properties_list
        .iter()
        .all(|properties| properties.len().eq(&keys.len()))
    {
        return Err("The lenght of keys and properties are not matching.".to_string());
    }

    let mut keys_row = "|".to_string();
    let mut properties_rows = vec!["|".to_string(); properties_list.len()];

    for i in 0..keys.len() {
        let mut max_length = keys[i].len();
        properties_list.iter().for_each(|properties| {
            max_length = max_length.max(properties[i].len());
        });
        keys_row += &format!(" {:<width$} |", keys[i], width = max_length);
        for j in 0..properties_list.len() {
            properties_rows[j] +=
                &format!(" {:<width$} |", properties_list[j][i], width = max_length);
        }
    }
    Ok(format!("{}\n{}", keys_row, properties_rows.join("\n")))
}

#[cfg(test)]
mod tests {
    use crate::make_aligned_string;

    #[test]
    fn read_csv() {
        let test_keys = vec!["1".to_string(), "12".to_string(), "12345".to_string()];
        let test_properties: Vec<Vec<String>> =
            vec![vec!["1".to_string(), "1".to_string(), "1".to_string()]];
        let result = make_aligned_string(test_keys, test_properties);
        assert_eq!(
            result,
            Ok("| 1 | 12 | 12345 |\n| 1 | 1  | 1     |".to_string())
        );
    }
}
