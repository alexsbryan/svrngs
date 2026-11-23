//! A phone's write to an app's database (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Phones"; MAGNETS_BUILD S2): prepared
//! by a computer, which keeps none of it, checked and signed by the phone's
//! signer, applied as a carried set. At the database itself, with no node
//! or route; the signer is the page's own, `web/src/set.rs`, compiled here.
//! Each test names the plant it was watched failing on.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use svrngs_core::pass::{Label, Pass};
use svrngs_core::set::site;
use svrngs_db::{Asked, Binds, Db, Seq, Set, SigningKey, States, Took};

use set::change::{self, Value as Sql};
// The phone's signer, the page's file compiled here, formatted by its own
// crate's rustfmt.toml.
#[path = "../../../web/src/set.rs"]
#[allow(dead_code)]
#[rustfmt::skip]
mod set;

const SCHEMA: &str = "CREATE TABLE cards (id TEXT PRIMARY KEY NOT NULL, title TEXT, col TEXT DEFAULT 'todo');\n\
    CREATE TABLE seats (row INTEGER NOT NULL, seat INTEGER NOT NULL, who TEXT, PRIMARY KEY (seat, row));\n\
    CREATE TABLE claims (person_key TEXT NOT NULL, item TEXT NOT NULL, PRIMARY KEY (person_key, item)) STRICT;\n";

/// The board's database on the computer of the key `[key; 32]`, in a
/// directory of its own.
fn board(key: u8) -> (tempfile::TempDir, Arc<Db>) {
    let dir = tempfile::tempdir().unwrap();
    let states = States::new(dir.path(), SigningKey::from_bytes(&[key; 32]));
    (dir, states.open("g", "board", Some(SCHEMA)).unwrap())
}

/// The keys every copy here binds: every test key's.
fn bound() -> Binds {
    let keys = (0..=255).map(|n| svrngs_core::actor_of(&SigningKey::from_bytes(&[n; 32])));
    Binds::new(keys.map(|k| (k.clone(), k, false)), 4 << 20)
}

/// Bo's phone: its key, `[7; 32]`, and its device.
fn bo() -> (SigningKey, String) {
    let key = SigningKey::from_bytes(&[7; 32]);
    let device = svrngs_core::actor_of(&key);
    (key, device)
}

/// Bo's open's approver, `[8; 32]`.
fn approver() -> SigningKey {
    SigningKey::from_bytes(&[8; 32])
}

/// The pass Bo's phone signed for his open of Board in `g`, naming its
/// approver, lapsing at `until`, as hand.js signs one (web/src/lib.rs
/// `pass`).
fn pass_until(until: i64) -> Pass {
    let (app, approver) = (Label::carried("board", "g"), svrngs_core::actor_of(&approver()));
    Pass::approvable(&bo().0, &"h".repeat(64), (app, vec![]), until, Some(approver))
}

/// Bo's phone's signer's store: its id, and the seq it last signed at each
/// computer, its home there, as web/sign.js keeps them in one record.
#[derive(Default)]
struct Store {
    id: [u8; 16],
    counts: BTreeMap<String, u64>,
}

thread_local! {
    /// Bo's phone's signer's store in this test, its id `[7; 16]` until a
    /// test clears it.
    static STORE: RefCell<Store> = RefCell::new(Store { id: [7; 16], ..Store::default() });
}

/// What Bo's phone's signer is asked for `ops`, as `answer`, a prepare's,
/// gave them back, under his open's pass, stamped at 1,000, at the place it
/// picks: its store, and the seq after the last it signed at that computer.
fn asked(ops: &Value, answer: &Value) -> set::Asked {
    let text = |f: &str| answer[f].as_str().unwrap().to_owned();
    let (group, app, ops) = ("g".into(), "board".into(), ops.clone());
    let (store, seq) = STORE.with_borrow(|s| (s.id, s.counts.get(&text("by")).copied().unwrap_or(0) + 1));
    set::Asked {
        group,
        app,
        ops,
        prepared: text("prepared"),
        by: text("by"),
        sig: text("sig"),
        pass: pass_until(2_000_000_000).encode(),
        ts: 1_000,
        store,
        seq,
    }
}

/// The set Bo's phone's signer makes for `asked`, framed by Board's own
/// page, signed by its open's approver, postcard's bytes in base64, its seq
/// then the last it signed at that computer; or its words for why not.
fn signed(asked: &set::Asked) -> Result<String, String> {
    let label = Label::carried("board", "g").to_string();
    let (message, set, _) = set::signed(&label, asked)?.ok_or("no set: it changes nothing")?;
    STORE.with_borrow_mut(|s| s.counts.insert(asked.by.clone(), asked.seq));
    Ok(set::sealed(&set, &svrngs_core::sign_message(&approver(), &message)).unwrap())
}

/// `ops`, prepared on `db` for Bo's phone and signed there.
fn prepared(db: &Db, ops: &Value) -> String {
    signed(&asked(ops, &db.prepare(ops, (&bo().1, &bound())).unwrap())).unwrap()
}

/// `card` inserted, as [`prepared`].
fn card(db: &Db, card: &str) -> String {
    prepared(
        db,
        &json!([{ "insert": "cards", "row": { "id": card, "title": card } }]),
    )
}

/// `set` carried to `db` as its route takes one, `{carry}`: how it ended.
fn carry(db: &Db, set: &str) -> Took {
    let Ok(Asked::Carry(set)) = svrngs_db::asked(&json!({ "carry": set })) else {
        panic!("{set} is no carry")
    };
    db.carry(set, &bound()).unwrap()
}

/// The site of the carried `set`.
fn site_of(set: &str) -> [u8; 16] {
    let set: Set = postcard::from_bytes(&STANDARD.decode(set).unwrap()).unwrap();
    site("g", "board", set.head.device.as_str(), &set.head.nonce)
}

/// Every row of `cards`, by id.
fn cards(db: &Db) -> Value {
    db.read("SELECT id, title FROM cards ORDER BY id", &[]).unwrap()["rows"].clone()
}

/// **A computer prepares a phone's write and keeps none of it** (§4,
/// "Phones", step 1): Ana's computer prepares an update of her card and a
/// new one, and answers them signed as hers, with the ops, each table's
/// key in order, the rows changed, and each op's row as it stood before,
/// and no place, which Bo's phone's signer picks. Her board is as it was,
/// and her next write takes the version the prepare didn't spend. Plant,
/// watched failing: the prepare committing its transaction (Ana's board had
/// the eggs and the jam).
#[test]
fn a_computer_prepares_a_phone_s_write_and_keeps_none_of_it() {
    let (_dir, db) = board(1);
    let milk = json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]);
    assert_eq!(db.write(&milk, &bound()).unwrap().1.map(|s| s.get()), Some(1));
    let ops = json!([
        { "update": "cards", "key": { "id": "c1" }, "set": { "title": "eggs" } },
        { "insert": "cards", "row": { "id": "c2", "title": "jam" } },
    ]);
    let answer = db.prepare(&ops, (&bo().1, &bound())).unwrap();
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
    assert_eq!(
        prepared["keys"],
        json!({ "cards": ["id"], "claims": ["person_key", "item"], "seats": ["seat", "row"] })
    );
    assert_eq!(prepared["count"], 2);
    assert_eq!(
        prepared["stands"],
        json!([{ "id": "c1", "title": "milk", "col": null }, null]),
        "as they stood"
    );
    assert!(
        prepared.get("seq").is_none() && prepared.get("epoch").is_none(),
        "no place: the phone's signer picks it"
    );
    assert_eq!(prepared["schema"].as_str().unwrap().len(), 44, "a hash, in base64");
    assert_eq!(
        cards(&db),
        json!([{ "id": "c1", "title": "milk" }]),
        "nothing of it kept"
    );
    let tea = json!([{ "insert": "cards", "row": { "id": "c3", "title": "tea" } }]);
    assert_eq!(
        db.write(&tea, &bound()).unwrap().1.map(|s| s.get()),
        Some(2),
        "no version spent"
    );
}

/// **Every prepare is signed and checked, one that changes nothing too**
/// (svrn-docs `app-state-diligence/invariants-2026-11-22/`, change 4): Ana's
/// computer signs its prepare of a no-op, an update of her card to the title
/// it has, with no change row, and Bo's phone checks its ops
/// against the card as it stood and signs no set. A computer whose rows for
/// an insert came back empty signs that same shape, and the phone refuses
/// it in words, where before it was answered its count unsigned, no signer
/// asked, and the write was lost. Plant, watched failing: the signer taking
/// a prepare with no change row as no set before it checks every effect
/// (the lost insert said to change nothing).
#[test]
fn every_prepare_is_signed_and_checked_one_that_changes_nothing_too() {
    let (_dir, ana) = board(1);
    let milk = json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]);
    ana.write(&milk, &bound()).unwrap();
    let same = json!([{ "update": "cards", "key": { "id": "c1" }, "set": { "title": "milk" } }]);
    let answer = ana.prepare(&same, (&bo().1, &bound())).unwrap();
    let label = Label::carried("board", "g").to_string();
    let text: Value = serde_json::from_str(answer["prepared"].as_str().unwrap()).unwrap();
    let rows: Vec<change::Change> =
        postcard::from_bytes(&STANDARD.decode(text["changes"].as_str().unwrap()).unwrap()).unwrap();
    assert!(rows.is_empty(), "no change row");
    assert_eq!(text["count"], json!(1), "its count");
    assert_eq!(
        set::signed(&label, &asked(&same, &answer)),
        Ok(None),
        "checked, and no set"
    );

    // An insert whose rows came back empty, signed as Ana's computer signs.
    let jam = json!([{ "insert": "cards", "row": { "id": "c2", "title": "jam" } }]);
    let mut lost = text.clone();
    lost["ops"] = jam.clone();
    lost["stands"] = json!([null]);
    let lost = lost.to_string();
    let sig = svrngs_core::sign_message(
        &SigningKey::from_bytes(&[1; 32]),
        &svrngs_core::set::prepared("g", "board", &lost),
    );
    let refused = set::signed(
        &label,
        &set::Asked {
            prepared: lost,
            sig,
            ..asked(&jam, &answer)
        },
    );
    let words = "This phone didn’t sign board’s write, since the computer left out a change it asked for. Try again, or open it through another computer.";
    assert_eq!(refused, Err(words.into()), "every effect missing");
}

/// **A phone signs only what its page asked** (§4, "Phones", step 2; W3):
/// a mixed batch, an update, an insert, a delete and a no-op, prepared on
/// Ana's computer, is signed. Each lie a computer could tell is refused
/// whole, in words that say why: one effect deliberately missing, the
/// insert's; a value the page didn't ask; other ops than it sent; a prepare
/// no computer signed; and a page of another app asking. Plants, each
/// watched failing: a no-op read with no regard to its row (the missing
/// insert signed); a column's value left unread (the ham signed).
#[test]
fn a_phone_signs_only_what_its_page_asked() {
    let (_dir, ana) = board(1);
    let both = json!([
        { "insert": "cards", "row": { "id": "c1", "title": "milk" } },
        { "insert": "cards", "row": { "id": "c3", "title": "tea" } },
    ]);
    ana.write(&both, &bound()).unwrap();
    let ops = json!([
        { "update": "cards", "key": { "id": "c1" }, "set": { "title": "eggs" } },
        { "insert": "cards", "row": { "id": "c2", "title": "jam" } },
        { "delete": "cards", "key": { "id": "c3" } },
        { "update": "cards", "key": { "id": "c1" }, "set": { "col": null } },
    ]);
    let answer = ana.prepare(&ops, (&bo().1, &bound())).unwrap();
    let honest = asked(&ops, &answer);
    let set = signed(&honest).expect("the batch as its page asked it");
    assert_eq!(carry(&ana, &set), Took::Taken);
    assert_eq!(
        cards(&ana),
        json!([{ "id": "c1", "title": "eggs" }, { "id": "c2", "title": "jam" }])
    );

    // A computer's lie, signed as Ana's computer signs a prepare: its text
    // with its changes rewritten by `lie`.
    let lied = |lie: &dyn Fn(&mut Vec<change::Change>)| {
        let mut text: Value = serde_json::from_str(&honest.prepared).unwrap();
        let bytes = STANDARD.decode(text["changes"].as_str().unwrap()).unwrap();
        let mut rows: Vec<change::Change> = postcard::from_bytes(&bytes).unwrap();
        lie(&mut rows);
        text["changes"] = STANDARD.encode(postcard::to_allocvec(&rows).unwrap()).into();
        let text = text.to_string();
        let sig = svrngs_core::sign_message(
            &SigningKey::from_bytes(&[1; 32]),
            &svrngs_core::set::prepared("g", "board", &text),
        );
        signed(&set::Asked {
            prepared: text,
            sig,
            ..asked(&ops, &answer)
        })
    };
    let jam = |c: &change::Change| c.val == Sql::Text("jam".into());
    let missing = lied(&|rows| {
        let pk = rows.iter().find(|c| jam(c)).unwrap().pk.clone();
        rows.retain(|c| c.pk != pk);
    });
    let words = "This phone didn’t sign board’s write, since the computer left out a change it asked for. Try again, or open it through another computer.";
    assert_eq!(missing, Err(words.into()), "one effect deliberately missing");
    let ham = lied(&|rows| {
        rows.iter_mut()
            .filter(|c| jam(c))
            .for_each(|c| c.val = Sql::Text("ham".into()))
    });
    assert!(
        ham.unwrap_err().contains("a change it didn’t ask for"),
        "a value the page didn't ask"
    );
    let other = signed(&set::Asked {
        ops: json!([ops[0]]),
        ..asked(&ops, &answer)
    });
    assert!(other.unwrap_err().contains("other ops than it asked"));
    let unsigned = signed(&set::Asked {
        by: bo().1,
        ..asked(&ops, &answer)
    });
    assert!(unsigned.unwrap_err().contains("isn’t a computer’s prepare"));
    let elsewhere = set::signed(&Label::carried("magnets", "g").to_string(), &honest);
    assert!(
        elsewhere.unwrap_err().contains("isn’t a computer’s prepare"),
        "another app's page"
    );
}

/// **A phone reads a write-form value as the computer that prepared it
/// does** (svrngs-db form.rs `value`): a BLOB is `{base64}`, its one key.
/// Ana's computer refuses `{base64, also}` before any prepare, and Bo's
/// phone refuses a prepare of it her computer signed, as ops it can't read,
/// though it signs her prepare of the BLOB written as the form writes one.
/// Plant, watched failing: the phone's own reader, which took any object
/// with a `base64` key (the prepare of `{base64, also}` signed).
#[test]
fn a_phone_reads_a_value_as_its_computer_does() {
    let (_dir, ana) = board(1);
    let blob = |v: Value| json!([{ "insert": "cards", "row": { "id": "b", "title": v } }]);
    let one = blob(json!({ "base64": "AQ==" }));
    let two = blob(json!({ "base64": "AQ==", "also": 1 }));
    let refused = ana.prepare(&two, (&bo().1, &bound()));
    assert!(matches!(refused, Err(svrngs_db::Refused::Form(_))), "{refused:?}");
    let answer = ana.prepare(&one, (&bo().1, &bound())).unwrap();
    signed(&asked(&one, &answer)).expect("a BLOB as the write form writes one");
    // Her prepare of the BLOB, its ops `two`, signed as her computer signs one.
    let mut text: Value = serde_json::from_str(answer["prepared"].as_str().unwrap()).unwrap();
    text["ops"] = two.clone();
    let text = text.to_string();
    let ana_s = SigningKey::from_bytes(&[1; 32]);
    let sig = svrngs_core::sign_message(&ana_s, &svrngs_core::set::prepared("g", "board", &text));
    let read = signed(&set::Asked {
        ops: two.clone(),
        prepared: text,
        sig,
        ..asked(&one, &answer)
    });
    assert!(
        read.unwrap_err().contains("other ops than it asked"),
        "a value no computer would write"
    );
}

/// **A phone's set is applied as carried, never moving a mark past a gap**
/// (§4, "Phones", step 3; "What travels"): Bo's two cards, each prepared
/// and applied at Ana's computer, his home there, at seqs 1 and 2. The
/// second reaches Ben's computer first, which takes it, its card shown, and
/// counts it above Bo's site's mark, none yet; then the first, and the mark
/// is 2. The same bytes again, as a retry sends them, are held. Plants,
/// each watched failing: the carry judging the set by no keys the record
/// binds (Bo's set waits); a copy reading only its mark for a set it
/// holds, never its signature (the retry taken again, as a twin of itself).
#[test]
fn a_phone_s_set_is_applied_as_carried_never_past_a_gap() {
    let ((_a, ana), (_b, ben)) = (board(1), board(2));
    let milk = card(&ana, "milk");
    assert_eq!(carry(&ana, &milk), Took::Taken);
    let jam = card(&ana, "jam");
    assert_eq!(carry(&ben, &jam), Took::Taken);
    assert_eq!(cards(&ben), json!([{ "id": "jam", "title": "jam" }]));
    let (ask, bo) = (ben.ask().unwrap(), site_of(&milk));
    assert_eq!(ask.marks.get(&bo), None, "no mark past the gap");
    assert!(ask.haves.contains(&(bo, Seq::new(2).unwrap())), "counted above it");
    assert_eq!(carry(&ben, &milk), Took::Taken);
    assert_eq!(ben.ask().unwrap().marks[&bo].get(), 2, "the gap filled");
    assert_eq!(carry(&ben, &milk), Took::Held, "a retry's same bytes, held");
    assert_eq!(cards(&ben).as_array().unwrap().len(), 2);
}

/// **A switch of computers after apply, before the reply, ends with one
/// set at that seq and the write present** (APP_STATE §11, W3; §4, "A
/// retry: one seq, one set of bytes"): Bo's set reaches Ana's computer and
/// is applied, and its answer is lost, so his phone carries the same bytes
/// through Ben's. Once Ana's and Ben's copies meet, each holds his card,
/// counts his seq once, and names no twin; and the bytes carried again are
/// held on both. This holds for an honest phone's signer: a device acting
/// in bad faith can still show different copies different writes of its
/// own. Plant, watched failing: a copy reading only its mark for a set it
/// holds (the retry taken again, as a twin of itself).
#[test]
fn a_switch_of_computers_after_apply_ends_with_one_set_at_that_seq() {
    let ((_a, ana), (_b, ben)) = (board(1), board(2));
    let milk = card(&ana, "milk");
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
        assert_eq!(ask.marks[&site_of(&milk)].get(), 1);
        assert!(ask.twins.is_empty(), "one set at that seq: {:?}", ask.twins);
        assert_eq!(carry(db, &milk), Took::Held);
    }
}

/// The board on the computer of `[key; 32]` made synced, as the computer
/// that carries its schema makes it, so its copy is an established one.
fn root(key: u8) -> (tempfile::TempDir, Arc<Db>) {
    let dir = tempfile::tempdir().unwrap();
    let states = States::new(dir.path(), SigningKey::from_bytes(&[key; 32]));
    (dir, states.carry("g", "board", SCHEMA).unwrap())
}

/// The device of the key `[key; 32]`.
fn device(key: u8) -> svrngs_db::Device {
    svrngs_db::Device::parse(&svrngs_core::actor_of(&SigningKey::from_bytes(&[key; 32]))).unwrap()
}

/// `to` takes `from`'s full call, as the round has it from `from`'s
/// computer.
fn call(from: &Db, (to, by): (&Db, u8)) {
    let call = from.missing(&to.ask().unwrap(), 4 << 20).unwrap();
    to.ingest(&call, &bound(), Some(&device(by))).unwrap();
}

/// **A phone's site is vouched for by its home** (svrn-docs
/// `app-state-diligence/sync-simplify-2026-11-22/`, change 4): Bo's phone
/// writes through Ana's computer, its home there, twice to one card's
/// title, and Ana's copy drops the first set, outranked, before any round.
/// Ben's established copy, called by Ana's, is sent the second alone, at
/// seq 2, and takes Ana's word for the site's mark up to it: the gap
/// closes on the home's call, nothing kept above it. A set naming a home
/// its nonce isn't derived from is not its device's. Plant, watched
/// failing: the word trusted from the site's device alone, as before
/// change 4 (Ben's mark for Bo's site none, seq 2 kept above it).
#[test]
fn a_phone_s_site_is_vouched_for_by_its_home() {
    let ((_a, ana), (_b, ben)) = (root(1), root(2));
    ana.write(
        &json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]),
        &bound(),
    )
    .unwrap();
    let mut nonce = [0; 16];
    for title in ["eggs", "jam"] {
        let set = prepared(
            &ana,
            &json!([{ "update": "cards", "key": { "id": "c1" }, "set": { "title": title } }]),
        );
        nonce = postcard::from_bytes::<Set>(&STANDARD.decode(&set).unwrap())
            .unwrap()
            .head
            .nonce;
        assert_eq!(carry(&ana, &set), Took::Taken);
    }
    let jam = json!([{ "id": "c1", "title": "jam" }]);
    assert_eq!(cards(&ana), jam);
    call(&ana, (&ben, 1));
    let (ask, bo) = (ben.ask().unwrap(), site("g", "board", &bo().1, &nonce));
    assert_eq!(cards(&ben), jam, "the second set alone, the first outranked");
    assert_eq!(
        ask.marks.get(&bo).map(|m| m.get()),
        Some(2),
        "the gap closed on the home's call"
    );
    assert!(ask.haves.is_empty(), "nothing kept above a gap: {:?}", ask.haves);

    let set = prepared(
        &ana,
        &json!([{ "update": "cards", "key": { "id": "c1" }, "set": { "title": "tea" } }]),
    );
    let mut moved: Set = postcard::from_bytes(&STANDARD.decode(&set).unwrap()).unwrap();
    moved.phone.as_mut().unwrap().home = device(2);
    let moved = STANDARD.encode(postcard::to_allocvec(&moved).unwrap());
    assert_eq!(
        carry(&ben, &moved),
        Took::NotItsDevice,
        "a home its nonce isn't derived from"
    );
}

/// **A phone's signer picks each place, so none is left unfilled and none
/// signed twice** (svrn-docs `app-state-diligence/invariants-2026-11-22/`,
/// change 5): Ana's computer, Bo's home, prepares three cards and hands no
/// place. His signer signs the third, then the first, at seqs 1 and 2, and
/// both are taken, where a place handed at prepare told the earlier one
/// "write it again". The second's page closes before its signer sees it,
/// and leaves nothing: Ana's copy counts Bo's site to 2, nothing above it,
/// where a handed place left a have there for good. His signer's store
/// cleared, its count with it, his next card is signed at seq 1 of a site
/// of its own, and taken, no twin. Plant, watched failing: wire.rs `nonce`
/// derived from the home alone (the cleared store's first set at Bo's old
/// site, a twin of the third card's).
#[test]
fn a_phone_s_signer_picks_each_place_so_none_is_left_unfilled_or_signed_twice() {
    let (_dir, ana) = board(1);
    let card = |card: &str| json!([{ "insert": "cards", "row": { "id": card, "title": card } }]);
    let [milk, _closed, jam] =
        ["milk", "tea", "jam"].map(|c| (card(c), ana.prepare(&card(c), (&bo().1, &bound())).unwrap()));
    let jam = signed(&asked(&jam.0, &jam.1)).unwrap();
    let milk = signed(&asked(&milk.0, &milk.1)).unwrap();
    let seq = |set: &str| {
        postcard::from_bytes::<Set>(&STANDARD.decode(set).unwrap())
            .unwrap()
            .head
            .seq
            .get()
    };
    assert_eq!((seq(&jam), seq(&milk)), (1, 2), "in the order signed");
    assert_eq!(carry(&ana, &milk), Took::Taken);
    assert_eq!(carry(&ana, &jam), Took::Taken);
    let (ask, site) = (ana.ask().unwrap(), site_of(&jam));
    assert_eq!(ask.marks.get(&site).map(|m| m.get()), Some(2));
    assert!(ask.haves.is_empty(), "no place left unfilled: {:?}", ask.haves);

    STORE.with_borrow_mut(|s| {
        *s = Store {
            id: [9; 16],
            ..Store::default()
        }
    });
    let pie = card("pie");
    let pie = signed(&asked(&pie, &ana.prepare(&pie, (&bo().1, &bound())).unwrap())).unwrap();
    assert_eq!(seq(&pie), 1, "the cleared store's first");
    assert_ne!(site_of(&pie), site, "a site of its own");
    assert_eq!(carry(&ana, &pie), Took::Taken, "no twin");
    assert_eq!(cards(&ana).as_array().unwrap().len(), 3);
}

/// **A phone's prepare refuses an owned row the record doesn't give its
/// person, as a write does** (APP_STATE §4, "Who may write which row"):
/// Bo's phone's claim on Ana's row is refused at Ana's computer before
/// anything is signed, naming the table, and his claim on his own is
/// prepared, signed and taken, as every copy judges it again at apply.
/// Plant, watched failing: the prepare judging no owned row (Bo's claim on
/// Ana's row prepared).
#[test]
fn a_phone_s_prepare_refuses_another_s_owned_row() {
    let (_dir, ana) = board(1);
    let claim = |person: &str| json!([{ "insert": "claims", "row": { "person_key": person, "item": "ice" } }]);
    let anas = svrngs_core::actor_of(&SigningKey::from_bytes(&[1; 32]));
    let refused = ana.prepare(&claim(&anas), (&bo().1, &bound()));
    assert_eq!(refused.unwrap_err(), svrngs_db::Refused::NotYours("claims".into()));
    let set = prepared(&ana, &claim(&bo().1));
    assert_eq!(carry(&ana, &set), Took::Taken, "his own row, taken at apply");
}

/// **A phone's set holds only under its open's pass** (the operator's B′,
/// 2026-11-22; svrn-docs HOLDER_APPROVAL §2, rule 4): its pass must be
/// signed by the set's device, open this app's carried label in this group,
/// name the approver that signs the set, and lapse no sooner than the set's
/// stamp. Bo's signer refuses a pass his phone didn't sign, a pass of
/// another app and a stamp past its lapse, each in words for him; a set
/// signed by his phone's own key, not the approver its pass names, is not
/// his device's at Ana's copy; the set as his open's approver signs it is
/// taken. Plants, each watched failing in wire.rs `Phoned::signer`: the
/// pass's own signature unread (the forged pass signed); its app unread
/// (another app's signed); its lapse unread (the late stamp signed); the
/// set's signer read as its device (the device-signed set taken).
#[test]
fn a_phone_s_set_holds_only_under_its_open_s_pass() {
    let (_dir, ana) = board(1);
    let ops = json!([{ "insert": "cards", "row": { "id": "c1", "title": "milk" } }]);
    let answer = ana.prepare(&ops, (&bo().1, &bound())).unwrap();
    let refused = |asked: set::Asked| signed(&asked).unwrap_err();
    let holds = "This phone didn’t sign board’s write, since its open’s pass doesn’t hold. Try again, or open it through another computer.";
    let mut forged = pass_until(2_000_000_000);
    forged.person = svrngs_core::actor_of(&SigningKey::from_bytes(&[9; 32]));
    let pass = forged.encode();
    assert_eq!(
        refused(set::Asked {
            pass,
            ..asked(&ops, &answer)
        }),
        holds,
        "a pass its person didn't sign"
    );
    let (magnets, approver) = (Label::carried("magnets", "g"), svrngs_core::actor_of(&approver()));
    let other = Pass::approvable(
        &bo().0,
        &"h".repeat(64),
        (magnets, vec![]),
        2_000_000_000,
        Some(approver),
    );
    let pass = other.encode();
    assert_eq!(
        refused(set::Asked {
            pass,
            ..asked(&ops, &answer)
        }),
        holds,
        "another app's pass"
    );
    let late = set::Asked {
        ts: 2_000_000_001,
        ..asked(&ops, &answer)
    };
    assert_eq!(refused(late), holds, "a stamp past its lapse");

    let label = Label::carried("board", "g").to_string();
    let (message, set, _) = set::signed(&label, &asked(&ops, &answer)).unwrap().unwrap();
    let by_device = set::sealed(&set, &svrngs_core::sign_message(&bo().0, &message)).unwrap();
    assert_eq!(
        carry(&ana, &by_device),
        Took::NotItsDevice,
        "signed by his device, not its approver"
    );
    assert_eq!(carry(&ana, &signed(&asked(&ops, &answer)).unwrap()), Took::Taken);
}

/// **A phone signs a write as its ops leave the rows, in their order**
/// (W1's phones found the phone refusing these, one honest write in five):
/// an update of a row that isn't there, which changes nothing; an insert
/// and an update of one new row; a delete and an insert of one row; and
/// two updates of one cell. Each is prepared at Ana's computer, signed, and
/// taken, the cards as the ops leave them. Plants, each watched failing in
/// web/src/set.rs: an update read as making the row it names (`net`; the
/// write refused as a left-out change); the effects left unchecked (the
/// mixed batch's missing insert signed, in
/// `a_phone_signs_only_what_its_page_asked`).
#[test]
fn a_phone_signs_a_write_as_its_ops_leave_the_rows() {
    let (_dir, ana) = board(1);
    ana.write(
        &json!([{ "insert": "cards", "row": { "id": "c3", "title": "tea" } }]),
        &bound(),
    )
    .unwrap();
    let ops = json!([
        { "update": "cards", "key": { "id": "c9" }, "set": { "title": "nobody" } },
        { "insert": "cards", "row": { "id": "c1", "title": "milk" } },
        { "update": "cards", "key": { "id": "c1" }, "set": { "title": "eggs" } },
        { "delete": "cards", "key": { "id": "c3" } },
        { "insert": "cards", "row": { "id": "c3", "title": "jam" } },
        { "update": "cards", "key": { "id": "c3" }, "set": { "col": "a" } },
        { "update": "cards", "key": { "id": "c3" }, "set": { "col": "b" } },
    ]);
    let set = prepared(&ana, &ops);
    assert_eq!(carry(&ana, &set), Took::Taken);
    let all = ana.read("SELECT id, title, col FROM cards ORDER BY id", &[]).unwrap()["rows"].clone();
    assert_eq!(
        all,
        json!([{ "id": "c1", "title": "eggs", "col": null }, { "id": "c3", "title": "jam", "col": "b" }])
    );
}
