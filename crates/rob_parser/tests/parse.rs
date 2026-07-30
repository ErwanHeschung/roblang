use rob_parser::parse_to_sexpr;

#[test]
fn precedence() {
    insta::assert_snapshot!(parse_to_sexpr("1 + 2 * 3 - 4").unwrap());
}
