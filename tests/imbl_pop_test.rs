use imbl::Vector;

#[test]
fn test_imbl_pop() {
    let mut v: Vector<i32> = Vector::new();
    v.pop_front();
    v.pop_back();
}
