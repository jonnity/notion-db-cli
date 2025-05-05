use notion_client::{
    NotionClientError,
    endpoints::{
        Client,
        search::title::{request, response::PageOrDatabase},
    },
    objects::database::Database,
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
    pub async fn list_database(&self) -> Result<Vec<Database>, NotionClientError> {
        let list_database_request = request::SearchByTitleRequest {
            filter: Some(request::Filter {
                value: request::FilterValue::Database,
                property: request::FilterProperty::Object,
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
                let mut databases: Vec<Database> = vec![];
                for page_or_database in response.results {
                    if let PageOrDatabase::Database(database) = page_or_database {
                        databases.push(database);
                    };
                }
                return Ok(databases);
            }
        }
    }
}
