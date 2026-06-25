use super::*;

#[test]
fn center_dropdowns_do_not_get_screen_edge_margin() {
    assert_eq!(
        margin_start_for_placement(BarEdge::Top, BarRegion::Center),
        0
    );
    assert_eq!(margin_end_for_placement(BarEdge::Top, BarRegion::Center), 0);
    assert_eq!(
        margin_top_for_placement(BarEdge::Left, BarRegion::Center),
        0
    );
    assert_eq!(
        margin_bottom_for_placement(BarEdge::Left, BarRegion::Center),
        0
    );
}

#[test]
fn end_dropdown_on_top_bar_gets_right_edge_margin() {
    assert_eq!(
        margin_end_for_placement(BarEdge::Top, BarRegion::End),
        DROPDOWN_GAP
    );
    assert_eq!(margin_start_for_placement(BarEdge::Top, BarRegion::End), 0);
}
