use super::{CssValueKind, CssVariableSpec};

pub(super) const CSS_VARIABLES: &[CssVariableSpec] = &[
    CssVariableSpec {
        group: "notifications",
        key: "body-font-weight",
        variable: "--notification-body-font-weight",
        kind: CssValueKind::Integer { unit: "" },
    },
    CssVariableSpec {
        group: "notifications",
        key: "normal-border-width",
        variable: "--notification-normal-border-width",
        kind: CssValueKind::Integer { unit: "px" },
    },
    CssVariableSpec {
        group: "notifications",
        key: "list-icon-size",
        variable: "--notification-list-icon-size",
        kind: CssValueKind::Integer { unit: "px" },
    },
    CssVariableSpec {
        group: "notifications",
        key: "hide-scrollbar",
        variable: "--notification-scrollbar-opacity",
        kind: CssValueKind::Bool {
            true_value: "0",
            false_value: "1",
        },
    },
    CssVariableSpec {
        group: "notifications",
        key: "font-family",
        variable: "--notification-font-family",
        kind: CssValueKind::String { quoted: true },
    },
    CssVariableSpec {
        group: "bar",
        key: "font-family",
        variable: "--bar-font-family",
        kind: CssValueKind::String { quoted: true },
    },
    CssVariableSpec {
        group: "bar",
        key: "font-size",
        variable: "--bar-font-size",
        kind: CssValueKind::Integer { unit: "px" },
    },
    CssVariableSpec {
        group: "bar",
        key: "font-weight",
        variable: "--bar-font-weight",
        kind: CssValueKind::Integer { unit: "" },
    },
    CssVariableSpec {
        group: "bar",
        key: "item-padding-x",
        variable: "--bar-item-padding-x",
        kind: CssValueKind::Integer { unit: "px" },
    },
    CssVariableSpec {
        group: "bar",
        key: "item-content-margin-y",
        variable: "--bar-item-content-margin-y",
        kind: CssValueKind::Integer { unit: "px" },
    },
];

