use notion_client::{
    NotionClientError,
    endpoints::{Client, search::title},
    objects::database,
};
use std::process;

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
    pub r#type: String,
    pub example: String,
}

pub fn database_to_properties_info(database: &database::Database) -> Vec<PropertyInfo> {
    database
        .properties
        .iter()
        .map(|(name, property)| PropertyInfo {
            name: name.clone(),
            example: "foo".to_string(),
            r#type: match *property {
                database::DatabaseProperty::Checkbox { .. } => "Checkbox".to_string(),
                database::DatabaseProperty::CreatedBy { .. } => "CreatedBy".to_string(),
                database::DatabaseProperty::CreatedTime { .. } => "CreatedTime".to_string(),
                database::DatabaseProperty::Date { .. } => "Date".to_string(),
                database::DatabaseProperty::Email { .. } => "Email".to_string(),
                database::DatabaseProperty::Files { .. } => "Files".to_string(),
                database::DatabaseProperty::Formula { .. } => "Formula".to_string(),
                database::DatabaseProperty::LastEditedBy { .. } => "LastEditedBy".to_string(),
                database::DatabaseProperty::LastEditedTime { .. } => "LastEditedTime".to_string(),
                database::DatabaseProperty::MultiSelect { .. } => "MultiSelect".to_string(),
                database::DatabaseProperty::Number { .. } => "Number".to_string(),
                database::DatabaseProperty::People { .. } => "People".to_string(),
                database::DatabaseProperty::PhoneNumber { .. } => "PhoneNumber".to_string(),
                database::DatabaseProperty::Relation { .. } => "Relation".to_string(),
                database::DatabaseProperty::RichText { .. } => "RichText".to_string(),
                database::DatabaseProperty::Rollup { .. } => "Rollup".to_string(),
                database::DatabaseProperty::Select { .. } => "Select".to_string(),
                database::DatabaseProperty::Status { .. } => "Status".to_string(),
                database::DatabaseProperty::Title { .. } => "Title".to_string(),
                database::DatabaseProperty::Url { .. } => "Url".to_string(),
            },
        })
        .collect()
}
