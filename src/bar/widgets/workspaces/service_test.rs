use super::*;

#[test]
fn clicked_workspace_reference_uses_left_click_workspace_id() {
    assert_eq!(
        clicked_workspace_reference("42", 1),
        Some(WorkspaceReferenceArg::Id(42))
    );
}

#[test]
fn clicked_workspace_reference_ignores_non_left_clicks() {
    assert_eq!(clicked_workspace_reference("42", 2), None);
    assert_eq!(clicked_workspace_reference("42", 3), None);
}

#[test]
fn clicked_workspace_reference_ignores_invalid_workspace_ids() {
    assert_eq!(clicked_workspace_reference("not-a-workspace", 1), None);
}
