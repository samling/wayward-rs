use super::*;

#[test]
fn parses_automatic_status() {
    let status = parse_status(
        r#"
 Active preset: default
Current period: Day 󰖨
         State: stable
   Temperature: 6500K
         Gamma: 100.0%
   Next period: 19:18:46 (in 7h6m)
"#,
    )
    .unwrap();

    assert_eq!(status.active_preset, "default");
    assert_eq!(status.current_period.as_deref(), Some("Day 󰖨"));
    assert_eq!(status.state.as_deref(), Some("stable"));
    assert_eq!(status.temperature.as_deref(), Some("6500K"));
    assert_eq!(status.gamma.as_deref(), Some("100.0%"));
    assert_eq!(status.next_period.as_deref(), Some("19:18:46 (in 7h6m)"));
}

#[test]
fn parses_static_status_without_next_period() {
    let status = parse_status(
        r#"
 Active preset: day
Current period: Static 󰋙
         State: static
   Temperature: 6500K
         Gamma: 100.0%
"#,
    )
    .unwrap();

    assert_eq!(status.active_preset, "day");
    assert_eq!(status.current_period.as_deref(), Some("Static 󰋙"));
    assert_eq!(status.next_period, None);
}

#[test]
fn classifies_default_preset_as_automatic() {
    let config = SunsetrConfig::default();
    let status = SunsetrStatus {
        active_preset: "default".to_string(),
        current_period: Some("Day 󰖨".to_string()),
        state: Some("stable".to_string()),
        temperature: Some("6500K".to_string()),
        gamma: Some("100.0%".to_string()),
        next_period: None,
    };

    assert!(matches!(
        SunsetrState::from_status(status, &config),
        SunsetrState::Automatic(_)
    ));
}

#[test]
fn classifies_paused_preset_as_paused() {
    let config = SunsetrConfig::default();
    let status = SunsetrStatus {
        active_preset: "day".to_string(),
        current_period: Some("Static 󰋙".to_string()),
        state: Some("static".to_string()),
        temperature: Some("6500K".to_string()),
        gamma: Some("100.0%".to_string()),
        next_period: None,
    };

    assert!(matches!(
        SunsetrState::from_status(status, &config),
        SunsetrState::Paused(_)
    ));
}
