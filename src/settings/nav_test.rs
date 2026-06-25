use super::super::page::{SettingsConfig, build_page};
use super::*;
use std::collections::HashSet;

#[test]
fn nav_item_keys_are_unique() {
    let mut seen = HashSet::new();
    for group in nav() {
        for item in group.items {
            assert!(seen.insert(item.key), "duplicate nav key: {}", item.key);
        }
    }
}

#[test]
fn default_item_exists() {
    assert!(find_item(DEFAULT_ITEM).is_some());
}

#[test]
fn build_page_for_appearance_item_has_single_section() {
    let item = find_item("palette").unwrap();
    let config = SettingsConfig {
        style: crate::config::StyleConfig::default(),
        widgets: std::collections::BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };
    let page = build_page(item, &config).unwrap();
    assert_eq!(page.title, "Palette");
    assert_eq!(page.sections.len(), 1);
}

#[test]
fn build_page_for_bar_layout_is_none() {
    let item = find_item("bars").unwrap();
    let config = SettingsConfig {
        style: crate::config::StyleConfig::default(),
        widgets: std::collections::BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };
    assert!(build_page(item, &config).is_none());
}
