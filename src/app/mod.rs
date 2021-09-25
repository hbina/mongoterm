use crate::shared::{keywords, Config, MongoDbCommand};

mod aggregate;
mod count;
mod create;
mod delete_many;
mod delete_one;
mod find_many;
mod find_one;
mod list_databases;

pub fn main_app() -> clap::App<'static, 'static> {
    clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .subcommand(aggregate::aggregate_app())
        .subcommand(create::create_app())
        .subcommand(find_many::find_many_app())
        .subcommand(find_one::find_one_app())
        .subcommand(count::count_app())
        .subcommand(delete_many::delete_many_app())
        .subcommand(delete_one::delete_one_app())
        .subcommand(list_databases::list_databases_app())
        .arg(
            clap::Arg::with_name(keywords::CONNECTION_URI)
                .long(keywords::CONNECTION_URI)
                .required(false)
                .multiple(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(keywords::COLLECTION_NAME)
                .long(keywords::COLLECTION_NAME)
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(keywords::DATABASE_NAME)
                .long(keywords::DATABASE_NAME)
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(keywords::CONFIG_FILE)
                .long(keywords::CONFIG_FILE)
                .required(false)
                .takes_value(true),
        )
}

pub fn to_handler(
    input: clap::ArgMatches,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(matches) = input.subcommand_matches(MongoDbCommand::Aggregate.to_str()) {
        aggregate::handler(matches, config)?;
    } else if let Some(matches) = input.subcommand_matches(MongoDbCommand::Create.to_str()) {
        create::handler(matches, config)?;
    } else if let Some(matches) = input.subcommand_matches(MongoDbCommand::FindMany.to_str()) {
        find_many::handler(matches, config)?;
    } else if let Some(matches) = input.subcommand_matches(MongoDbCommand::FindOne.to_str()) {
        find_one::handler(matches, config)?;
    } else if let Some(matches) = input.subcommand_matches(MongoDbCommand::DeleteOne.to_str()) {
        delete_one::handler(matches, config)?
    } else if let Some(matches) = input.subcommand_matches(MongoDbCommand::DeleteMany.to_str()) {
        delete_many::handler(matches, config)?;
    } else if let Some(matches) = input.subcommand_matches(MongoDbCommand::Count.to_str()) {
        count::handler(matches, config)?;
    } else if let Some(matches) = input.subcommand_matches(MongoDbCommand::ListDatabases.to_str()) {
        list_databases::handler(matches, config)?;
    } else if let Some(subcommand) = input.subcommand_name() {
        return Err(format!(
            "There are no subcommand '{}'. Please see --help",
            subcommand
        )
        .into());
    } else {
        main_app().print_long_help()?;
    }
    Ok(())
}
