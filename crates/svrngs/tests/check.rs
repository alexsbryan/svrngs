//! `svrngs check <folder> [<earlier>]` run as a developer runs it: an app's
//! `schema.sql` held to the rule its database is made by
//! (svrngs-db `schema.rs`), beside what its page asks.

use std::path::Path;
use std::process::Command;

/// An app's folder under `root`, named `name`, with an `index.html` and,
/// given one, a `schema.sql`.
fn app(root: &Path, name: &str, schema: Option<&str>) -> std::path::PathBuf {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><title>Board</title>\n",
    )
    .unwrap();
    if let Some(schema) = schema {
        std::fs::write(dir.join("schema.sql"), schema).unwrap();
    }
    dir
}

/// `svrngs check` with `args`: whether it passed, and what it said.
fn checked(args: &[&Path]) -> (bool, String) {
    let ran = Command::new(env!("CARGO_BIN_EXE_svrngs"))
        .arg("check")
        .args(args)
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&ran.stdout) + String::from_utf8_lossy(&ran.stderr);
    println!("MEASURE svrngs check {args:?}: {said}");
    (ran.status.success(), said.into_owned())
}

/// **`svrngs check` reads schema.sql by svrngs-db's rule**: a schema it
/// takes is named table by table, owned or shared, each file column said;
/// a NOCASE key is refused
/// with its fix, beside the page's findings, and the exit code says so, as
/// a row schema.sql inserts is, since it only declares, and a CHECK, an
/// index of an expression and a table past 32 columns are, each with its
/// fix; a schema that drops a column of the version it follows is refused, and a
/// folder named as that version that is no app's is said, never taken for
/// one with no schema. Plant, watched failing: `checked` not reading
/// schema.sql, and no table is named.
#[test]
fn svrngs_check_reads_schema_sql_by_svrngs_db_s_rule() {
    let root = tempfile::tempdir().unwrap();
    let cards = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);\n";
    let votes = "CREATE TABLE votes (person_key TEXT NOT NULL, card TEXT NOT NULL, yes INTEGER, PRIMARY KEY (person_key, card)) STRICT;\n";
    let board = app(root.path(), "board", Some(&format!("{cards}{votes}")));
    let (passed, said) = checked(&[&board]);
    assert!(passed, "{said}");
    assert!(said.contains("Its database's tables: cards, any member's to change. votes, each row its person's own."), "{said}");
    let photos =
        "CREATE TABLE photos (id TEXT PRIMARY KEY NOT NULL, photo_file TEXT, poster_file TEXT);\n";
    let (passed, said) = checked(&[&app(root.path(), "photos", Some(photos))]);
    assert!(passed && said.contains("photos, any member's to change. photo_file names a file. poster_file names a file."), "{said}");

    let tags = app(
        root.path(),
        "tags",
        Some("CREATE TABLE tags (name TEXT PRIMARY KEY NOT NULL COLLATE NOCASE);\n"),
    );
    let (passed, said) = checked(&[&tags]);
    assert!(!passed, "{said}");
    assert!(
        said.contains("schema.sql: The primary key of tags compares text as NOCASE"),
        "{said}"
    );
    let seeded = app(
        root.path(),
        "seeded",
        Some("CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL);\nINSERT INTO cards VALUES ('c1');\n"),
    );
    let (passed, said) = checked(&[&seeded]);
    assert!(!passed, "{said}");
    assert!(
        said.contains("schema.sql: schema.sql asks SQLite for Insert"),
        "{said}"
    );

    let wide = (1..33).map(|i| format!(", c{i} TEXT")).collect::<String>();
    for (name, schema, words) in [
        (
            "checked",
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, n INTEGER CHECK (n > 0));\n".into(),
            "The table cards has a CHECK",
        ),
        (
            "sorted",
            format!("{cards}CREATE INDEX cards_t ON cards (title || id);\n"),
            "The index cards_t orders an expression",
        ),
        (
            "wide",
            format!("CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL{wide});\n"),
            "has at most 32 columns",
        ),
    ] {
        let (passed, said) = checked(&[&app(root.path(), name, Some(&schema))]);
        assert!(
            !passed && said.contains(words) && said.contains("To fix it"),
            "{said}"
        );
    }

    let dropped = app(
        root.path(),
        "dropped",
        Some("CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL);\n"),
    );
    let (passed, said) = checked(&[&dropped, &board]);
    assert!(!passed, "{said}");
    assert!(
        said.contains("schema.sql drops or changes cards.title, which the version it follows has"),
        "{said}"
    );
    assert!(
        checked(&[&board, &dropped]).0,
        "adding title back to the version that lacks it grows"
    );

    let (passed, said) = checked(&[&board, &root.path().join("nowhere")]);
    assert!(
        !passed && said.contains("nowhere holds no index.html"),
        "{said}"
    );
    assert!(
        checked(&[&board, &app(root.path(), "plain", None)]).0,
        "a database where there was none grows"
    );
}
