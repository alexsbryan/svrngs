//! What a `schema.sql` may make, as `svrngs check` reads it and as a
//! database is made from it (`src/schema.rs`; svrn-docs APP_STATE, the
//! pitch's first and fourth places; MAGNETS_BUILD S5): each refusal in its
//! words, with its fix, pinned by a piece of those words; and the one
//! decider, the same words where a database is opened. Each names the plant
//! it was watched failing on.

use svrngs_db::{Refused, SigningKey, States, check};

/// `schema` refused by the check: its words, or a panic naming what it took.
fn refused(schema: &str) -> String {
    match check(schema, None) {
        Err(Refused::Schema(words)) => words.to_string(),
        other => panic!("{schema}\nnot refused as a schema: {other:?}"),
    }
}

/// **cr-sqlite's six refusals, each said with its fix**, by its own text
/// (`vendor/cr-sqlite/core/rs/core/src/tableinfo.rs:1044-1106`): none falls
/// to the words for a text svrngs doesn't know. Plant, watched failing:
/// `SHAPES` without its UNIQUE row, and the unique column and index are
/// said as "svrngs has no words for why yet".
#[test]
fn cr_sqlite_s_six_refusals_are_each_said_with_their_fix() {
    let six = [
        (
            "CREATE TABLE notes (body TEXT);",
            "has no primary key, or its key can be NULL",
        ),
        (
            "CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT);",
            "`INTEGER PRIMARY KEY` needs NOT NULL too",
        ),
        (
            "CREATE TABLE seats (row INTEGER NOT NULL, seat INTEGER, PRIMARY KEY (row, seat));",
            "Part of the primary key of seats can be NULL",
        ),
        (
            "CREATE TABLE people (id TEXT PRIMARY KEY NOT NULL, email TEXT UNIQUE);",
            "people has a UNIQUE column or index",
        ),
        (
            "CREATE TABLE people (id TEXT PRIMARY KEY NOT NULL, email TEXT);\nCREATE UNIQUE INDEX people_email ON people (email);",
            "people has a UNIQUE column or index",
        ),
        (
            "CREATE TABLE lists (id TEXT PRIMARY KEY NOT NULL);\nCREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, list TEXT REFERENCES lists (id));",
            "cards has a foreign key (REFERENCES)",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT NOT NULL);",
            "a NOT NULL column with no DEFAULT",
        ),
        (
            "CREATE TABLE notes (id TEXT PRIMARY KEY NOT NULL, autoincrement_note TEXT);",
            "has the word autoincrement in it",
        ),
    ];
    for (schema, said) in six {
        let words = refused(schema);
        println!("MEASURE {schema}\n  {words}");
        assert!(words.contains(said), "{schema}: {words}");
        assert!(words.contains("To fix it"), "{words}");
    }
}

/// **What the schema's authorizer refuses is said, with its fix**: an
/// AUTOINCREMENT, by the `sqlite_sequence` SQLite makes for it; any other
/// statement but a table or an index, a name the database keeps for itself
/// and a transaction among them, by what it asked; and SQL SQLite can't run,
/// in SQLite's words, as any SQL it can't run is said. Plant, watched failing: the check's authorizer saying
/// nothing as it refuses, and each is said as SQLite's "not authorized".
#[test]
fn what_the_schema_s_authorizer_refuses_is_said_with_its_fix() {
    let cards = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);\n";
    let refusals = [
        (
            "CREATE TABLE log (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, line TEXT);".into(),
            "says AUTOINCREMENT",
        ),
        (
            format!("{cards}CREATE TRIGGER gone AFTER DELETE ON cards BEGIN SELECT 1; END;"),
            "for CreateTrigger",
        ),
        (
            format!("{cards}CREATE VIEW titles AS SELECT title FROM cards;"),
            "for CreateView",
        ),
        (
            "CREATE VIRTUAL TABLE search USING fts5(body);".into(),
            "for CreateVtable",
        ),
        (
            "CREATE TABLE svrngs_sets (id TEXT PRIMARY KEY NOT NULL);".into(),
            "for CreateTable { table_name: \"svrngs_sets\" }",
        ),
        (format!("{cards}COMMIT;"), "for Transaction"),
    ];
    for (schema, said) in refusals {
        let words = refused(&schema);
        println!("MEASURE {schema}\n  {words}");
        assert!(words.contains(said), "{schema}: {words}");
        assert!(words.contains("To fix it"), "{words}");
    }
    let unrun = check("CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL,);", None).unwrap_err();
    assert!(
        unrun
            .to_string()
            .starts_with("SQLite couldn't run it: schema.sql: near \")\": syntax error"),
        "{unrun}"
    );
}

/// **An index is made**: SQLite asks the authorizer for a `REINDEX` of an
/// index as it makes it, so a rule that refused it would refuse every
/// `CREATE INDEX` (svrn-docs `magnets-spikes/README.md`, S5). Plant, watched
/// failing: `schema_rule` without its `Reindex` arm, and the index is
/// refused as "asks SQLite for Reindex".
#[test]
fn an_index_is_made() {
    let schema = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, col TEXT);\nCREATE INDEX cards_col ON cards (col);";
    assert_eq!(check(schema, None), Ok(vec![("cards".into(), false)]));
}

/// **A key compared as anything but BINARY is refused**: cr-sqlite keys a
/// row by its bytes and SQLite's NOCASE key folds 'Cat' and 'cat' into one
/// row, so copies diverge without a word (S5), in a rowid table and one
/// WITHOUT ROWID alike. Plant, watched failing: `limits` without its
/// collation query, and both are taken.
#[test]
fn a_key_compared_as_anything_but_binary_is_refused() {
    for schema in [
        "CREATE TABLE tags (name TEXT PRIMARY KEY NOT NULL COLLATE NOCASE, n INTEGER);",
        "CREATE TABLE tags (name TEXT NOT NULL COLLATE NOCASE, n INTEGER, PRIMARY KEY (name)) WITHOUT ROWID;",
    ] {
        let words = refused(schema);
        assert!(
            words.starts_with("The primary key of tags compares text as NOCASE"),
            "{words}"
        );
    }
    let plain = "CREATE TABLE tags (name TEXT PRIMARY KEY NOT NULL, label TEXT COLLATE NOCASE);";
    assert_eq!(
        check(plain, None),
        Ok(vec![("tags".into(), false)]),
        "a column outside the key may fold"
    );
}

/// **An owned table's person_key is TEXT, under BINARY, first in the key of
/// a STRICT table** (APP_STATE §4, "Who may write which row"), and a column
/// of that name anywhere else is refused, as it would promise a check it
/// doesn't get; a table that keeps all of it is named owned. Plant, watched
/// failing: `limits` with each of its three person_key arms and its
/// collation query off, and each of the five is taken, so none is refused
/// but by its own.
#[test]
fn an_owned_table_s_person_key_is_text_first_in_a_strict_key() {
    let votes = |cols: &str, key: &str, strict: &str| {
        format!("CREATE TABLE votes ({cols}, film TEXT NOT NULL, yes INTEGER, PRIMARY KEY ({key})){strict};")
    };
    let text = "person_key TEXT NOT NULL";
    let refusals = [
        (
            votes(text, "person_key, film", ""),
            "In votes, person_key is in a table that isn't STRICT",
        ),
        (
            votes(text, "film, person_key", " STRICT"),
            "In votes, person_key isn't first in its primary key",
        ),
        (
            votes("person_key TEXT", "film", " STRICT"),
            "In votes, person_key isn't first in its primary key",
        ),
        (
            votes("person_key BLOB NOT NULL", "person_key, film", " STRICT"),
            "In votes, person_key isn't TEXT",
        ),
        (
            votes("person_key TEXT NOT NULL COLLATE NOCASE", "person_key, film", " STRICT"),
            "compares text as NOCASE",
        ),
    ];
    let missed: Vec<_> = (refusals.iter())
        .filter(|(schema, said)| !check(schema, None).is_err_and(|r| r.to_string().contains(said)))
        .collect();
    assert_eq!(missed, Vec::<&(String, &str)>::new(), "each refused in its words");
    let schema =
        votes(text, "person_key, film", " STRICT") + "\nCREATE TABLE films (id TEXT PRIMARY KEY NOT NULL, title TEXT);";
    assert_eq!(
        check(&schema, None),
        Ok(vec![("films".into(), false), ("votes".into(), true)])
    );
}

/// **A schema that follows another only adds to it** (APP_STATE §4, "It
/// only adds"; invariant 14): a new table, and a column that can be NULL
/// with no DEFAULT, are taken; a column dropped, retyped, made NOT NULL,
/// moved into the key or moved in the table, as a new column before it
/// moves it, a table renamed, and a column added as NOT NULL, with a
/// DEFAULT, in the key or generated, are each refused, naming the column.
/// Plant, watched failing: `check` not calling `grows`, and the first is
/// taken.
#[test]
fn a_schema_that_follows_another_only_adds_to_it() {
    let was = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT);";
    let grown = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title text, note TEXT);\nCREATE TABLE lists (id TEXT PRIMARY KEY NOT NULL);";
    assert_eq!(
        check(grown, Some(was)),
        Ok(vec![("cards".into(), false), ("lists".into(), false)])
    );
    let refusals = [
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL);",
            "drops or changes cards.title",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title INTEGER);",
            "drops or changes cards.title",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT NOT NULL DEFAULT '');",
            "drops or changes cards.title",
        ),
        (
            "CREATE TABLE cards (id TEXT NOT NULL, title TEXT NOT NULL DEFAULT '', PRIMARY KEY (id, title));",
            "drops or changes cards.title",
        ),
        (
            "CREATE TABLE board (id TEXT PRIMARY KEY NOT NULL, title TEXT);",
            "drops or changes cards.id",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT NOT NULL DEFAULT '');",
            "adds cards.note",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, note TEXT DEFAULT 'none');",
            "adds cards.note",
        ),
        (
            "CREATE TABLE cards (id TEXT NOT NULL, title TEXT, n INTEGER NOT NULL, PRIMARY KEY (id, n));",
            "adds cards.n",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, note TEXT, title TEXT);",
            "drops or changes cards.title",
        ),
        (
            "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, n INTEGER GENERATED ALWAYS AS (1));",
            "adds cards.n",
        ),
    ];
    for (now, said) in refusals {
        let words = match check(now, Some(was)) {
            Err(Refused::Schema(w)) => w.to_string(),
            other => panic!("{now}\nnot refused: {other:?}"),
        };
        println!("MEASURE {now}\n  {words}");
        assert!(words.contains(said), "{now}: {words}");
    }
}

/// **One decider: a database is made by the check's rule**, so a schema the
/// check refuses opens no database, refused in the same words. Plant,
/// watched failing: `limits`' collation query off, and the NOCASE key
/// opens.
#[test]
fn a_database_is_refused_in_the_check_s_words() {
    let dir = tempfile::tempdir().unwrap();
    let states = States::new(dir.path(), SigningKey::from_bytes(&[1; 32]));
    let schema = "CREATE TABLE tags (name TEXT PRIMARY KEY NOT NULL COLLATE NOCASE, n INTEGER);";
    let opened = states.open("g", "tags", Some(schema)).err();
    assert_eq!(opened, check(schema, None).err());
    assert!(opened.is_some_and(|r| r.said() == (422, "schema")));
}
