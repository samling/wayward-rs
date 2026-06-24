// Pure color math: parsing, alpha extraction, and color+opacity composition.

pub(crate) fn parse_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some((r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some((r, g, b))
            }
            _ => None,
        };
    }

    let inner = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let mut parts = inner.split(',').map(str::trim);
    let r = parts.next()?.parse().ok()?;
    let g = parts.next()?.parse().ok()?;
    let b = parts.next()?.parse().ok()?;
    Some((r, g, b))
}

pub(crate) fn alpha_percent(value: &str) -> u16 {
    let Some(inner) = value.trim().strip_prefix("rgba(").and_then(|v| v.strip_suffix(')')) else {
        return 100;
    };
    let Some(alpha) = inner.split(',').nth(3) else {
        return 100;
    };
    match alpha.trim().parse::<f64>() {
        Ok(alpha) => (alpha * 100.0).round().clamp(0.0, 100.0) as u16,
        Err(_) => 100,
    }
}

pub(crate) fn solid_hex(value: &str) -> Option<String> {
    let (r, g, b) = parse_rgb(value)?;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

pub(crate) fn compose(color: &str, opacity: u16) -> String {
    if color.trim() == "transparent" {
        return "transparent".to_string();
    }
    let Some((r, g, b)) = parse_rgb(color) else {
        return color.to_string();
    };
    if opacity >= 100 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        let alpha = opacity as f64 / 100.0;
        format!("rgba({r}, {g}, {b}, {alpha:.3})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rgb_handles_hex_and_rgba() {
        assert_eq!(parse_rgb("#89b4fa"), Some((137, 180, 250)));
        assert_eq!(parse_rgb("rgba(137, 180, 250, 0.22)"), Some((137, 180, 250)));
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
        assert_eq!(solid_hex("rgba(137, 180, 250, 0.22)").as_deref(), Some("#89b4fa"));
        assert_eq!(solid_hex("#1e1e2e").as_deref(), Some("#1e1e2e"));
        assert_eq!(solid_hex("transparent"), None);
    }

    #[test]
    fn compose_combines_color_and_opacity() {
        assert_eq!(compose("#89b4fa", 22), "rgba(137, 180, 250, 0.220)");
        assert_eq!(compose("#89b4fa", 100), "#89b4fa");
        assert_eq!(compose("rgba(137, 180, 250, 0.5)", 22), "rgba(137, 180, 250, 0.220)");
        assert_eq!(compose("transparent", 40), "transparent");
        assert_eq!(compose("unparseable", 50), "unparseable");
    }
}
