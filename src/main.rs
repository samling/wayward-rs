mod bar;
mod config;
mod file_watch;
mod niri;
mod style;
mod workspace;

use relm4::RelmApp;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("wayward starting");

    let app = RelmApp::new("dev.sboynton.wayward");
    style::apply_initial_css();
    style::start_hot_reload();

    let config = config::AppConfig::load();
    let init = bar::BarInit::from_config(config.first_bar());
    app.run::<bar::Bar>(init);
}
