use chrono::DateTime;
use email_address_parser::EmailAddress;
use notion_client::{
    NotionClientError,
    endpoints::{
        Client, databases::query::request::QueryDatabaseRequest, pages::create, search::title,
    },
    objects::{
        database::{self, DatabaseProperty},
        page::{DatePropertyValue, PageProperty, SelectPropertyValue},
        rich_text::{RichText, Text},
    },
};
use regex::Regex;
use serde_json::Number;

use std::{
    collections::{BTreeMap, HashMap},
    vec,
};

pub struct NotionClient {
    client: Client,
}

pub struct DatabaseListResult {
    pub title: String,
    pub id: String,
}
pub struct DatabaseViewResult {
    pub key: String,
    pub r#type: String,
    pub example: String,
}

pub struct DatabaseQueryResult {
    pub keys: Vec<String>,
    pub properties_list: Vec<Vec<String>>,
    pub has_more: bool,
}

impl NotionClient {
    pub fn new(token: String) -> Result<Self, String> {
        let client = Client::new(token, None);
        match client {
            Ok(client) => Ok(Self { client }),
            Err(e) => Err(format!("Fail to obtain a client.\n{}", e)),
        }
    }
    pub async fn list_database(&self) -> Result<Vec<DatabaseListResult>, String> {
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
            Err(e) => Err(format!("Fail to obtain the list of databases. {}", e)),
            Ok(response) => {
                let mut database_list: Vec<DatabaseListResult> = vec![];
                for page_or_database in response.results {
                    if let title::response::PageOrDatabase::Database(database) = page_or_database {
                        database_list.push(DatabaseListResult {
                            title: database.title[0].plain_text().expect("no title is set"),
                            id: database.id.expect("no id is set"),
                        });
                    };
                }
                Ok(database_list)
            }
        }
    }

    pub async fn get_database_properties(
        &self,
        database_id: &str,
    ) -> Result<Vec<DatabaseViewResult>, String> {
        match self.retrieve_database(database_id).await {
            Err(e) => Err(format!("Fail to retrieve the databases information. {}", e)),
            Ok(database) => {
                let mut database_properties: Vec<DatabaseViewResult> = vec![];
                for (key, property) in database.properties {
                    database_properties.push(DatabaseViewResult {
                        key,
                        r#type: propery_to_string(&property.clone()),
                        example: get_example_for_database_property(&property.clone()),
                    })
                }
                Ok(database_properties)
            }
        }
    }

    pub async fn add_item_to_database(
        &self,
        database_id: &str,
        properties: &HashMap<String, String>,
    ) -> Result<(), String> {
        let target_db = match self.retrieve_database(database_id).await {
            Ok(database) => database,
            Err(e) => {
                return Err(format!(
                    "Fail to retrieve the database to add items.\n{}",
                    e
                ));
            }
        };

        if properties.len().ne(&target_db.properties.len()) {
            return Err("The lengths of keys in Notion DB and in csv header differ.".to_string());
        }

        let mut parsed_properties = BTreeMap::<String, PageProperty>::new();
        let dates_regex = Regex::new(r"from\s+(\S+)\s+to\s+(\S+)").unwrap();
        for (key, property) in target_db.properties {
            let input_value = properties.get(&key as &str).unwrap();
            match property {
                DatabaseProperty::Checkbox { .. } => {
                    let input_value: bool = match input_value.parse() {
                        Ok(b) => b,
                        Err(e) => {
                            return Err(format!(
                                "{} cannot be parsed as an input for {}. Please enter \"true\" or \"false\" as a Checkbox property.\n{}",
                                input_value, key, e
                            ));
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
                    let date_property = if dates_regex.is_match(input_value) {
                        let (start_date, end_date) = dates_regex
                            .captures(input_value)
                            .map(|caps| {
                                let start_date = DateTime::parse_from_rfc3339(&caps[1])
                                    .or_else(|_| DateTime::parse_from_rfc2822(&caps[1]));
                                let end_date = DateTime::parse_from_rfc3339(&caps[2])
                                    .or_else(|_| DateTime::parse_from_rfc2822(&caps[2]));

                                (start_date, end_date)
                            })
                            .unwrap();

                        let start_date = match start_date {
                            Ok(date) => date,
                            Err(e) => {
                                return Err(format!("Fail to parse the start date string.\n{}", e));
                            }
                        };
                        let end_date = match end_date {
                            Ok(date) => date,
                            Err(e) => {
                                return Err(format!("Fail to parse the end date string.\n{}", e));
                            }
                        };

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
                                return Err(format!("Fail to parse the date string.\n{}", e));
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
                DatabaseProperty::Email { .. } => {
                    match EmailAddress::parse(
                        input_value,
                        Some(email_address_parser::ParsingOptions { is_lax: true }),
                    ) {
                        Some(email) => {
                            parsed_properties.insert(
                                key,
                                PageProperty::Email {
                                    id: None,
                                    email: Some(email.to_string()),
                                },
                            );
                        }
                        None => {
                            return Err("Fail to parse the email address.".to_string());
                        }
                    };
                }
                DatabaseProperty::MultiSelect { multi_select, .. } => {
                    let options: Vec<String> = multi_select
                        .options
                        .iter()
                        .map(|option| option.name.clone())
                        .collect();
                    let values: Vec<&str> = input_value.split("/").collect();
                    if !values
                        .iter()
                        .all(|value| options.contains(&value.to_string()))
                    {
                        return Err(format!(
                            "{} cannot be used as an input for {}. Please select from following options: {}",
                            input_value,
                            key,
                            options.join(" / ")
                        ));
                    }
                    parsed_properties.insert(
                        key,
                        PageProperty::MultiSelect {
                            id: None,
                            multi_select: values
                                .iter()
                                .map(|value| SelectPropertyValue {
                                    name: Some(value.to_string()),
                                    color: None,
                                    id: None,
                                })
                                .collect(),
                        },
                    );
                }
                DatabaseProperty::Number { .. } => match input_value.parse::<Number>() {
                    Ok(number) => {
                        parsed_properties.insert(
                            key,
                            PageProperty::Number {
                                id: None,
                                number: Some(number),
                            },
                        );
                    }
                    Err(e) => {
                        return Err(format!("Fail to parse a number.\n{}", e));
                    }
                },
                DatabaseProperty::PhoneNumber { .. } => {
                    parsed_properties.insert(
                        key,
                        PageProperty::PhoneNumber {
                            id: None,
                            phone_number: Some(input_value.to_string()),
                        },
                    );
                }
                DatabaseProperty::RichText { .. } => {
                    parsed_properties.insert(
                        key,
                        PageProperty::RichText {
                            id: None,
                            rich_text: vec![RichText::Text {
                                text: Text {
                                    content: input_value.to_string(),
                                    link: None,
                                },
                                annotations: None,
                                plain_text: Some(input_value.to_string()),
                                href: None,
                            }],
                        },
                    );
                }
                DatabaseProperty::Select { select, .. } => {
                    if select
                        .options
                        .iter()
                        .any(|option| option.name.eq(input_value))
                    {
                        parsed_properties.insert(
                            key,
                            PageProperty::Select {
                                id: None,
                                select: Some(SelectPropertyValue {
                                    id: None,
                                    name: Some(input_value.to_string()),
                                    color: None,
                                }),
                            },
                        );
                    } else {
                        return Err(format!(
                            "invalid option for select property. The options are following: {}",
                            select
                                .options
                                .iter()
                                .map(|option| option.name.to_string())
                                .collect::<Vec<String>>()
                                .join(" / ")
                        ));
                    }
                }
                DatabaseProperty::Status { status, .. } => {
                    if status
                        .options
                        .iter()
                        .any(|option| option.name.eq(input_value))
                    {
                        parsed_properties.insert(
                            key,
                            PageProperty::Status {
                                id: None,
                                status: Some(SelectPropertyValue {
                                    id: None,
                                    name: Some(input_value.to_string()),
                                    color: None,
                                }),
                            },
                        );
                    } else {
                        return Err(format!(
                            "invalid option for status property. The options are following: {}",
                            status
                                .options
                                .iter()
                                .map(|option| option.name.to_string())
                                .collect::<Vec<String>>()
                                .join(" / ")
                        ));
                    }
                }
                DatabaseProperty::Title { .. } => {
                    parsed_properties.insert(
                        key,
                        PageProperty::Title {
                            id: None,
                            title: vec![RichText::Text {
                                text: Text {
                                    content: input_value.to_string(),
                                    link: None,
                                },
                                annotations: None,
                                plain_text: Some(input_value.to_string()),
                                href: None,
                            }],
                        },
                    );
                }
                DatabaseProperty::Url { .. } => {
                    let input_value = match url::Url::parse(input_value) {
                        Ok(b) => b,
                        Err(e) => {
                            return Err(format!(
                                "{} cannot be parsed as an input for {}. Please enter proper URL as a Url property.\n{}",
                                input_value, key, e
                            ));
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
            Err(e) => Err(format!("Fail to create a page.\n{}", e)),
        }
    }

    pub async fn query_database(
        &self,
        database_id: &str,
        _query: Option<&str>,
    ) -> Result<DatabaseQueryResult, NotionClientError> {
        let query_request: QueryDatabaseRequest = QueryDatabaseRequest {
            ..Default::default()
        };
        match self
            .client
            .databases
            .query_a_database(database_id, query_request)
            .await
        {
            Ok(res) => {
                let pages = res.results;
                let keys: Vec<String> = pages
                    .first()
                    .unwrap()
                    .properties
                    .keys()
                    .map(|key| key.to_string())
                    .collect();
                let properties_list: Vec<Vec<String>> = pages
                    .iter()
                    .map(|page| {
                        page.properties
                            .values()
                            .map(get_property_value_str)
                            .collect()
                    })
                    .collect();

                Ok(DatabaseQueryResult {
                    keys,
                    properties_list,
                    has_more: res.has_more,
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn retrieve_database(
        &self,
        database_id: &str,
    ) -> Result<database::Database, NotionClientError> {
        let database = self.client.databases.retrieve_a_database(database_id).await;
        match database {
            Err(e) => Err(e),
            Ok(database) => Ok(database),
        }
    }
}

pub fn get_example_for_database_property(database_property: &DatabaseProperty) -> String {
    match database_property {
        DatabaseProperty::Checkbox { .. } => "true/false".to_string(),
        DatabaseProperty::CreatedBy { .. } => "-".to_string(),
        DatabaseProperty::CreatedTime { .. } => "-".to_string(),
        DatabaseProperty::Date { .. } => {
            "2020-12-07T12:00:00Z/from 2020-12-08T12:00:00Z to 2020-12-09T12:00:00Z".to_string()
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
        DatabaseProperty::RichText { .. } => "only plain text is supported".to_string(),
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
        DatabaseProperty::Url { .. } => "https://example.com".to_string(),
        DatabaseProperty::Button { .. } => "-".to_string(),
    }
}

pub fn propery_to_string(database_property: &DatabaseProperty) -> String {
    match database_property {
        DatabaseProperty::Checkbox { .. } => "Checkbox".to_string(),
        DatabaseProperty::CreatedBy { .. } => "CreatedBy".to_string(),
        DatabaseProperty::CreatedTime { .. } => "CreatedTime".to_string(),
        DatabaseProperty::Date { .. } => "Date".to_string(),
        DatabaseProperty::Email { .. } => "Email".to_string(),
        DatabaseProperty::Files { .. } => "Files".to_string(),
        DatabaseProperty::Formula { .. } => "Formula".to_string(),
        DatabaseProperty::LastEditedBy { .. } => "LastEditedBy".to_string(),
        DatabaseProperty::LastEditedTime { .. } => "LastEditedTime".to_string(),
        DatabaseProperty::MultiSelect { .. } => "MultiSelect".to_string(),
        DatabaseProperty::Number { .. } => "Number".to_string(),
        DatabaseProperty::People { .. } => "People".to_string(),
        DatabaseProperty::PhoneNumber { .. } => "PhoneNumber".to_string(),
        DatabaseProperty::Relation { .. } => "Relation".to_string(),
        DatabaseProperty::RichText { .. } => "RichText".to_string(),
        DatabaseProperty::Rollup { .. } => "Rollup".to_string(),
        DatabaseProperty::Select { .. } => "Select".to_string(),
        DatabaseProperty::Status { .. } => "Status".to_string(),
        DatabaseProperty::Title { .. } => "Title".to_string(),
        DatabaseProperty::Url { .. } => "Url".to_string(),
        DatabaseProperty::Button { .. } => "Button".to_string(),
    }
}

pub fn get_property_value_str(property: &PageProperty) -> String {
    match property {
        PageProperty::Checkbox { checkbox, .. } => checkbox.to_string(),
        PageProperty::CreatedBy { created_by, .. } => {
            created_by.name.clone().unwrap_or("".to_string())
        }
        PageProperty::CreatedTime { created_time, .. } => created_time.to_rfc2822(),
        PageProperty::Date {
            date: Some(date), ..
        } => {
            if date.start.is_some() && date.end.is_some() {
                format!(
                    "from {:?} to {:?}",
                    date.start.clone().unwrap(),
                    date.end.clone().unwrap()
                )
            } else if date.start.is_some() {
                format!("{:?}", date.start.clone().unwrap())
            } else {
                "".to_string()
            }
        }
        PageProperty::Email { email, .. } => email.clone().unwrap_or("".to_string()),
        PageProperty::LastEditedBy { last_edited_by, .. } => {
            last_edited_by.name.clone().unwrap_or("".to_string())
        }
        PageProperty::LastEditedTime {
            last_edited_time: Some(last_edited_time),
            ..
        } => last_edited_time.to_rfc2822(),
        PageProperty::MultiSelect { multi_select, .. } => multi_select
            .iter()
            .map(|select| select.name.clone().unwrap_or("".to_string()))
            .collect::<Vec<String>>()
            .join("/"),
        PageProperty::Number {
            number: Some(number),
            ..
        } => number.to_string(),
        PageProperty::People { people, .. } => people
            .iter()
            .map(|user| user.name.clone().unwrap_or("".to_string()))
            .collect::<Vec<String>>()
            .join("/"),
        PageProperty::PhoneNumber { phone_number, .. } => {
            phone_number.clone().unwrap_or("".to_string())
        }
        PageProperty::RichText { rich_text, .. } => rich_text
            .iter()
            .map(|rich_text| rich_text.plain_text().unwrap_or("".to_string()))
            .collect::<Vec<String>>()
            .join(""),
        PageProperty::Select {
            select: Some(select),
            ..
        } => select.name.clone().unwrap_or("".to_string()),
        PageProperty::Status {
            status: Some(select),
            ..
        } => select.name.clone().unwrap_or("".to_string()),
        PageProperty::Title { title, .. } => title
            .iter()
            .map(|rich_text| rich_text.plain_text().unwrap_or("".to_string()))
            .collect::<Vec<String>>()
            .join(""),
        PageProperty::Url { url, .. } => url.clone().unwrap_or("".to_string()),
        PageProperty::UniqueID {
            unique_id: Some(unique_id),
            ..
        } => format!(
            "{}{}",
            unique_id.prefix.clone().unwrap_or("".to_string()),
            match &unique_id.number {
                Some(number) => number.to_string(),
                None => "".to_string(),
            }
        ),
        PageProperty::Verification {
            verification: Some(verification),
            ..
        } => {
            let mut verification_str = match verification.state {
                notion_client::objects::page::VerificationState::Verified => "verified".to_string(),
                notion_client::objects::page::VerificationState::Unverified => {
                    "unverified".to_string()
                }
            };
            if let Some(user) = &verification.verified_by {
                if let Some(name) = &user.name {
                    verification_str += &format!(" by {}", name.clone());
                }
            }
            if let Some(date) = &verification.date {
                if let Some(start_date) = &date.start {
                    if let Some(end_date) = &date.end {
                        verification_str += &format!(" (from {:?} to {:?})", start_date, end_date);
                    } else {
                        verification_str += &format!(" ({:?})", start_date);
                    }
                }
            }
            verification_str
        }
        _ => "".to_string(),
    }
}
