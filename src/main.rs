use sixtyfps::{Model, SharedString};

mod shared;

sixtyfps::include_modules!();

pub fn main() {
    let mongodb_url_text = std::sync::Arc::new(std::sync::Mutex::new(
        "mongodb://localhost:27017".to_string(),
    ));
    let mongodb_client = std::sync::Arc::new(std::sync::Mutex::new(None));
    let mongodb_database = std::sync::Arc::new(std::sync::Mutex::new(None));
    let mongodb_collection = std::sync::Arc::new(std::sync::Mutex::new(None));

    let app = App::new();
    app.on_mongodb_url_edited({
        let mongodb_url_key = mongodb_url_text.clone();
        move |input| {
            let mut mongodb_url = mongodb_url_key.lock().unwrap();
            *mongodb_url = input.to_string();
        }
    });
    app.on_mongodb_url_clicked({
        let app_weak = app.as_weak();
        let mongodb_url_key = mongodb_url_text.clone();
        let mongodb_client_key = mongodb_client.clone();
        move || {
            let mongodb_url = {
                let mongodb_url = mongodb_url_key.lock().unwrap();
                mongodb_url.clone()
            };
            if let Ok(client) = mongodb::sync::Client::with_uri_str(&mongodb_url) {
                println!("Connected to {}", mongodb_url);
                let databases = client.list_database_names(None, None).unwrap();
                {
                    let app = app_weak.unwrap();
                    app.set_mongodb_url_input_enabled(false);

                    app.set_mongodb_databases(sixtyfps::ModelHandle::new(std::rc::Rc::new(
                        sixtyfps::VecModel::from(
                            databases
                                .iter()
                                .map(|s| SharedString::from(s))
                                .collect::<Vec<_>>(),
                        ),
                    )));
                    app.set_mongodb_database_combobox_enabled(true);
                }
                {
                    let mut mongodb_client = mongodb_client_key.lock().unwrap();
                    *mongodb_client = Some(client);
                }
            } else {
                eprintln!("Cannot connect to {}", mongodb_url);
            };
        }
    });
    app.on_mongodb_database_clicked({
        let app_weak = app.as_weak();
        let mongodb_client_key = mongodb_client.clone();
        let mongodb_database_key = mongodb_database.clone();
        move |input| {
            let mongodb_client = mongodb_client_key.lock().unwrap();
            let mut mongodb_database = mongodb_database_key.lock().unwrap();
            {
                let app = app_weak.unwrap();
                let selected_database = app
                    .get_mongodb_databases()
                    .iter()
                    .find(|c| c == &input)
                    .unwrap();
                let collections = mongodb_client
                    .as_ref()
                    .unwrap()
                    .database(selected_database.as_str())
                    .list_collections(None, None)
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap()
                    .iter()
                    .map(|v| v.name.to_string())
                    .collect::<Vec<_>>();
                *mongodb_database = Some(
                    mongodb_client
                        .as_ref()
                        .unwrap()
                        .database(selected_database.as_str()),
                );
                app.set_mongodb_collections(sixtyfps::ModelHandle::new(std::rc::Rc::new(
                    sixtyfps::VecModel::from(
                        collections
                            .iter()
                            .map(|s| SharedString::from(s))
                            .collect::<Vec<_>>(),
                    ),
                )));
                app.set_mongodb_collection_combobox_enabled(true);
            }
        }
    });
    app.on_mongodb_collection_clicked({
        let app_weak = app.as_weak();
        let mongodb_database_key = mongodb_database.clone();
        let mongodb_collection_key = mongodb_collection.clone();
        move |input| {
            let mongodb_database = mongodb_database_key.lock().unwrap();
            let mut mongodb_collection = mongodb_collection_key.lock().unwrap();
            {
                let app = app_weak.unwrap();
                let selected_collection = app
                    .get_mongodb_collections()
                    .iter()
                    .find(|c| c == &input)
                    .unwrap();
                *mongodb_collection = Some(
                    mongodb_database
                        .as_ref()
                        .unwrap()
                        .collection::<mongodb::bson::document::Document>(
                            selected_collection.as_str(),
                        ),
                );
                app.set_mongodb_run_query_button_enabled(true);
            }
        }
    });
    app.on_mongodb_run_query_clicked({
        let app_weak = app.as_weak();
        let mongodb_collection_key = mongodb_collection.clone();
        move || {
            let mongodb_collection = mongodb_collection_key.lock().unwrap();
            let results = mongodb_collection
                .as_ref()
                .unwrap()
                .aggregate(vec![], None)
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .into_iter()
                .map(|d| SharedString::from(format!("{:#?}", d)))
                .collect::<Vec<_>>();

            let app = app_weak.unwrap();
            app.set_mongodb_documents(sixtyfps::ModelHandle::new(std::rc::Rc::new(
                sixtyfps::VecModel::from(results),
            )));
        }
    });
    app.run();
}
