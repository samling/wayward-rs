use super::CssVariables;
use crate::config::StyleConfig;

#[test]
fn default_css_variables_match_golden() {
    let mut css = String::new();
    StyleConfig::default().write_css_variables(&mut css);

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden-css-variables.txt"
    );
    if std::path::Path::new(path).exists() {
        let expected = std::fs::read_to_string(path).unwrap();
        assert_eq!(css, expected, "emitted CSS variables changed vs golden");
    } else {
        std::fs::write(path, &css).unwrap();
        panic!("wrote golden snapshot; re-run to assert");
    }
}

#[test]
fn consumer_opacity_defaults_preserve_token_alpha() {
    use super::super::color::alpha_percent;
    // Pre-migration alpha-bearing palette token values.
    const OLD: &[(&str, &str)] = &[
        ("primary-container", "rgba(137, 180, 250, 0.22)"),
        ("secondary-container", "rgba(203, 166, 247, 0.18)"),
        ("tertiary-container", "rgba(253, 214, 100, 0.18)"),
        ("on-surface-variant", "rgba(241, 243, 244, 0.72)"),
        ("surface-container-lowest", "rgba(30, 30, 46, 0.96)"),
        ("surface-container-low", "rgba(241, 243, 244, 0.045)"),
        ("surface-container", "rgba(241, 243, 244, 0.06)"),
        ("surface-container-high", "rgba(241, 243, 244, 0.08)"),
        ("surface-container-highest", "rgba(241, 243, 244, 0.12)"),
        ("outline", "rgba(241, 243, 244, 0.14)"),
        ("outline-variant", "rgba(241, 243, 244, 0.08)"),
        ("error-container", "rgba(242, 139, 130, 0.18)"),
    ];
    let expected = |tok: &str| -> u16 {
        OLD.iter()
            .find(|(t, _)| *t == tok)
            .map(|(_, v)| alpha_percent(v))
            .unwrap_or(100)
    };
    for spec in super::specs::style_settings() {
        if spec.group == "palette" {
            continue;
        }
        if let Some(super::SettingUiSpec::Color {
            default: super::ColorDefault::Palette(tok),
            opacity_default,
            ..
        }) = spec.setting
        {
            assert_eq!(
                opacity_default,
                expected(tok),
                "{}/{} (-> {}) opacity_default {} != expected {}",
                spec.group,
                spec.key,
                tok,
                opacity_default,
                expected(tok)
            );
        }
    }
}

#[test]
fn consumer_colors_always_emit_composed_defaults() {
    let mut css = String::new();
    StyleConfig::default().write_css_variables(&mut css);

    // outline at 14% opacity - preserves old rgba(241,243,244,0.14)
    assert!(
        css.contains("  --osd-border-color: rgba(241, 243, 244, 0.140);"),
        "osd-border-color should emit composed default"
    );
    // primary-container at 22% opacity - preserves old rgba(137,180,250,0.22)
    assert!(
        css.contains("  --workspace-indicator-background-color: rgba(137, 180, 250, 0.220);"),
        "workspace-indicator-background-color should emit composed default"
    );
    // transparent literal stays transparent
    assert!(
        css.contains("  --bar-widget-background-color: transparent;"),
        "bar-widget-background-color should emit transparent"
    );
    // opaque palette token stays as solid hex
    assert!(
        css.contains("  --workspace-focused-color: #89b4fa;"),
        "workspace-focused-color should emit solid hex"
    );
}
