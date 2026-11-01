#[test]
fn mean_works() {
    assert_eq!(stats::mean(&[1.0, 2.0, 3.0]), Some(2.0));
    assert_eq!(stats::mean(&[]), None);
}
