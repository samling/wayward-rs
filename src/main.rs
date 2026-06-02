mod bar;
mod config;
mod file_watch;
mod osd;
mod services;
mod shell;
mod style;

use relm4::RelmApp;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("wayward starting");

    let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    let _guard = runtime.enter();

    let services = runtime.block_on(services::init_shell_services());

    let app = RelmApp::new("dev.sboynton.wayward").visible_on_activate(false);
    if let Some(style) = style::apply_initial_css() {
        style::start_hot_reload(style);
    }

    app.run::<shell::Shell>(shell::ShellInit { services });

    runtime.shutdown_background();
}
