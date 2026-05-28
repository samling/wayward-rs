mod bar;
mod niri;
mod workspace;

use relm4::RelmApp;

const CSS: &str = r#"
window,
.bar {
    background: #202124;
    color: #f1f3f4;
    font: 12px sans-serif;
}

.workspace {
    padding: 3px 8px;
    border-radius: 4px;
    background: #3c4043;
}

.workspace.active {
    background: #5f6368;
}

.workspace.focused {
    background: #8ab4f8;
    color: #202124;
}

.workspace.urgent {
    background: #f28b82;
    color: #202124;
}

.status {
    color: #fdd664;
}
"#;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("wayward starting");

    let app = RelmApp::new("dev.sboynton.wayward");
    relm4::set_global_css(CSS);
    app.run::<bar::Bar>(());
}
