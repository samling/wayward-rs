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
mod tests {
    use super::{DEFAULT_CSS, generated_style_config};
    use crate::config::StyleConfig;
    use crate::config::style::StyleValue;

    #[test]
    fn default_css_has_one_bar_item_base_rule() {
        assert_eq!(DEFAULT_CSS.matches("\n.bar-item {").count(), 1);
    }

    #[test]
    fn default_css_resets_bar_window_minimum_size() {
        let bar_rule = DEFAULT_CSS
            .split("\n.bar {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("bar rule should exist");

        assert!(bar_rule.contains("min-height: 0;"));
        assert!(bar_rule.contains("min-width: 0;"));
        assert!(bar_rule.contains("padding: var(--bar-padding-y, 0px) var(--bar-padding-x, 0px);"));
    }

    #[test]
    fn default_css_bar_orientation_rules_apply_global_size() {
        let horizontal_rule = DEFAULT_CSS
            .split("\n.bar.horizontal {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("horizontal bar rule should exist");
        let vertical_rule = DEFAULT_CSS
            .split("\n.bar.vertical {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("vertical bar rule should exist");

        assert!(horizontal_rule.contains("min-height: var(--bar-size, 24px);"));
        assert!(vertical_rule.contains("min-width: var(--bar-size, 24px);"));
    }

    #[test]
    fn default_css_bar_item_has_no_spacing_padding() {
        let item_rule = DEFAULT_CSS
            .split("\n.bar-item {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("bar item rule should exist");

        assert!(item_rule.contains("padding: 0;"));
        assert!(!item_rule.contains("--bar-item-gap-x"));
        assert!(!item_rule.contains("--bar-item-padding-x"));
    }

    #[test]
    fn default_css_has_one_bar_item_content_base_rule() {
        let content_rule = DEFAULT_CSS
            .split("\n.bar-item-content {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("bar item content rule should exist");

        assert!(
            content_rule.contains(
                "margin: var(--bar-widget-margin-y, 0px) var(--bar-widget-margin-x, 0px);"
            )
        );
        assert!(content_rule.contains(
            "padding: var(--bar-widget-padding-y, 0px) var(--bar-widget-padding-x, 4px);"
        ));
        assert!(!content_rule.contains("--bar-item-content-margin-y"));
        assert!(!content_rule.contains("--bar-item-padding-x"));
    }

    #[test]
    fn default_css_resets_bar_item_content_child_spacing() {
        let child_rule = DEFAULT_CSS
            .split(".bar-item-content label,")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("bar item content child reset should exist");

        assert!(child_rule.contains("margin: 0;"));
        assert!(child_rule.contains("min-height: 0;"));
        assert!(child_rule.contains("padding: 0;"));
    }

    #[test]
    fn default_css_bar_item_state_rule_does_not_override_layout_or_shape() {
        let state_rule = DEFAULT_CSS
            .split(".bar-item:hover,")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("bar item state rule should exist");

        assert!(!state_rule.contains("padding:"));
        assert!(!state_rule.contains("min-height:"));
        assert!(!state_rule.contains("min-width:"));
        assert!(!state_rule.contains("border-radius:"));
        assert!(!state_rule.contains("background:"));
    }

    #[test]
    fn default_css_bar_facing_labels_do_not_override_font_weight() {
        for selector in [".workspace-label", ".updates-count"] {
            let rule = DEFAULT_CSS
                .split(selector)
                .nth(1)
                .and_then(|css| css.split_once('}'))
                .map(|(rule, _)| rule);

            if let Some(rule) = rule {
                assert!(!rule.contains("font-weight:"));
            }
        }
    }

    #[test]
    fn default_css_workspace_indicator_does_not_force_size() {
        let rule = DEFAULT_CSS
            .split("\n.workspace-indicator {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("workspace indicator rule should exist");

        assert!(!rule.contains("min-height:"));
        assert!(!rule.contains("min-width:"));
        assert!(!rule.contains("--workspace-indicator-size"));
    }

    #[test]
    fn default_css_workspaces_overlay_does_not_shrink_item_height() {
        let rule = DEFAULT_CSS
            .split("\n.workspaces-overlay {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("workspaces overlay rule should exist");

        assert!(rule.contains("margin-bottom: 0;"));
        assert!(rule.contains("margin-top: 0;"));
    }

    #[test]
    fn default_css_workspaces_apply_global_vertical_padding_once() {
        let content_rule = DEFAULT_CSS
            .split("\n.workspaces-content {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("workspaces content rule should exist");

        let workspace_rule = DEFAULT_CSS
            .split("\n.workspace {")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("workspace rule should exist");

        assert!(
            content_rule.contains(
                "padding: 0 var(--workspaces-content-padding, var(--bar-widget-padding-x"
            )
        );
        assert!(workspace_rule.contains("padding: var(--bar-widget-padding-y, 0px) 0.65em;"));
    }

    #[test]
    fn default_css_bar_menu_buttons_inherit_font_weight() {
        for selector in [".bar-item {", "menubutton.bar-item > button,"] {
            let rule = DEFAULT_CSS
                .split(selector)
                .nth(1)
                .and_then(|css| css.split_once('}'))
                .map(|(rule, _)| rule)
                .unwrap_or_else(|| panic!("{selector} rule should exist"));

            assert!(rule.contains("font-weight: inherit;"));
        }
    }

    #[test]
    fn default_css_bar_menu_buttons_reset_theme_size() {
        let rule = DEFAULT_CSS
            .split("menubutton.bar-item > button,")
            .nth(1)
            .and_then(|css| css.split_once('}'))
            .map(|(rule, _)| rule)
            .expect("bar menu button reset rule should exist");

        assert!(rule.contains("margin: 0;"));
        assert!(rule.contains("min-height: 0;"));
        assert!(rule.contains("padding: 0;"));
    }

    #[test]
    fn default_css_bar_menu_button_reset_does_not_target_dropdown_buttons() {
        assert!(!DEFAULT_CSS.contains("menubutton.bar-item button,"));
        assert!(!DEFAULT_CSS.contains("menubutton.bar-item button.flat"));
        assert!(DEFAULT_CSS.contains("menubutton.bar-item > button,"));
    }

    #[test]
    fn default_css_does_not_use_negative_margins() {
        assert!(!DEFAULT_CSS.contains("margin: -"));
        assert!(!DEFAULT_CSS.contains("margin-top: -"));
        assert!(!DEFAULT_CSS.contains("margin-bottom: -"));
        assert!(!DEFAULT_CSS.contains("margin-left: -"));
        assert!(!DEFAULT_CSS.contains("margin-right: -"));
        assert!(!DEFAULT_CSS.contains("margin-start: -"));
        assert!(!DEFAULT_CSS.contains("margin-end: -"));
    }

    #[test]
    fn default_css_avoids_unsupported_gtk_properties() {
        for property in [
            "align-items:",
            "align-self:",
            "box-sizing:",
            "max-height:",
            "max-width:",
        ] {
            assert!(
                !DEFAULT_CSS.contains(property),
                "default GTK CSS should not contain unsupported property {property}"
            );
        }
    }

    #[test]
    fn generated_style_config_includes_bar_font_weight() {
        let mut style = StyleConfig::default();
        style
            .bar
            .insert("font-weight".to_string(), StyleValue::Integer(500));

        let css = generated_style_config(&style);

        assert!(css.contains("--bar-font-weight: 500;"));
    }

    #[test]
    fn generated_style_config_includes_bar_layout_controls() {
        let css = generated_style_config(&StyleConfig::default());

        assert!(css.contains("--bar-size: 24px;"));
        assert!(css.contains("--bar-padding-x: 0px;"));
        assert!(css.contains("--bar-padding-y: 0px;"));
        assert!(css.contains("--bar-widget-gap: 4px;"));
        assert!(css.contains("--bar-widget-padding-x: 4px;"));
        assert!(css.contains("--bar-widget-padding-y: 0px;"));
        assert!(css.contains("--bar-widget-margin-x: 0px;"));
        assert!(css.contains("--bar-widget-margin-y: 0px;"));
        assert!(!css.contains("--bar-item-content-margin-y"));
        assert!(!css.contains("--bar-item-padding-x"));
        assert!(!css.contains("--bar-item-gap-x"));
    }

    #[test]
    fn generated_style_config_includes_global_widget_surface_controls() {
        let mut style = StyleConfig::default();
        style.bar.insert(
            "widget-background-color".to_string(),
            StyleValue::String("rgba(255, 255, 255, 0.1)".to_string()),
        );
        style
            .bar
            .insert("widget-border-width".to_string(), StyleValue::Integer(1));

        let css = generated_style_config(&style);

        assert!(css.contains("--bar-widget-background-color: rgba(255, 255, 255, 0.1);"));
        assert!(css.contains("--bar-widget-border-width: 1px;"));
    }

    #[test]
    fn generated_style_config_includes_configured_per_widget_surface_controls() {
        let mut style = StyleConfig::default();
        style.brightness.insert(
            "widget-background-color".to_string(),
            StyleValue::String("rgba(137, 180, 250, 0.2)".to_string()),
        );
        style
            .brightness
            .insert("widget-border-radius".to_string(), StyleValue::Integer(8));

        let css = generated_style_config(&style);

        assert!(css.contains("--brightness-widget-background-color: rgba(137, 180, 250, 0.2);"));
        assert!(css.contains("--brightness-widget-border-radius: 8px;"));
    }

    #[test]
    fn generated_style_config_does_not_emit_unconfigured_per_widget_surface_defaults() {
        let css = generated_style_config(&StyleConfig::default());

        assert!(css.contains("--bar-widget-background-color: transparent;"));
        assert!(!css.contains("--brightness-widget-background-color: transparent;"));
        assert!(!css.contains("--volume-widget-border-width: 0px;"));
    }

    #[test]
    fn generated_style_config_includes_notification_body_font_weight() {
        let mut style = StyleConfig::default();
        style
            .notifications
            .insert("body-font-weight".to_string(), StyleValue::Integer(500));

        let css = generated_style_config(&style);

        assert!(css.contains("--notification-body-font-weight: 500;"));
    }

    #[test]
    fn generated_style_config_includes_notification_indicator_border_width() {
        let mut style = StyleConfig::default();
        style
            .notifications
            .insert("indicator-border-width".to_string(), StyleValue::Integer(2));

        let css = generated_style_config(&style);

        assert!(css.contains("--notification-indicator-border-width: 2px;"));
    }
}
