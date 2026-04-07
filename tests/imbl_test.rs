use imbl::Vector;

#[test]
fn test_imbl_insert() {
    let mut v = Vector::new();
    v.push_back(1);
    v.insert(0, 10);
    assert_eq!(v.len(), 2);
    assert_eq!(v[0], 10);
    assert_eq!(v[1], 1);
}

#[test]
fn test_imbl_set() {
    let mut v = Vector::new();
    v.push_back(1);
    v.set(0, 10);
    assert_eq!(v[0], 10);
}
