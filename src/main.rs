#![warn(clippy::pedantic)]

pub mod alert;
pub mod app;
pub mod cli;
pub mod constants;
pub mod event;
pub mod opensnitch_json;
pub mod opensnitch_proto;
pub mod operator_util;
pub mod serde_impl;
pub mod server;
pub mod ui;

/// Main.
/// # Errors
/// Returns an error if there was bad input at init.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let matches = cli::setup().get_matches();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = app::App::new(
        matches.get_one::<String>("ip_port").unwrap(),
        matches.get_one::<String>("default_action").unwrap(),
        matches.get_one::<String>("temp_rule_lifetime").unwrap(),
        matches.get_one::<u64>("dispo_seconds").unwrap(),
        matches.get_one::<String>("rule_presets").unwrap(),
    )
    .expect("Initialization failed: ");
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
