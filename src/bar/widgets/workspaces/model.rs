#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSummary {
    pub id: u64,
    pub idx: u8,
    pub name: Option<String>,
    pub output: Option<String>,
    pub is_active: bool,
    pub is_focused: bool,
    pub is_urgent: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawWorkspace {
    pub id: u64,
    pub idx: u8,
    pub name: Option<String>,
    pub output: Option<String>,
    pub is_active: bool,
    pub is_focused: bool,
    pub is_urgent: bool,
}

impl WorkspaceSummary {
    pub fn from_wayle_workspace(workspace: &wayle_niri::core::Workspace) -> Self {
        Self {
            id: workspace.id.get(),
            idx: workspace.idx.get(),
            name: workspace.name.get(),
            output: workspace.output.get(),
            is_active: workspace.is_active.get(),
            is_focused: workspace.is_focused.get(),
            is_urgent: workspace.is_urgent.get(),
        }
    }

    pub fn formatted_label(&self, format: &str) -> String {
        let index = self.idx.to_string();
        let title = self
            .name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or("");
        let title_or_index = self
            .name
            .as_ref()
            .filter(|name| !name.is_empty())
            .cloned()
            .unwrap_or_else(|| index.clone());

        let mut output = String::new();
        let mut chars = format.chars();

        while let Some(ch) = chars.next() {
            if ch != '%' {
                output.push(ch);
                continue;
            }

            match chars.next() {
                Some('I') => output.push_str(&index),
                Some('T') => output.push_str(title),
                Some('L') => output.push_str(&title_or_index),
                Some('%') => output.push_str("%"),
                Some(unknown) => {
                    output.push('%');
                    output.push(unknown);
                }
                None => output.push('%'),
            }
        }

        output
    }
}

impl From<RawWorkspace> for WorkspaceSummary {
    fn from(workspace: RawWorkspace) -> Self {
        Self {
            id: workspace.id,
            idx: workspace.idx,
            name: workspace.name,
            output: workspace.output,
            is_active: workspace.is_active,
            is_focused: workspace.is_focused,
            is_urgent: workspace.is_urgent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(name: Option<&str>) -> WorkspaceSummary {
        WorkspaceSummary {
            id: 10,
            idx: 3,
            name: name.map(str::to_string),
            output: Some("DP-1".to_string()),
            is_active: true,
            is_focused: true,
            is_urgent: false,
        }
    }

    #[test]
    fn formatted_label_supports_index() {
        assert_eq!(workspace(Some("code")).formatted_label("%I"), "3");
    }

    #[test]
    fn formatted_label_supports_title() {
        assert_eq!(workspace(Some("code")).formatted_label("%T"), "code");
    }

    #[test]
    fn formatted_label_uses_empty_title_when_name_is_missing() {
        assert_eq!(workspace(None).formatted_label("%I: %T"), "3: ");
    }

    #[test]
    fn formatted_label_supports_composite_formats() {
        assert_eq!(workspace(Some("code")).formatted_label("%I: %T"), "3: code");
    }

    #[test]
    fn formatted_label_supports_literal_percent() {
        assert_eq!(workspace(Some("code")).formatted_label("%%%I"), "%3");
    }

    #[test]
    fn formatted_label_preserves_unknown_tokens() {
        assert_eq!(workspace(Some("code")).formatted_label("%X %I"), "%X 3");
    }

    #[test]
    fn formatted_label_preserves_trailing_percent() {
        assert_eq!(workspace(Some("code")).formatted_label("%I%"), "3%");
    }
}
