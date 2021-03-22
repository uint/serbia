// We use trybuild so that we can also test for what compiles and what doesn't.
// Each test case is a file in the `trybuild` directory and essentially a minimal application.
//
// https://github.com/dtolnay/trybuild

#[test]
fn trybuild() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/01-regular-struct-roundtrip.rs");
    t.pass("tests/trybuild/02-tuple-struct-roundtrip.rs");
    t.pass("tests/trybuild/03-enum-roundtrip.rs");
    t.pass("tests/trybuild/04-type-alias.rs");
}
