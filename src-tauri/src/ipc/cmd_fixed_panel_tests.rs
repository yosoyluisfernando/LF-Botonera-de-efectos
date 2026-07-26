use super::{valid_library_display, valid_view};

#[test]
fn search_is_a_persistable_fixed_panel_view() {
    assert!(valid_view("buttons"));
    assert!(valid_view("player"));
    assert!(valid_view("search"));
    assert!(!valid_view("library"));
}

#[test]
fn library_display_accepts_only_the_two_product_modes() {
    assert!(valid_library_display("filename"));
    assert!(valid_library_display("metadata"));
    assert!(!valid_library_display("title"));
}
