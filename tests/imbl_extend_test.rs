use imbl::Vector;

#[test]
fn test_imbl_extend() {
    let mut v = Vector::new();
    v.push_back(1);
    v.extend(vec![2, 3]);
    assert_eq!(v.len(), 3);
}

#[test]
fn test_imbl_set() {
    let mut v = Vector::new();
    v.push_back(1);
    v.set(0, 10);
    assert_eq!(v[0], 10);
}
