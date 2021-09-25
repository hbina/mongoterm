mod app;
mod shared;

use app::{main_app, to_handler};
use shared::Config;

// TODO: Implement our own error type
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = main_app().get_matches();
    let config = Config::from_matches(&matches)?;
    to_handler(matches, config)?;
    Ok(())
}
