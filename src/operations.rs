use chrono::DateTime;
use notion_client::{
    NotionClientError,
    endpoints::{Client, pages::create, search::title},
    objects::{
        database::{self, DatabaseProperty},
        page::{DatePropertyValue, PageProperty, SelectPropertyValue},
    },
};
use regex::Regex;
use url;

use std::{
    collections::{BTreeMap, HashMap},
    process,
};

pub struct NotionClient {
    client: Client,
}

impl NotionClient {
    pub fn new(token: String) -> Self {
        let client = Client::new(token, None);
        match client {
            Ok(client) => return Self { client: client },
            Err(e) => {
                eprintln!("fail to obtain a client.");
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
    pub async fn list_database(&self) -> Result<Vec<database::Database>, NotionClientError> {
        let list_database_request = title::request::SearchByTitleRequest {
            filter: Some(title::request::Filter {
                value: title::request::FilterValue::Database,
                property: title::request::FilterProperty::Object,
            }),
            ..Default::default()
        };

        let response = self
            .client
            .search
            .search_by_title(list_database_request)
            .await;
        match response {
            Err(e) => return Err(e),
            Ok(response) => {
                let mut databases: Vec<database::Database> = vec![];
                for page_or_database in response.results {
                    if let title::response::PageOrDatabase::Database(database) = page_or_database {
                        databases.push(database);
                    };
                }
                return Ok(databases);
            }
        }
    }

    pub async fn view_database(
        &self,
        database_id: &str,
    ) -> Result<database::Database, NotionClientError> {
        let database = self.client.databases.retrieve_a_database(database_id).await;
        match database {
            Err(e) => return Err(e),
            Ok(database) => return Ok(database),
        }
    }

    pub async fn add_item_to_database(
        &self,
        database_id: &str,
        properties: HashMap<&str, &str>,
    ) -> Result<(), NotionClientError> {
        let target_db = match self.view_database(database_id).await {
            Ok(database) => database,
            Err(e) => return Err(e),
        };

        if properties.len().ne(&target_db.properties.len()) {
            eprintln!("the lengths of keys in Notion DB and in csv header differ.");
            process::exit(1);
        }

        let mut parsed_properties = BTreeMap::<String, PageProperty>::new();
        for (key, property) in target_db.properties {
            let input_value = *properties.get(&key as &str).unwrap();
            match property {
                DatabaseProperty::Checkbox { .. } => {
                    let input_value: bool = match input_value.parse() {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!(
                                "{} cannot be parsed as an input for {}. Please enter \"true\" or \"false\" as a Checkbox property.",
                                input_value, key
                            );
                            eprintln!("{}", e);
                            process::exit(1);
                        }
                    };
                    parsed_properties.insert(
                        key,
                        PageProperty::Checkbox {
                            id: None,
                            checkbox: input_value,
                        },
                    );
                }
                DatabaseProperty::Date { .. } => {
                    let dates_regex = Regex::new(r"from\s+(\S+)\s+to\s+(\S+)").unwrap();
                    let date_property = if dates_regex.is_match(input_value) {
                        let (start_date, end_date) = dates_regex
                            .captures(input_value)
                            .map(|caps| {
                                let start_date = match DateTime::parse_from_rfc3339(&caps[1])
                                    .or_else(|_| DateTime::parse_from_rfc2822(&caps[1]))
                                {
                                    Ok(date) => date,
                                    Err(e) => {
                                        eprintln!("fail to parse the start date string.");
                                        eprintln!("{}", e);
                                        process::exit(1);
                                    }
                                };
                                let end_date = match DateTime::parse_from_rfc3339(&caps[2])
                                    .or_else(|_| DateTime::parse_from_rfc2822(&caps[2]))
                                {
                                    Ok(date) => date,
                                    Err(e) => {
                                        eprintln!("fail to parse the end date string.");
                                        eprintln!("{}", e);
                                        process::exit(1);
                                    }
                                };
                                (start_date, end_date)
                            })
                            .unwrap();
                        DatePropertyValue {
                            start: Some(notion_client::objects::page::DateOrDateTime::DateTime(
                                start_date.to_utc(),
                            )),
                            end: Some(notion_client::objects::page::DateOrDateTime::DateTime(
                                end_date.to_utc(),
                            )),
                            time_zone: None,
                        }
                    } else {
                        let date = DateTime::parse_from_rfc3339(input_value)
                            .or_else(|_| DateTime::parse_from_rfc2822(input_value));
                        let date = match date {
                            Ok(date) => date,
                            Err(e) => {
                                eprintln!("fail to parse the date string.");
                                eprintln!("{}", e);
                                process::exit(1);
                            }
                        };
                        DatePropertyValue {
                            start: Some(notion_client::objects::page::DateOrDateTime::DateTime(
                                date.to_utc(),
                            )),
                            end: None,
                            time_zone: None,
                        }
                    };

                    parsed_properties.insert(
                        key,
                        PageProperty::Date {
                            id: None,
                            date: Some(date_property),
                        },
                    );
                }
                DatabaseProperty::Email { .. } => todo!(),
                DatabaseProperty::MultiSelect { multi_select, .. } => {
                    let options: Vec<String> = multi_select
                        .options
                        .iter()
                        .map(|option| option.name.clone())
                        .collect();
                    if !options.iter().any(|option| option.eq(input_value)) {
                        eprintln!(
                            "{} cannot be used as an input for {}. Please select from following options: {}",
                            input_value,
                            key,
                            options.join(" / ")
                        );
                        process::exit(1);
                    }
                    parsed_properties.insert(
                        key,
                        PageProperty::MultiSelect {
                            id: None,
                            multi_select: vec![SelectPropertyValue {
                                name: Some(input_value.to_string()),
                                color: None,
                                id: None,
                            }],
                        },
                    );
                }
                DatabaseProperty::Number { .. } => todo!(),
                DatabaseProperty::PhoneNumber { .. } => todo!(),
                DatabaseProperty::Select { select, .. } => todo!(),
                DatabaseProperty::Status { status, .. } => todo!(),
                DatabaseProperty::Title { .. } => todo!(),
                DatabaseProperty::Url { .. } => {
                    let input_value = match url::Url::parse(input_value) {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!(
                                "{} cannot be parsed as an input for {}. Please enter proper URL as a Url property.",
                                input_value, key
                            );
                            eprintln!("{}", e);
                            process::exit(1);
                        }
                    };
                    parsed_properties.insert(
                        key,
                        PageProperty::Url {
                            id: None,
                            url: Some(input_value.to_string()),
                        },
                    );
                }
                _ => {}
            }
        }
        let request = create::request::CreateAPageRequest {
            parent: notion_client::objects::parent::Parent::DatabaseId {
                database_id: database_id.to_string(),
            },
            properties: parsed_properties,
            ..Default::default()
        };
        match self.client.pages.create_a_page(request).await {
            Ok(_db) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

pub fn get_example_for_database_property(database_property: &DatabaseProperty) -> String {
    match database_property {
        DatabaseProperty::Checkbox { .. } => "true/false".to_string(),
        DatabaseProperty::CreatedBy { .. } => "-".to_string(),
        DatabaseProperty::CreatedTime { .. } => "-".to_string(),
        DatabaseProperty::Date { .. } => {
            return "2020-12-07T12:00:00Z/from 2020-12-08T12:00:00Z to 2020-12-09T12:00:00Z"
                .to_string();
        }
        DatabaseProperty::Email { .. } => "foo@example.com".to_string(),
        DatabaseProperty::Files { .. } => "-".to_string(),
        DatabaseProperty::Formula { .. } => "-".to_string(),
        DatabaseProperty::LastEditedBy { .. } => "-".to_string(),
        DatabaseProperty::LastEditedTime { .. } => "-".to_string(),
        DatabaseProperty::MultiSelect { multi_select, .. } => multi_select
            .options
            .iter()
            .map(|option_value| option_value.name.clone())
            .collect::<Vec<String>>()
            .join("/"),
        DatabaseProperty::Number { .. } => "123".to_string(),
        DatabaseProperty::People { .. } => "-".to_string(),
        DatabaseProperty::PhoneNumber { .. } => "123-456-7890".to_string(),
        DatabaseProperty::Relation { .. } => "-".to_string(),
        DatabaseProperty::RichText { .. } => "-".to_string(),
        DatabaseProperty::Rollup { .. } => "-".to_string(),
        DatabaseProperty::Select { select, .. } => select
            .options
            .iter()
            .map(|option_value| option_value.name.clone())
            .collect::<Vec<String>>()
            .join("/"),
        DatabaseProperty::Status { status, .. } => status
            .groups
            .iter()
            .map(|group| group.name.clone())
            .collect::<Vec<String>>()
            .join("/"),
        DatabaseProperty::Title { .. } => "Title".to_string(),
        DatabaseProperty::Url { .. } => "https://jonnity.com".to_string(),
        DatabaseProperty::Button { .. } => "-".to_string(),
    }
}

pub fn propery_to_string(database_property: &DatabaseProperty) -> &str {
    match database_property {
        DatabaseProperty::Checkbox { .. } => "Checkbox",
        DatabaseProperty::CreatedBy { .. } => "CreatedBy",
        DatabaseProperty::CreatedTime { .. } => "CreatedTime",
        DatabaseProperty::Date { .. } => "Date",
        DatabaseProperty::Email { .. } => "Email",
        DatabaseProperty::Files { .. } => "Files",
        DatabaseProperty::Formula { .. } => "Formula",
        DatabaseProperty::LastEditedBy { .. } => "LastEditedBy",
        DatabaseProperty::LastEditedTime { .. } => "LastEditedTime",
        DatabaseProperty::MultiSelect { .. } => "MultiSelect",
        DatabaseProperty::Number { .. } => "Number",
        DatabaseProperty::People { .. } => "People",
        DatabaseProperty::PhoneNumber { .. } => "PhoneNumber",
        DatabaseProperty::Relation { .. } => "Relation",
        DatabaseProperty::RichText { .. } => "RichText",
        DatabaseProperty::Rollup { .. } => "Rollup",
        DatabaseProperty::Select { .. } => "Select",
        DatabaseProperty::Status { .. } => "Status",
        DatabaseProperty::Title { .. } => "Title",
        DatabaseProperty::Url { .. } => "Url",
        DatabaseProperty::Button { .. } => "Button",
    }
}
