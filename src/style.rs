use crate::config::{StyleConfig, variables::CssVariables};
use futures::{StreamExt, channel::mpsc};
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use relm4::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, gdk, style_context_add_provider_for_display,
};

const DEFAULT_CSS: &str = include_str!("style.css");

pub(crate) fn default_style_config() -> &'static str {
    DEFAULT_CSS
}

pub(crate) fn generated_style_config(style: &StyleConfig) -> String {
    let mut css = String::new();

    css.push_str(":root {\n");
    style.write_css_variables(&mut css);
    css.push_str("}\n");

    css
}

#[derive(Clone)]
pub(crate) struct StyleHandle {
    provider: CssProvider,
    current_css: Rc<RefCell<String>>,
    generated_css: Rc<RefCell<String>>,
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
        generated_css: Rc::new(RefCell::new(String::new())),
    })
}

impl StyleHandle {
    fn reload(&self) -> bool {
        let css = load_css_with_generated(&self.generated_css.borrow());

        if *self.current_css.borrow() == css {
            return false;
        }

        self.provider.load_from_string(&css);
        *self.current_css.borrow_mut() = css;
        tracing::info!("Reloaded style");

        true
    }

    pub(crate) fn set_generated_css(&self, css: String) -> bool {
        *self.generated_css.borrow_mut() = css;
        self.reload()
    }
}

pub fn start_hot_reload<F>(handle: StyleHandle, on_reload: F)
where
    F: Fn() + 'static,
{
    let Some(dir) = crate::config::config_dir() else {
        tracing::info!("Could not determine config directory, style hot reload disabled");
        return;
    };

    let (reload_tx, mut reload_rx) = mpsc::unbounded::<()>();

    relm4::spawn_local(async move {
        while reload_rx.next().await.is_some() {
            if handle.reload() {
                on_reload();
            }
        }
    });

    crate::file_watch::start_debounced_watch(
        "style",
        dir,
        notify::RecursiveMode::Recursive,
        move |event| event.paths.iter().any(|path| is_style_reload_path(path)),
        move || {
            let _ = reload_tx.unbounded_send(());
        },
    );
}

fn load_css() -> String {
    load_css_with_generated("")
}

fn load_css_with_generated(generated_css: &str) -> String {
    let mut css = DEFAULT_CSS.to_string();

    if let Some(theme_css) = load_theme_css() {
        css.push('\n');
        css.push_str(&theme_css);
    }

    if !generated_css.is_empty() {
        css.push('\n');
        css.push_str(generated_css);
    }

    css
}

fn load_theme_css() -> Option<String> {
    let config = crate::config::AppConfig::load();
    let theme = config.theme.as_deref()?;
    let path = theme_path(theme)?;

    match fs::read_to_string(&path) {
        Ok(css) => Some(css),
        Err(error) => {
            tracing::error!("Failed to read theme {}: {error}", path.display());
            None
        }
    }
}

fn theme_path(theme: &str) -> Option<PathBuf> {
    if theme.contains('/') || theme.contains('\\') || theme.contains("..") {
        tracing::error!("Ignoring invalid theme name: {theme}");
        return None;
    }

    crate::config::themes_dir().map(|dir| dir.join(format!("{theme}.css")))
}

fn is_style_reload_path(path: &Path) -> bool {
    if crate::config::config_path().as_deref() == Some(path) {
        return true;
    }

    crate::config::themes_dir().is_some_and(|themes_dir| path.starts_with(themes_dir))
}

#[cfg(test)]
#[path = "style_test.rs"]
mod tests;
