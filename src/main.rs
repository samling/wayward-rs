mod bar;
mod config;
mod file_watch;
mod niri;
mod services;
mod shell;
mod style;
mod workspace;

use relm4::RelmApp;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("wayward starting");

    let app = RelmApp::new("dev.sboynton.wayward").visible_on_activate(false);
    style::apply_initial_css();
    style::start_hot_reload();

    app.run::<shell::Shell>(());
}
