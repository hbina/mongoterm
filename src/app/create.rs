use crate::shared::{
    convert_json_value_to_bson_document, create_values_from_reader, keywords, stringify_bson,
    Config, InputType, InsertResult, MongoDbCommand,
};

pub fn create_app() -> clap::App<'static, 'static> {
    clap::App::new(MongoDbCommand::Create.to_str())
        .about("Insert documents into the collection")
        .arg(
            clap::Arg::with_name(keywords::INPUT_DOCUMENTS)
                .long(keywords::INPUT_DOCUMENTS)
                .help("Get the documents directly as an argument. Supports JSON lines")
                .takes_value(true)
                .required(false),
        )
        .arg(
            clap::Arg::with_name(keywords::INPUT_FILE)
                .long(keywords::INPUT_FILE)
                .help("Get the documents from a file. Support JSON lines")
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
    let handle = if let Some(file) = matches.value_of(keywords::INPUT_FILE) {
        InputType::BufReader(std::io::BufReader::new(std::fs::File::open(file)?))
    } else if let Some(arg) = matches.value_of(keywords::INPUT_DOCUMENTS) {
        // TODO: Possible to avoid allocation here?
        InputType::Arg(arg.to_string())
    } else if !atty::is(atty::Stream::Stdin) {
        InputType::Stdin(std::io::stdin())
    } else {
        return Err("Please provide an input either by piping something in or specifying a file with '--input-file <file>'".into());
    };
    let doc = match handle {
        InputType::Stdin(s) => create_values_from_reader(s.lock()),
        InputType::Arg(s) => create_values_from_reader(std::io::BufReader::new(s.as_bytes())),
        InputType::BufReader(b) => create_values_from_reader(b),
    }?;
    let result = match doc.as_slice() {
        [doc] => InsertResult::One(collection.insert_one(
            convert_json_value_to_bson_document(doc).expect("Only documents can be inserted"),
            None,
        )?),
        o => InsertResult::Many(
            collection.insert_many(
                o.iter()
                    .map(|s| convert_json_value_to_bson_document(s))
                    .collect::<Option<Vec<_>>>()
                    .expect("Only documents can be inserted"),
                None,
            )?,
        ),
    };
    match result {
        InsertResult::One(o) => println!(
            "Successfully inserted one document with _id:{}\n",
            stringify_bson(&o.inserted_id)
        ),
        InsertResult::Many(m) => {
            println!(
                "Successfully inserted {} document{} with _id:",
                m.inserted_ids.len(),
                if m.inserted_ids.len() == 1 { "" } else { "s" },
            );
            m.inserted_ids
                .values()
                .for_each(|s| println!("{}", stringify_bson(s)));
        }
    }
    Ok(())
}
