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
                    let mut property_keys_row = "|".to_string();
                    let mut property_type_row = "|".to_string();
                    properties.iter().for_each(|property| {
                        let property_type = &property.r#type;
                        let key = &property.key;

                        let name_len = key.chars().count();
                        let type_len = property_type.chars().count();
                        let max_len = name_len.max(type_len);
                        let pudded_key = format!(" {:<width$} |", key, width = max_len).to_string();
                        let pudded_type =
                            format!(" {:<width$} |", property_type, width = max_len).to_string();
                        property_keys_row += &pudded_key;
                        property_type_row += &pudded_type;
                    });
                    println!("{}", property_keys_row);
                    println!("{}", property_type_row);
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

            match diplay_properties_table(result.keys, result.properties_list) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("fail to diplay properties. {}", e);
                    process::exit(1);
                }
            };
        }
    }
}

fn diplay_properties_table(
    keys: Vec<String>,
    properties_list: Vec<Vec<String>>,
) -> Result<(), String> {
    if !properties_list
        .iter()
        .all(|properties| properties.len().eq(&keys.len()))
    {
        return Err("The lenght of keys and properties are not matching.".to_string());
    }

    let mut keys_row = "|".to_string();
    let mut properties_rows = vec!["|".to_string(); keys.len()];

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
    println!("{}", keys_row);
    properties_rows
        .iter()
        .for_each(|properties_row| println!("{}", properties_row));
    Ok(())
}
