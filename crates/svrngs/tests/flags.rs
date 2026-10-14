//! The flags that take a value (#147): each reads the next argument as its
//! value, and nothing else does.

use svrngs::cli;

fn argv(args: &[&str]) -> Vec<String> {
    std::iter::once("svrngs")
        .chain(args.iter().copied())
        .map(String::from)
        .collect()
}

/// **Each flag taking a value wants one, and an argument that is no flag
/// takes none** (#147): every one of the twelve, last on the line, is
/// refused as wanting a value; and an empty argument before a word leaves
/// the word unexpected, never read as the empty argument's value. Red on
/// the plant the finding names, a doubled space in the space-joined string
/// the flags were kept in: the empty string among the flags, so `""` took
/// `x` as its value.
#[test]
fn each_flag_taking_a_value_wants_one_and_no_other_argument_takes_one() {
    let valued = [
        "--as",
        "--origin",
        "--name",
        "--kind",
        "--limit",
        "--refuse",
        "--upload",
        "--reads",
        "--sign-in",
        "--folder",
        "--group",
        "--ask",
    ];
    for flag in valued {
        let said = cli::parse(argv(&["share", "8080", flag]));
        assert_eq!(said.err(), Some(format!("{flag} wants a value")), "{flag}");
    }
    let said = cli::parse(argv(&["share", "", "x"]));
    assert_eq!(said.err(), Some("unexpected argument `x`".to_string()));
}
