use super::valid_view;

#[test]
fn search_is_a_persistable_fixed_panel_view() {
    assert!(valid_view("buttons"));
    assert!(valid_view("player"));
    assert!(valid_view("search"));
    assert!(!valid_view("library"));
}
