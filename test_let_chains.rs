fn main() {
    let a = Some(1);
    let b = true;
    if let Some(x) = a && b {
        println!("{}", x);
    }
}
