use notion_client::{
    NotionClientError,
    endpoints::{Client, search::title},
    objects::database,
};
use std::{option, process};

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
                example: "true / false".to_string(),
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
                example: "2020-12-07T12:00:00Z / 2020-12-08T12:00:00Z - 2020-12-09T12:00:00Z"
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
                    .join(" / "),
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
                    .join(" / "),
            },
            database::DatabaseProperty::Status { status, .. } => PropertyInfo {
                name: name.clone(),
                r#type: "Status",
                example: status
                    .groups
                    .iter()
                    .map(|group| group.name.clone())
                    .collect::<Vec<String>>()
                    .join(" / "),
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
