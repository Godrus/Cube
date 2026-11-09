//! Negative tests for the mutation boundary (TEST-06): code that tries to bypass the handles must
//! not compile. Expected compiler output lives in `tests/ui/*.stderr`.

#[test]
fn bypass_attempts_do_not_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
