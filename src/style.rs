use std::{fs, path::PathBuf};

const DEFAULT_CSS: &str = r#"
window,
.bar {
    background: #202124;
    color: #f1f3f4;
    font: 12px sans-serif;
}

.bar-item {
    padding: 0 8px;
}

.bar-region {
    background: transparent;
}

.battery {
    opacity: 0.85;
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

pub fn apply_initial_css() {
    relm4::set_global_css(&load_css());
}

pub fn start_hot_reload() {
    let Some(dir) = style_dir() else {
        tracing::info!("Could not determine config directory, style hot reload disabled");
        return;
    };

    let Some(path) = style_path() else {
        tracing::info!("Could not determine style path, styhle hot reload disabled");
        return;
    };

    crate::file_watch::start_debounced_file_watch("style", dir, path, || {
        relm4::gtk::glib::MainContext::default().invoke(|| {
            apply_initial_css();
            tracing::info!("Reloaded style");
        })
    });
}

fn load_css() -> String {
    let mut css = DEFAULT_CSS.to_string();

    let Some(path) = style_path() else {
        tracing::info!("Could not determine config directory, using default style");
        return css;
    };

    match fs::read_to_string(&path) {
        Ok(user_css) => {
            css.push('\n');
            css.push_str(&user_css);
            css
        }
        Err(_) => {
            tracing::info!(
                "No style file found at {}, using default style",
                path.display()
            );
            css
        }
    }
}

fn style_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("wayward"))
}

fn style_path() -> Option<PathBuf> {
    style_dir().map(|dir| dir.join("style.css"))
}
