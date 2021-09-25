use crate::shared::{
    convert_json_value_to_bson_document, find_one_args, keywords, stringify_document, Config,
    MongoDbCommand,
};

pub fn find_one_app() -> clap::App<'static, 'static> {
    clap::App::new(MongoDbCommand::FindOne.to_str())
        .about("Find the first document that matches a given filter")
        .args(&find_one_args())
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
    let find_project = matches
        .value_of(keywords::PROJECT)
        .map(|s| serde_json::from_slice::<serde_json::Value>(s.as_bytes()))
        .transpose()?
        .map(|v| convert_json_value_to_bson_document(&v))
        .unwrap_or_default();
    let find_one_options = mongodb::options::FindOneOptions::builder()
        .projection(find_project)
        .build();
    let cursor = collection.find_one(find_filter, find_one_options)?;
    if let Some(result) = cursor {
        println!("{}", stringify_document(&result));
    } else {
        println!("No such documents");
    }
    Ok(())
}
