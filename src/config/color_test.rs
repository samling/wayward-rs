use super::*;

#[test]
fn parse_rgb_handles_hex_and_rgba() {
    assert_eq!(parse_rgb("#89b4fa"), Some((137, 180, 250)));
    assert_eq!(
        parse_rgb("rgba(137, 180, 250, 0.22)"),
        Some((137, 180, 250))
    );
    assert_eq!(parse_rgb("rgb(30, 30, 46)"), Some((30, 30, 46)));
    assert_eq!(parse_rgb("transparent"), None);
    assert_eq!(parse_rgb("not a color"), None);
}

#[test]
fn alpha_percent_reads_rgba_alpha() {
    assert_eq!(alpha_percent("rgba(137, 180, 250, 0.22)"), 22);
    assert_eq!(alpha_percent("rgba(30, 30, 46, 0.96)"), 96);
    assert_eq!(alpha_percent("#89b4fa"), 100);
    assert_eq!(alpha_percent("rgb(1, 2, 3)"), 100);
}

#[test]
fn solid_hex_drops_alpha() {
    assert_eq!(
        solid_hex("rgba(137, 180, 250, 0.22)").as_deref(),
        Some("#89b4fa")
    );
    assert_eq!(solid_hex("#1e1e2e").as_deref(), Some("#1e1e2e"));
    assert_eq!(solid_hex("transparent"), None);
}

#[test]
fn compose_combines_color_and_opacity() {
    assert_eq!(compose("#89b4fa", 22), "rgba(137, 180, 250, 0.220)");
    assert_eq!(compose("#89b4fa", 100), "#89b4fa");
    assert_eq!(
        compose("rgba(137, 180, 250, 0.5)", 22),
        "rgba(137, 180, 250, 0.220)"
    );
    assert_eq!(compose("transparent", 40), "transparent");
    assert_eq!(compose("unparseable", 50), "unparseable");
}
