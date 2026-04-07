fn test_im() {
    let mut v = eyeball_im::Vector::new();
    v.push_back(1);
    v.push_back(2);
    v.set(0, 5);
    println!("{:?}", v);
}
