mod bar;
mod config;
mod file_watch;
mod osd;
mod services;
mod shell;
mod style;

use relm4::RelmApp;

fn main() {
    if std::env::args().any(|arg| arg == "--print-default-style-config") {
        print!("{}", style::default_style_config());
        return;
    }

    tracing_subscriber::fmt::init();
    tracing::info!("wayward starting");

    config::ensure_config_files();

    let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    let _guard = runtime.enter();

    let services = runtime.block_on(services::init_shell_services());

    let app = RelmApp::new("dev.sboynton.wayward").visible_on_activate(false);
    let style = style::apply_initial_css();

    app.run::<shell::Shell>(shell::ShellInit { services, style });

    runtime.shutdown_background();
}
