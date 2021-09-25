use std::convert::TryFrom;

use crate::shared::{keywords, stringify_document, Config, MongoDbCommand};

pub fn aggregate_app() -> clap::App<'static, 'static> {
    clap::App::new(MongoDbCommand::Aggregate.to_str())
        .about("Perform aggregation on a collection")
        .arg(
            clap::Arg::with_name(keywords::PIPELINE)
                .long(keywords::PIPELINE)
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
            clap::Arg::with_name(keywords::PIPELINE_INDEX)
                .long(keywords::PIPELINE_INDEX)
                .help(
                    "Index of the pipeline to be called. \
                    See --list",
                )
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(keywords::LIST)
                .help(
                    "List all available pipelines from the configuration file. \
                    Will be empty if '--config-file' is not passed. \
                    In this case, please pass the pipeline directly to be executed through '--pipeline'",
                )
                .long(keywords::LIST)
                .required(false),
        )
}

pub fn handler(
    aggregate_matches: &clap::ArgMatches,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = mongodb::sync::Client::with_uri_str(&config.connection_uri)?;
    let database = client.database(&config.database_name);
    let collection =
        database.collection::<mongodb::bson::document::Document>(&config.collection_name);
    if aggregate_matches.is_present(keywords::LIST) {
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
    } else if let Some(pipeline_str) = aggregate_matches.value_of(keywords::PIPELINE) {
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
                    println!("{}", stringify_document(&result?));
                }
            }
            _ => {
                return Err("Aggregation pipeline must be an array".into());
            }
        };
    } else if let Some(pipeline_index) = aggregate_matches.value_of(keywords::PIPELINE_INDEX) {
        let index = usize::from_str_radix(pipeline_index, 10)?;
        let pipeline_count = config.pipelines.len();

        if let Some(pipeline) = config.pipelines.into_iter().nth(index) {
            let cursor = collection.aggregate(pipeline.stages, None)?;
            for result in cursor {
                println!("{}", stringify_document(&result?));
            }
        } else {
            return Err(format!(
                "There are only {} pipeline{} available. \
            Note that it is 0-indexed",
                pipeline_count,
                if pipeline_count == 1 { "" } else { "s" },
            )
            .into());
        }
    } else {
    };
    Ok(())
}
