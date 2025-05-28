use notion_client::{
    NotionClientError,
    endpoints::{Client, pages::create, search::title},
    objects::{
        database::{self, DatabaseProperty},
        page::{PageProperty, SelectPropertyValue},
    },
};
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

pub struct PropertyInfo {
    pub name: String,
    pub r#type: &'static str,
    pub example: String,
}

pub fn database_to_properties_info(database: &database::Database) -> Vec<PropertyInfo> {
    database
        .properties
        .iter()
        .map(|(name, property)| match property {
            database::DatabaseProperty::Checkbox { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Checkbox",
                example: "true/false".to_string(),
            },
            database::DatabaseProperty::CreatedBy { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "CreatedBy",
                example: "-".to_string(),
            },
            database::DatabaseProperty::CreatedTime { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "CreatedTime",
                example: "-".to_string(),
            },
            database::DatabaseProperty::Date { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Date",
                example: "2020-12-07T12:00:00Z/from 2020-12-08T12:00:00Z to 2020-12-09T12:00:00Z"
                    .to_string(),
            },
            database::DatabaseProperty::Email { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Email",
                example: "foo@example.com".to_string(),
            },
            database::DatabaseProperty::Files { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Files",
                example: "-".to_string(),
            },
            database::DatabaseProperty::Formula { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Formula",
                example: "-".to_string(),
            },
            database::DatabaseProperty::LastEditedBy { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "LastEditedBy",
                example: "-".to_string(),
            },
            database::DatabaseProperty::LastEditedTime { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "LastEditedTime",
                example: "-".to_string(),
            },
            database::DatabaseProperty::MultiSelect { multi_select, .. } => PropertyInfo {
                name: name.clone(),
                r#type: "MultiSelect",
                example: multi_select
                    .options
                    .iter()
                    .map(|option_value| option_value.name.clone())
                    .collect::<Vec<String>>()
                    .join("/"),
            },
            database::DatabaseProperty::Number { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Number",
                example: "123".to_string(),
            },
            database::DatabaseProperty::People { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "People",
                example: "-".to_string(),
            },
            database::DatabaseProperty::PhoneNumber { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "PhoneNumber",
                example: "123-456-7890".to_string(),
            },
            database::DatabaseProperty::Relation { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Relation",
                example: "-".to_string(),
            },
            database::DatabaseProperty::RichText { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "RichText",
                example: "-".to_string(),
            },
            database::DatabaseProperty::Rollup { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Rollup",
                example: "-".to_string(),
            },
            database::DatabaseProperty::Select { select, .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Select",
                example: select
                    .options
                    .iter()
                    .map(|option_value| option_value.name.clone())
                    .collect::<Vec<String>>()
                    .join("/"),
            },
            database::DatabaseProperty::Status { status, .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Status",
                example: status
                    .groups
                    .iter()
                    .map(|group| group.name.clone())
                    .collect::<Vec<String>>()
                    .join("/"),
            },
            database::DatabaseProperty::Title { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Title",
                example: "-".to_string(),
            },
            database::DatabaseProperty::Url { .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Url",
                example: "https://jonnity.com".to_string(),
            },
        })
        .collect()
}
