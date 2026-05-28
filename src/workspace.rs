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
    pub fn label(&self) -> String {
        self.name
            .as_ref()
            .filter(|name| !name.is_empty())
            .cloned()
            .unwrap_or_else(|| self.idx.to_string())
    }

    pub fn css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["workspace"];

        if self.is_active {
            classes.push("active");
        }

        if self.is_focused {
            classes.push("focused");
        }

        if self.is_urgent {
            classes.push("urgent");
        }

        classes
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

    #[test]
    fn label_prefers_name() {
        let workspace = WorkspaceSummary {
            id: 10,
            idx: 3,
            name: Some("code".to_string()),
            output: Some("DP-1".to_string()),
            is_active: true,
            is_focused: true,
            is_urgent: false,
        };

        assert_eq!(workspace.label(), "code");
    }

    #[test]
    fn label_falls_back_to_index() {
        let workspace = WorkspaceSummary {
            id: 10,
            idx: 3,
            name: None,
            output: Some("DP-1".to_string()),
            is_active: true,
            is_focused: false,
            is_urgent: true,
        };

        assert_eq!(workspace.label(), "3");
    }

    #[test]
    fn css_classes_include_state() {
        let workspace = WorkspaceSummary {
            id: 10,
            idx: 3,
            name: None,
            output: None,
            is_active: true,
            is_focused: true,
            is_urgent: true,
        };

        assert_eq!(
            workspace.css_classes(),
            vec!["workspace", "active", "focused", "urgent"]
        );
    }
}