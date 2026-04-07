use imbl::Vector;

#[test]
fn test_imbl_methods() {
    let mut v = Vector::new();
    v.push_back(1);
    v.push_back(2);
    v.push_back(3);

    // Test remove
    v.remove(1);
    assert_eq!(v.len(), 2);
    assert_eq!(v[0], 1);
    assert_eq!(v[1], 3);

    // Test set
    v.set(1, 4);
    assert_eq!(v[1], 4);

    // Test push_front
    v.push_front(0);
    assert_eq!(v[0], 0);

    // Test pop_front
    v.pop_front();
    assert_eq!(v[0], 1);

    // Test pop_back
    v.pop_back();
    assert_eq!(v.len(), 1);

    // Test truncate
    v.push_back(2);
    v.push_back(3);
    v.truncate(2);
    assert_eq!(v.len(), 2);

    // Test clear
    v.clear();
    assert_eq!(v.len(), 0);
}
