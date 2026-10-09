// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn lender(seed: u8) -> [u8; 32] {
    key(seed).verifying_key().to_bytes()
}

/// A label is one DNS label — 62 lowercase letters and digits — that gives
/// its lender's key back whole; another name, or another lender, is
/// another label; and anything that is not one is refused.
#[test]
fn a_label_is_one_dns_label_that_names_its_lender_and_lend() {
    let label = Label::of(&lender(1), "jellyfin");
    assert_eq!(label.as_str().len(), 62);
    assert!(label
        .as_str()
        .bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    assert_eq!(label.lender(), lender(1));
    assert_eq!(Label::parse(label.as_str()), Some(label.clone()));
    assert_ne!(Label::of(&lender(1), "films"), label);
    assert_ne!(Label::of(&lender(2), "jellyfin"), label);
    // A 64-byte name still makes a 62-letter label.
    assert_eq!(Label::of(&lender(1), &"n".repeat(64)).as_str().len(), 62);
    for not in [
        "",
        "jellyfin",
        &label.as_str()[..61],
        &label.as_str().to_uppercase(),
        &alloc::format!("{}a", label.as_str()),
    ] {
        assert_eq!(Label::parse(not), None, "{not}");
    }
    // The 52nd letter holds the key's last bits: a set pad bit is not a key.
    let mut padded = label.as_str().as_bytes().to_vec();
    padded[51] = b'9';
    assert_eq!(Label::parse(core::str::from_utf8(&padded).unwrap()), None);
}

/// No label can spell a word DESIGN §1 keeps from people: each holds a
/// letter the label's alphabet leaves out. The failing input is RFC 4648's
/// alphabet, which holds every letter.
#[test]
fn no_label_spells_a_word_people_are_not_shown() {
    for word in crate::link::KEPT_WORDS {
        assert!(
            word.bytes().any(|c| !BASE32.contains(&c)),
            "{word} can be spelt in a label"
        );
    }
}

/// A pass opens what it names, to the key it names, until it lapses, and
/// nothing else; one whose fields were changed after signing opens
/// nothing. The failing inputs are each fault's plant: a pass taken for any
/// lend, a holder not checked, a lapse not checked, a signature not
/// checked.
#[test]
fn a_pass_opens_only_the_lends_it_names_to_its_holder_until_it_lapses() {
    let (alex, holder) = (key(1), actor_of(&key(9)));
    let (sams, bos) = (Label::of(&lender(2), "app"), Label::of(&lender(3), "app"));
    let pass = Pass::sign(&alex, &holder, (sams.clone(), alloc::vec![]), 1_000);
    assert_eq!(pass.check(&holder, &sams, 999), Ok(()));
    assert_eq!(pass.check(&holder, &bos, 999), Err(PassFault::NotNamed));
    assert_eq!(
        pass.check(&actor_of(&key(8)), &sams, 999),
        Err(PassFault::NotTheHolder)
    );
    assert_eq!(pass.check(&holder, &sams, 1_000), Err(PassFault::Lapsed));
    let mut widened = pass.clone();
    widened.reads.push(bos.clone());
    assert_eq!(widened.check(&holder, &bos, 999), Err(PassFault::Unsigned));
    let mut longer = pass.clone();
    longer.until = 5_000;
    assert_eq!(longer.check(&holder, &sams, 999), Err(PassFault::Unsigned));
    let mut other = pass.clone();
    other.person = actor_of(&key(4));
    assert_eq!(other.check(&holder, &sams, 999), Err(PassFault::Unsigned));
}

/// A pass round-trips through the header's form, and what is not one
/// decodes to nothing.
#[test]
fn a_pass_round_trips_through_its_header_form() {
    let pass = Pass::sign(
        &key(1),
        &actor_of(&key(9)),
        (Label::of(&lender(2), "app"), alloc::vec![]),
        1_000,
    );
    let encoded = pass.encode();
    assert!(encoded
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'));
    assert_eq!(Pass::decode(&encoded), Some(pass));
    assert_eq!(Pass::decode("not a pass"), None);
    assert_eq!(Pass::decode(""), None);
}

/// A pass names its app apart from the lends it reads (sup-1): it opens
/// its app and each read, and nothing else; moving a read into the app's
/// place, or the app among the reads, opens nothing; and it signs the bytes
/// a pass naming `[app, ...reads]` as its lends signed before, pinned as
/// that pass's signature at `8cb33fc`, so a page that signed one then is
/// read as naming its first lend the app. A pass of that shape, `lends`
/// and no `app`, no longer decodes, so the door refuses it `pass` and the
/// page signs another. The failing input: `check` opening the first read
/// alone.
#[test]
fn a_pass_names_its_app_and_its_reads_in_the_bytes_signed_before() {
    let holder = actor_of(&key(9));
    let (app, taste) = (Label::of(&lender(2), "app"), Label::of(&lender(3), "taste"));
    let pass = Pass::sign(
        &key(1),
        &holder,
        (app.clone(), alloc::vec![taste.clone()]),
        1_000,
    );
    assert_eq!(pass.sig, "1c4f8534215d277e1d0c49432768ba8f02273c738ff9aa016543dd973108237d959217dbb6c7a0713736c4320361e0ce9f4701f6b18e97a24a33eef31adaa708");
    assert_eq!(pass.check(&holder, &app, 999), Ok(()));
    assert_eq!(pass.check(&holder, &taste, 999), Ok(()));
    let other = Label::of(&lender(4), "app");
    assert_eq!(pass.check(&holder, &other, 999), Err(PassFault::NotNamed));
    let mut swapped = pass.clone();
    (swapped.app, swapped.reads) = (taste.clone(), alloc::vec![app.clone()]);
    assert_eq!(
        swapped.check(&holder, &taste, 999),
        Err(PassFault::Unsigned)
    );
    let mut before = serde_json::to_value(&pass).unwrap();
    let lends = serde_json::json!([app.as_str(), taste.as_str()]);
    let fields = before.as_object_mut().unwrap();
    fields.remove("app");
    fields.remove("reads");
    fields.insert("lends".into(), lends);
    let json = serde_json::to_vec(&before).unwrap();
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json);
    assert_eq!(Pass::decode(&raw), None);
}
