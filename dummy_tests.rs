use matrix_sdk::ruma::RoomId;

#[test]
fn test_is_in_space_edge_cases() {
    let mut hierarchy = SpaceHierarchy::new();
    let space_id = RoomId::parse("!space:example.com").unwrap();

    // Check itself
    assert!(hierarchy.is_in_space(&space_id, &space_id));
}
