use crate::app::App;

pub mod app;
pub mod constants;
pub mod event;
pub mod opensnitch_proto;
pub mod serde_impl;
pub mod server;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
