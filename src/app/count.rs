use crate::shared::{keywords, Config, MongoDbCommand};

pub fn count_app() -> clap::App<'static, 'static> {
    clap::App::new(MongoDbCommand::Count.to_str())
        .about("Returns the number of documents that match a given filter")
        .arg(
            clap::Arg::with_name(keywords::INPUT_FILTER)
                .long(keywords::INPUT_FILTER)
                .help("The filter to be applied")
                .takes_value(true)
                .required(false),
        )
}

pub fn handler(
    matches: &clap::ArgMatches,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = mongodb::sync::Client::with_uri_str(&config.connection_uri)?;
    let database = client.database(&config.database_name);
    let collection =
        database.collection::<mongodb::bson::document::Document>(&config.collection_name);
    let count_filter = matches
        .value_of(keywords::INPUT_FILTER)
        .map(|s| mongodb::bson::from_slice(s.as_bytes()))
        .transpose()?;
    let count_options = mongodb::options::CountOptions::builder().build();
    let count = collection.count_documents(count_filter, count_options)?;
    println!("{}", count);
    Ok(())
}
