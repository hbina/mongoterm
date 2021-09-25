use crate::shared::{
    convert_json_value_to_bson_document, delete_args, keywords, Config, MongoDbCommand,
};

pub fn delete_one_app() -> clap::App<'static, 'static> {
    clap::App::new(MongoDbCommand::DeleteOne.to_str())
        .about("Delete the first document that matches a given filter")
        .args(&delete_args())
}

pub fn handler(
    matches: &clap::ArgMatches,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = mongodb::sync::Client::with_uri_str(&config.connection_uri)?;
    let database = client.database(&config.database_name);
    let collection =
        database.collection::<mongodb::bson::document::Document>(&config.collection_name);
    let delete_one_filter = matches
        .value_of(keywords::INPUT_FILTER)
        .map(|s| serde_json::from_slice::<serde_json::Value>(s.as_bytes()))
        .transpose()?
        .map(|v| convert_json_value_to_bson_document(&v))
        .flatten()
        .unwrap_or_default();
    let cursor = collection
        .delete_one(delete_one_filter, None)?
        .deleted_count;
    println!(
        "Deleted {} document{}",
        cursor,
        if cursor == 1 { "" } else { "s" }
    );
    Ok(())
}
