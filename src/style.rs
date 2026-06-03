use futures::{StreamExt, channel::mpsc};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

use relm4::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, gdk, style_context_add_provider_for_display,
};

const DEFAULT_CSS: &str = include_str!("style.css");

#[derive(Clone)]
pub(crate) struct StyleHandle {
    provider: CssProvider,
    current_css: Rc<RefCell<String>>,
}

pub fn apply_initial_css() -> Option<StyleHandle> {
    let display = gdk::Display::default()?;
    let provider = CssProvider::new();
    let css = load_css();

    provider.load_from_string(&css);
    style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_USER);

    Some(StyleHandle {
        provider,
        current_css: Rc::new(RefCell::new(css)),
    })
}

impl StyleHandle {
    fn reload(&self) {
        let css = load_css();

        if *self.current_css.borrow() == css {
            return;
        }

        self.provider.load_from_string(&css);
        *self.current_css.borrow_mut() = css;
        tracing::info!("Reloaded style");
    }
}

pub fn start_hot_reload(handle: StyleHandle) {
    let Some(dir) = style_dir() else {
        tracing::info!("Could not determine config directory, style hot reload disabled");
        return;
    };

    let Some(path) = style_path() else {
        tracing::info!("Could not determine style path, styhle hot reload disabled");
        return;
    };

    let (reload_tx, mut reload_rx) = mpsc::unbounded::<()>();

    relm4::spawn_local(async move {
        while reload_rx.next().await.is_some() {
            handle.reload();
        }
    });

    crate::file_watch::start_debounced_file_watch("style", dir, path, move || {
        let _ = reload_tx.unbounded_send(());
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
