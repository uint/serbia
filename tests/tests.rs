use serbia::serbia;

#[serbia]
fn bar() -> u32 {
    2 + 2
}

#[test]
fn it_works() {
    assert_eq!(4, bar());
}