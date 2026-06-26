use super::*;
use crate::config::StyleConfig;
use crate::settings_spec::SettingSpec;

#[test]
fn palette_options_are_nonempty_and_contain_primary() {
    let options = super::palette_options(&StyleConfig::default());
    assert!(!options.is_empty());
    assert!(options.iter().any(|o| o.token == "primary"));
}

#[test]
fn consumer_color_spec_carries_opacity_and_palette_ref() {
    let section = section("Bar", &StyleConfig::default());
    let spec = section
        .settings
        .iter()
        .find_map(|s| match s {
            SettingSpec::Color(c) if c.path == ["style", "bar", "widget-border-color"] => Some(c),
            _ => None,
        })
        .expect("widget-border-color color spec");
    assert_eq!(spec.opacity_default, 8);
    assert!(spec.is_palette_ref);
    assert_eq!(
        spec.opacity_path,
        vec!["style", "bar", "widget-border-opacity"]
    );
}

#[test]
fn consumer_color_spec_carries_default_token() {
    let section = section("Bar", &StyleConfig::default());

    // palette-defaulted: widget-border-color should have a default_token
    let border_spec = section
        .settings
        .iter()
        .find_map(|s| match s {
            SettingSpec::Color(c) if c.path == ["style", "bar", "widget-border-color"] => Some(c),
            _ => None,
        })
        .expect("widget-border-color color spec");
    assert_eq!(border_spec.default_token, Some("outline-variant"));

    // literal-defaulted: widget-background-color should have no default_token
    let bg_spec = section
        .settings
        .iter()
        .find_map(|s| match s {
            SettingSpec::Color(c) if c.path == ["style", "bar", "widget-background-color"] => {
                Some(c)
            }
            _ => None,
        })
        .expect("widget-background-color color spec");
    assert_eq!(bg_spec.default_token, None);
}
