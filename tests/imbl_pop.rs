use imbl::Vector;
fn main() {
    let mut v: Vector<i32> = Vector::new();
    let res = v.pop_front();
    println!("{:?}", res);
}
