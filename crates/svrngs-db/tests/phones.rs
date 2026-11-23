//! A phone's write to an app's database (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Phones"; MAGNETS_BUILD S2): prepared
//! by a computer, which keeps none of it, signed by the phone, applied as
//! a carried set. At the database itself, with no node or route. Each test
//! names the plant it was watched failing on.

use std::path::Path;
use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use svrngs_core::set::{At, site};
use svrngs_db::{Asked, Binds, Db, Device, Head, Seq, Set, Sig, SigningKey, States, Took};

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, col TEXT DEFAULT 'todo');\n\
    CREATE TABLE seats (row INTEGER NOT NULL, seat INTEGER NOT NULL, who TEXT, PRIMARY KEY (seat, row));\n";

/// `key`'s computer's databases under `dir`.
fn states(dir: &Path, key: u8) -> States {
    States::new(dir, SigningKey::from_bytes(&[key; 32]))
}

/// The board's database on the computer of `key`, in a directory of its own.
fn board(key: u8) -> (tempfile::TempDir, Arc<Db>) {
    let dir = tempfile::tempdir().unwrap();
    let db = states(dir.path(), key).open("g", "board", Some(SCHEMA)).unwrap();
    (dir, db)
}

/// The keys every copy here binds: every test key's.
fn bound() -> Binds {
    let keys = (0..=255).map(|n| svrngs_core::actor_of(&SigningKey::from_bytes(&[n; 32])));
    Binds::new(keys, 4 << 20)
}

/// Bo's phone, its key `[7; 32]`, and its database's nonce `[9; 16]`.
fn bo() -> (SigningKey, String) {
    let key = SigningKey::from_bytes(&[7; 32]);
    let device = svrngs_core::actor_of(&key);
    (key, device)
}

/// Bo's phone's site at the board.
fn bo_site() -> [u8; 16] {
    site("g", "board", &bo().1, &[9; 16])
}

/// The set at `seq` of the changes `answer`, a prepare's, names, signed by
/// Bo's phone, unchecked: the check is its signer's (`web/src/set.rs`).
fn signed(answer: &Value, seq: u64) -> Set {
    let text: Value = serde_json::from_str(answer["prepared"].as_str().unwrap()).unwrap();
    let bytes = |f: &str| STANDARD.decode(text[f].as_str().unwrap()).unwrap();
    let (changes, schema): (_, [u8; 32]) = (bytes("changes"), bytes("schema").try_into().unwrap());
    let ((key, device), hash) = (bo(), svrngs_core::set::changes(&changes));
    let at = At {
        group: "g",
        app: "board",
        device: &device,
        nonce: &[9; 16],
        seq,
        schema: &schema,
    };
    let sig = Sig::parse(&at.sign(&key, &hash)).unwrap();
    let (device, seq) = (Device::parse(&device).unwrap(), Seq::new(seq).unwrap());
    let head = Head {
        device,
        nonce: [9; 16],
        seq,
        schema,
        changes: hash,
        sig,
    };
    Set { head, changes }
}

/// `set` carried to `db` as its route takes one, `{carry}`, postcard's
/// bytes in base64: how it ended.
fn carry(db: &Db, set: &Set) -> Took {
    let carry = json!({ "carry": STANDARD.encode(postcard::to_allocvec(set).unwrap()) });
    let Ok(Asked::Carry(set)) = svrngs_db::asked(&carry) else {
        panic!("{carry} is no carry")
    };
    db.carry(set, &bound()).unwrap()
}

/// `card` inserted, prepared on `db` for Bo's phone.
fn prepared(db: &Db, card: &str) -> Value {
    db.prepare(&json!([{ "insert": "cards", "row": { "id": card, "title": card } }]))
        .unwrap()
}

/// Every row of `cards`, by id.
fn cards(db: &Db) -> Value {
    db.read("SELECT id, title FROM cards ORDER BY id", &[]).unwrap()["rows"].clone()
}

/// **A computer prepares a phone's write and keeps none of it** (§4,
/// "Phones", step 1): Ana's computer prepares an update of her card and a
/// new one, and answers them signed as hers, with the ops, each table's
/// key in order, the rows changed and each op's row as it would stand.
/// Her board is as it was, and her next write takes the version the
/// prepare didn't spend. A no-op is answered its count alone, as nothing
/// is to sign. Plant, watched failing: the prepare committing its
/// transaction (Ana's board has the eggs).
#[test]
fn a_computer_prepares_a_phone_s_write_and_keeps_none_of_it() {
    let (_dir, db) = board(1);
    let milk = json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]);
    assert_eq!(db.write(&milk).unwrap().1.map(|s| s.get()), Some(1));
    let ops = json!([
        { "update": "cards", "key": { "id": "c1" }, "set": { "title": "eggs" } },
        { "insert": "cards", "row": { "id": "c2", "title": "jam" } },
    ]);
    let answer = db.prepare(&ops).unwrap();
    let (text, by, sig) = (answer["prepared"].as_str().unwrap(), &answer["by"], &answer["sig"]);
    assert_eq!(by, &json!(svrngs_core::actor_of(&SigningKey::from_bytes(&[1; 32]))));
    let message = svrngs_core::set::prepared("g", "board", text);
    assert!(svrngs_core::verify_hex(
        by.as_str().unwrap(),
        &message,
        sig.as_str().unwrap()
    ));
    let prepared: Value = serde_json::from_str(text).unwrap();
    assert_eq!(prepared["ops"], ops);
    assert_eq!(prepared["keys"], json!({ "cards": ["id"], "seats": ["seat", "row"] }));
    assert_eq!(prepared["count"], 2);
    let stands = json!([{ "id": "c1", "title": "eggs", "col": null }, { "id": "c2", "title": "jam", "col": null }]);
    assert_eq!(prepared["stands"], stands);
    assert_eq!(prepared["schema"].as_str().unwrap().len(), 44, "a hash, in base64");
    assert!(prepared["changes"].as_str().is_some_and(|c| !c.is_empty()));
    assert_eq!(
        cards(&db),
        json!([{ "id": "c1", "title": "milk" }]),
        "nothing of it kept"
    );
    let tea = json!([{ "insert": "cards", "row": { "id": "c3", "title": "tea" } }]);
    assert_eq!(db.write(&tea).unwrap().1.map(|s| s.get()), Some(2), "no version spent");

    let same = json!([{ "update": "cards", "key": { "id": "c1" }, "set": { "title": "milk" } }]);
    assert_eq!(
        db.prepare(&same).unwrap(),
        json!({ "count": 1 }),
        "a no-op: nothing to sign"
    );
}

/// **A phone's set is applied as carried, never moving a mark past a gap**
/// (§4, "Phones", step 3; "What travels"): Bo's phone signs its second set
/// before its first reaches Ana's computer, which takes it, its card shown,
/// and counts it above Bo's site's mark, none yet; then the first, and the
/// mark is 2. The same bytes again, as a retry sends them, are held.
/// Plants, each watched failing: the carry judging the set by no keys the
/// record binds (Bo's set waits); a copy reading only its mark for a set it
/// holds (the retry taken again, as a twin of itself).
#[test]
fn a_phone_s_set_is_applied_as_carried_never_past_a_gap() {
    let (_dir, ana) = board(1);
    let (jam, milk) = (signed(&prepared(&ana, "jam"), 2), signed(&prepared(&ana, "milk"), 1));
    assert_eq!(carry(&ana, &jam), Took::Taken);
    assert_eq!(cards(&ana), json!([{ "id": "jam", "title": "jam" }]));
    let ask = ana.ask().unwrap();
    assert_eq!(ask.marks.get(&bo_site()), None, "no mark past the gap");
    assert!(
        ask.haves.contains(&(bo_site(), Seq::new(2).unwrap())),
        "counted above it"
    );
    assert_eq!(carry(&ana, &milk), Took::Taken);
    assert_eq!(ana.ask().unwrap().marks[&bo_site()].get(), 2, "the gap filled");
    assert_eq!(carry(&ana, &milk), Took::Held, "a retry's same bytes, held");
    assert_eq!(cards(&ana).as_array().unwrap().len(), 2);
}

/// **A switch of computers after apply, before the reply, ends with one
/// set at that seq and the write present** (APP_STATE §11, W3; §4, "A
/// retry: one seq, one set of bytes"): Bo's set reaches Ana's computer and
/// is applied, and its answer is lost, so his phone carries the same bytes
/// through Ben's. Once Ana's and Ben's copies meet, each holds his card,
/// counts his seq once, and names no twin; and the bytes carried again are
/// held on both. Plant, watched failing: a copy reading only its mark for
/// a set it holds, never its signature (sync.rs `take`: the retry taken
/// again, as a twin of itself).
#[test]
fn a_switch_of_computers_after_apply_ends_with_one_set_at_that_seq() {
    let ((_a, ana), (_b, ben)) = (board(1), board(2));
    let milk = signed(&prepared(&ana, "milk"), 1);
    assert_eq!(carry(&ana, &milk), Took::Taken, "applied; its answer lost");
    assert_eq!(
        carry(&ben, &milk),
        Took::Taken,
        "the same bytes, through Ben's computer"
    );
    for (from, to) in [(&ana, &ben), (&ben, &ana), (&ana, &ben)] {
        to.ingest(&from.missing(&to.ask().unwrap(), 4 << 20).unwrap(), &bound(), None)
            .unwrap();
    }
    for db in [&ana, &ben] {
        let ask = db.ask().unwrap();
        assert_eq!(
            cards(db),
            json!([{ "id": "milk", "title": "milk" }]),
            "the write present"
        );
        assert_eq!(ask.marks[&bo_site()].get(), 1);
        assert!(ask.twins.is_empty(), "one set at that seq: {:?}", ask.twins);
        assert_eq!(carry(db, &milk), Took::Held);
    }
}
