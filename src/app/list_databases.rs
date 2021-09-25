use crate::shared::{Config, MongoDbCommand};

pub fn list_databases_app() -> clap::App<'static, 'static> {
    clap::App::new(MongoDbCommand::ListDatabases.to_str())
        .about("find all the documents that matches the given filter")
}

// TODO: Print as a table? Prettify option?
pub fn handler(_: &clap::ArgMatches, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let client = mongodb::sync::Client::with_uri_str(&config.connection_uri)?;
    let result = client.list_databases(None, None)?;
    for x in result {
        println!("{:#?}", x);
    }
    Ok(())
}
