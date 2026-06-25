use super::*;

#[test]
fn slide_progress_is_linear() {
    assert_eq!(
        animation_progress(WorkspaceIndicatorEffect::Slide, 0.5),
        0.5
    );
}

#[test]
fn ease_progress_moves_faster_than_linear_at_halfway() {
    assert_eq!(
        animation_progress(WorkspaceIndicatorEffect::Ease, 0.5),
        0.875
    );
}

#[test]
fn none_progress_finishes_immediately() {
    assert_eq!(
        animation_progress(WorkspaceIndicatorEffect::None, 0.25),
        1.0
    );
}
