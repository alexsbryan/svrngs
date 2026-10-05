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
        .all(|c| c.is_ascii_lowercase() || (b'2'..=b'7').contains(&c)));
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
    padded[51] = b'7';
    assert_eq!(Label::parse(core::str::from_utf8(&padded).unwrap()), None);
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
    let pass = Pass::sign(&alex, &holder, alloc::vec![sams.clone()], 1_000);
    assert_eq!(pass.check(&holder, &sams, 999), Ok(()));
    assert_eq!(pass.check(&holder, &bos, 999), Err(PassFault::NotNamed));
    assert_eq!(
        pass.check(&actor_of(&key(8)), &sams, 999),
        Err(PassFault::NotTheHolder)
    );
    assert_eq!(pass.check(&holder, &sams, 1_000), Err(PassFault::Lapsed));
    let mut widened = pass.clone();
    widened.lends.push(bos.clone());
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
        alloc::vec![Label::of(&lender(2), "app")],
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
