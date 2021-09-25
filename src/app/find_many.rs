use crate::shared::{
    convert_json_value_to_bson_document, find_one_args, keywords, stringify_document, Config,
    MongoDbCommand,
};

pub fn find_many_app() -> clap::App<'static, 'static> {
    let mut args = find_one_args();
    args.push(
        clap::Arg::with_name(keywords::LIMIT)
            .long(keywords::LIMIT)
            .help("Limit the result to N documents")
            .takes_value(true)
            .required(false),
    );
    clap::App::new(MongoDbCommand::FindMany.to_str())
        .about("find all the documents that matches the given filter")
        .args(&args)
}

pub fn handler(
    matches: &clap::ArgMatches,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = mongodb::sync::Client::with_uri_str(&config.connection_uri)?;
    let database = client.database(&config.database_name);
    let collection =
        database.collection::<mongodb::bson::document::Document>(&config.collection_name);
    let find_filter = matches
        .value_of(keywords::INPUT_FILTER)
        .map(|s| mongodb::bson::from_slice(s.as_bytes()))
        .transpose()?;
    let find_limit = matches
        .value_of(keywords::LIMIT)
        .map(|s| i64::from_str_radix(s, 10))
        .transpose()?;
    let find_project = matches
        .value_of(keywords::PROJECT)
        .map(|s| serde_json::from_slice::<serde_json::Value>(s.as_bytes()))
        .transpose()?
        .map(|v| convert_json_value_to_bson_document(&v))
        .unwrap_or_default();
    let find_options = mongodb::options::FindOptions::builder()
        .limit(find_limit)
        .projection(find_project)
        .build();
    let cursor = collection.find(find_filter, find_options)?;
    for result in cursor {
        println!("{}", stringify_document(&result?));
    }
    Ok(())
}
