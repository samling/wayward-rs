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
#[path = "model_test.rs"]
mod tests;
