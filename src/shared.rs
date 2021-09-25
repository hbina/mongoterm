use serde::{Deserialize, Serialize};

pub mod keywords {
    pub const INPUT_FILTER: &'static str = "input-filter";
    pub const CONNECTION_URI: &'static str = "connection-uri";
    pub const DATABASE_NAME: &'static str = "database-name";
    pub const COLLECTION_NAME: &'static str = "collection-name";
    pub const PIPELINE: &'static str = "pipeline";
    pub const INPUT_DOCUMENTS: &'static str = "input-documents";
    pub const INPUT_FILE: &'static str = "input-file";
    pub const PROJECT: &'static str = "project";
    pub const PIPELINE_INDEX: &'static str = "pipeline-index";
    pub const LIST: &'static str = "list";
    pub const LIMIT: &'static str = "limit";
    pub const CONFIG_FILE: &'static str = "config-file";
}

pub enum MongoDbCommand {
    Create,
    Aggregate,
    FindMany,
    FindOne,
    Count,
    DeleteMany,
    DeleteOne,
    ListDatabases,
}

impl MongoDbCommand {
    pub fn to_str(self) -> &'static str {
        match self {
            MongoDbCommand::Create => "create",
            MongoDbCommand::Aggregate => "aggregate",
            MongoDbCommand::FindMany => "find-many",
            MongoDbCommand::FindOne => "find-one",
            MongoDbCommand::Count => "count",
            MongoDbCommand::DeleteOne => "delete-one",
            MongoDbCommand::DeleteMany => "delete-many",
            MongoDbCommand::ListDatabases => "list-databases",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pipeline {
    pub name: String,
    pub description: Option<String>,
    pub stages: Vec<mongodb::bson::document::Document>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub connection_uri: String,
    pub collection_name: String,
    pub database_name: String,
    pub pipelines: Vec<Pipeline>,
}

impl Config {
    pub fn from_matches(matches: &clap::ArgMatches) -> Result<Self, Box<dyn std::error::Error>> {
        let config = if let Some(config_file) = matches.value_of(keywords::CONFIG_FILE) {
            let file = std::fs::File::open(config_file)?;
            serde_json::from_reader(file)?
        } else {
            match (
                matches.value_of(keywords::CONNECTION_URI),
                matches.value_of(keywords::DATABASE_NAME),
                matches.value_of(keywords::COLLECTION_NAME),
            ) {
                (Some(connection_uri), Some(database_name), Some(collection_name)) => Config {
                    connection_uri: connection_uri.into(),
                    database_name: database_name.into(),
                    collection_name: collection_name.into(),
                    pipelines: vec![],
                },
                _ => {
                    return Err("Please provide the connection-uri, database-name and collection-name by passing them as arguments or through config-file".into());
                }
            }
        };
        Ok(config)
    }
}

#[derive(Debug)]
pub enum InsertResult {
    One(mongodb::results::InsertOneResult),
    Many(mongodb::results::InsertManyResult),
}

#[derive(Debug)]

pub enum InputType {
    Stdin(std::io::Stdin),
    Arg(String),
    BufReader(std::io::BufReader<std::fs::File>),
}

pub fn find_one_args() -> Vec<clap::Arg<'static, 'static>> {
    vec![
        clap::Arg::with_name(keywords::INPUT_FILTER)
            .long(keywords::INPUT_FILTER)
            .help("The filter to be applied")
            .takes_value(true)
            .required(false),
        clap::Arg::with_name(keywords::PROJECT)
            .long(keywords::PROJECT)
            .help("Project the resulting documents")
            .takes_value(true)
            .required(false),
    ]
}

pub fn delete_args() -> Vec<clap::Arg<'static, 'static>> {
    vec![clap::Arg::with_name(keywords::INPUT_FILTER)
        .long(keywords::INPUT_FILTER)
        .help("The filter to be applied")
        .takes_value(true)
        .required(false)]
}

pub fn stringify_document(
    document: &mongodb::bson::document::Document,
) -> mongodb::bson::document::Document {
    document
        .iter()
        .map(|(key, value)| (key.clone(), stringify_bson(value)))
        .collect()
}

pub fn stringify_bson(document: &mongodb::bson::Bson) -> mongodb::bson::Bson {
    match document {
        mongodb::bson::Bson::ObjectId(id) => mongodb::bson::Bson::String(id.to_string()),
        mongodb::bson::Bson::DateTime(d) => mongodb::bson::Bson::String(d.to_chrono().to_rfc3339()),
        o => o.clone(),
    }
}

pub fn create_values_from_reader<R>(
    reader: R,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>>
where
    R: std::io::BufRead,
{
    let mut vec = Vec::new();
    for result in serde_json::Deserializer::from_reader(reader).into_iter::<serde_json::Value>() {
        let value = result?;
        match value {
            serde_json::Value::Array(mut arr) => {
                vec.reserve(arr.len());
                vec.append(&mut arr)
            }
            o => vec.push(o),
        }
    }
    Ok(vec)
}

pub fn convert_json_value_to_bson_document(
    json: &serde_json::Value,
) -> Option<mongodb::bson::Document> {
    match json {
        serde_json::Value::Object(o) => Some(
            o.iter()
                .map(|s| (s.0.clone(), convert_json_to_bson(s.1)))
                .collect::<mongodb::bson::Document>(),
        ),
        _ => None,
    }
}

pub fn convert_json_to_bson(json: &serde_json::Value) -> mongodb::bson::Bson {
    match json {
        serde_json::Value::Null => mongodb::bson::Bson::Null,
        serde_json::Value::Bool(b) => mongodb::bson::Bson::Boolean(*b),
        serde_json::Value::Number(n) => mongodb::bson::Bson::Double(
            n.as_f64()
                .expect("Cannot convert JSON number to BSON double"),
        ),
        serde_json::Value::String(s) => mongodb::bson::Bson::String(s.clone()),
        serde_json::Value::Array(v) => {
            mongodb::bson::Bson::Array(v.iter().map(convert_json_to_bson).collect())
        }
        serde_json::Value::Object(o) => mongodb::bson::Bson::Document(
            o.iter()
                .map(|s| (s.0.clone(), convert_json_to_bson(s.1)))
                .collect::<mongodb::bson::Document>(),
        ),
    }
}
