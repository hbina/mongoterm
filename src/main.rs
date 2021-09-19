use std::{convert::TryFrom, str::FromStr};

use serde::{Deserialize, Serialize};

pub fn main_app() -> clap::App<'static, 'static> {
    clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .subcommand(aggregate_app())
        .subcommand(create_app())
        .arg(
            clap::Arg::with_name("connection-uri")
                .long("connection-uri")
                .required(false)
                .multiple(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("collection-name")
                .long("collection-name")
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("database-name")
                .long("database-name")
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("config-file")
                .long("config-file")
                .required(false)
                .takes_value(true),
        )
}

pub fn aggregate_app() -> clap::App<'static, 'static> {
    clap::App::new("aggregate")
        .about("Perform aggregation on a collection")
        .arg(
            clap::Arg::with_name("pipeline")
                .long("pipeline")
                .help("The pipeline to be executed as a string")
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("pipeline-name")
                .long("pipeline-name")
                .help(
                    "Name of the pipeline to be called. \
                    See list",
                )
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("pipeline-index")
                .long("pipeline-index")
                .help(
                    "Index of the pipeline to be called. \
                    See --list",
                )
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("list")
                .help(
                    "List all available pipelines from the configuration file. \
                    Will be empty if '--config-file' is not passed. \
                    In this case, please pass the pipeline directly to be executed through '--pipeline'",
                )
                .long("list")
                .required(false),
        )
}

pub fn create_app() -> clap::App<'static, 'static> {
    clap::App::new("create").arg(
        clap::Arg::with_name("input-documents")
            .help("The document(s) to be created in the collection")
            .required(false),
    )
}

#[derive(Serialize, Deserialize, Debug)]
struct Pipeline {
    name: String,
    description: Option<String>,
    stages: Vec<mongodb::bson::document::Document>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    connection_uri: String,
    collection_name: String,
    database_name: String,
    pipelines: Vec<Pipeline>,
}

#[derive(Debug)]
pub enum InsertResult {
    One(mongodb::results::InsertOneResult),
    Many(mongodb::results::InsertManyResult),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = main_app().get_matches();

    let config = if let Some(config_file) = matches.value_of("config-file") {
        let file = std::fs::File::open(config_file)?;
        serde_json::from_reader(file)?
    } else {
        match (
            matches.value_of("connection-uri"),
            matches.value_of("database-name"),
            matches.value_of("collection-name"),
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
    let client = mongodb::sync::Client::with_uri_str(&config.connection_uri)?;
    let database = client.database(&config.database_name);
    let collection = database.collection::<serde_json::Value>(&config.collection_name);

    if let Some(aggregate_matches) = matches.subcommand_matches("aggregate") {
        if aggregate_matches.is_present("list") {
            // TODO: Improve this to print using a table? But is it queryable?
            config.pipelines.iter().enumerate().for_each(|(idx, p)| {
                println!(
                    "{}. {} {}",
                    idx,
                    p.name,
                    p.description
                        .as_ref()
                        .map(|s| format!("=> {}", s))
                        .unwrap_or_default()
                )
            });
        } else if let Some(pipeline_str) = aggregate_matches.value_of("pipeline") {
            println!("Perform query using pipeline:\n{}", pipeline_str);
            let value = serde_json::from_str::<serde_json::Value>(pipeline_str)?;
            match value {
                serde_json::Value::Array(pipeline) => {
                    let documents = pipeline
                        .into_iter()
                        .map(|s| match s {
                            serde_json::Value::Object(o) => {
                                mongodb::bson::document::Document::try_from(o)
                            }
                            // TODO: Properly handle this error.
                            _ => panic!("Each stage must be a valid object"),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let cursor = collection.aggregate(documents, None)?;
                    for result in cursor {
                        println!("{}", result?);
                    }
                }
                _ => {
                    return Err("Aggregation pipeline must be an array".into());
                }
            };
        } else if let Some(pipeline_index) = aggregate_matches.value_of("pipeline-index") {
            let index = usize::from_str_radix(pipeline_index, 10)?;
            let pipeline_count = config.pipelines.len();

            if let Some(pipeline) = config.pipelines.into_iter().nth(index) {
                let cursor = collection.aggregate(pipeline.stages, None)?;
                for result in cursor {
                    let result = stringify_document(result?);
                    println!("{}", result);
                }
            } else {
                return Err(format!(
                    "There are only {} pipeline{} available. \
                Note that it is 0-indexed",
                    pipeline_count,
                    if pipeline_count > 1 { "s" } else { "" },
                )
                .into());
            }
        } else {
        }
    } else if let Some(create_matches) = matches.subcommand_matches("create") {
        let documents_str = create_matches
            .value_of("input-documents")
            .unwrap()
            .to_string();
        let doc = serde_json::Value::from_str(documents_str.as_str())?;
        let result = match doc {
            serde_json::Value::Array(v) => InsertResult::Many(collection.insert_many(v, None)?),
            o => InsertResult::One(collection.insert_one(o, None)?),
        };
        match result {
            InsertResult::One(o) => println!(
                "Successfully inserted one document with id:{}",
                o.inserted_id
            ),
            InsertResult::Many(m) => {
                println!(
                    "Successfully inserted {} document{} with _id:",
                    m.inserted_ids.len(),
                    if m.inserted_ids.len() > 1 { "s" } else { "" },
                );
                m.inserted_ids
                    .values()
                    .for_each(|s| println!("{}", stringify_bson(s)));
            }
        }
    } else {
        return Err("Only aggregation is supported at the moment".into());
    }
    Ok(())
}

pub fn stringify_document(
    document: mongodb::bson::document::Document,
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
