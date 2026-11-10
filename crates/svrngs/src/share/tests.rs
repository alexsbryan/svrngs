// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs@e1a43bcd2
//! `resolve_target`'s tests, publish_cmd.rs:559-591. (svrngs: a port
//! alone, so a `host:port` is refused with the rest; then the lend's
//! decision, `lend_of`, and the well-known table.)
use super::*;
// The lend's names, as these tests read them before the move.
use svrngs_node::lend::{LendName, Program, RefusedPath};

fn port(p: u16) -> NonZeroU16 {
    NonZeroU16::new(p).unwrap()
}

/// A bare port is the common form and must mean loopback, never the
/// wildcard — the difference between "the house can reach it" and "the
/// coffee shop wifi can".
#[test]
fn a_bare_port_resolves_to_loopback() {
    assert_eq!(resolve_target("5000").unwrap(), port(5000));
}

/// The failing inputs. A routable target is refused rather than accepted
/// and quietly published, because accepting it would put somebody's app
/// on the LAN with no identity on the caller. (svrngs: and a loopback
/// `host:port` too, since a lend is a port on 127.0.0.1.)
#[test]
fn a_routable_or_nonsense_target_is_refused_by_name() {
    for bad in [
        "0.0.0.0:5000",
        "192.168.1.5:80",
        "not-a-port",
        "0",
        "65536",
        "127.0.0.1:8096",
        "[::1]:8096",
    ] {
        let e = resolve_target(bad).unwrap_err();
        assert!(e.contains(bad), "{bad} must be refused, named: {e}");
    }
}

fn share(p: u16) -> Share {
    Share {
        port: port(p),
        name: None,
        kind: None,
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        asks: None,
        sign_in: false,
        readers: false,
        raw: None,
        audience: svrngs_node::lend::Audience::Legacy,
    }
}

/// Every line of the table is a port or `-` for none, and a kind; the one
/// listed by a port is found by it, and every row's kind is read, its kept
/// paths each a path (`kept_of` panics on one that is not).
#[test]
fn the_well_known_table_reads_whole() {
    let rows: Vec<&str> = PROGRAMS
        .iter()
        .flat_map(|(_, file)| file.lines())
        .map(|l| l.split('#').next().unwrap().trim())
        .filter(|l| !l.is_empty())
        .collect();
    assert!(!rows.is_empty());
    for row in rows {
        let (p, rest) = row.split_once(' ').expect(row);
        let kind = Program::parse(rest.split_whitespace().next().unwrap()).expect(row);
        assert!(super::rows().any(|(_, k, _)| k == kind), "{row}");
        let _ = kept_of(&kind, "");
        if rest.contains(" readers: key ") {
            let (header, sent, _) = svrngs_node::share::key_of(&kind, "").expect(row);
            assert!(key_header_fits(header) && sent.contains("{}"), "{row}");
        } else if rest.contains(" readers: ") {
            let how = readers_of(&kind, "").expect(row);
            assert!(
                how.path.starts_with('/') && how.sent.contains("{}"),
                "{row}"
            );
        }
        if p != "-" {
            assert!(
                well_known_kind(port(p.parse().expect(row))).is_some(),
                "{row}"
            );
        }
    }
    assert_eq!(well_known_kind(port(8096)).unwrap().as_str(), "jellyfin");
    let jellyfin = readers_of(&Program::parse("jellyfin").unwrap(), "").unwrap();
    assert_eq!(
        (
            jellyfin.name,
            jellyfin.password,
            jellyfin.token,
            jellyfin.sent
        ),
        (
            "Username",
            "Pw",
            "AccessToken",
            r#"MediaBrowser Token="{}""#
        ),
        "Jellyfin 12.0.0's sign-in, as MOVIE_NIGHT_FOR_TEN's spike ran it"
    );
    assert_eq!(
        well_known_kind(port(8920)),
        None,
        "a TLS port is not shared by number"
    );
    assert_eq!(well_known_kind(port(5000)), None);
}

/// oidc-retire (SIGN_IN_WITH_SVRNGS §7 3): the table names no sign-in
/// helper, and the port the one row held names no kind. Red at HEAD, where
/// the table still carried the helper row for the app that retired; green
/// after the row and the app went.
#[test]
fn the_table_names_no_sign_in_helper() {
    let helpers: Vec<&str> = PROGRAMS
        .iter()
        .flat_map(|(_, file)| file.lines())
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| l.starts_with("helper "))
        .collect();
    assert!(helpers.is_empty(), "a helper row survives: {helpers:?}");
    assert_eq!(
        well_known_kind(port(8097)),
        None,
        "the retired helper's port names no kind"
    );
}

/// **A recipe needs no list a file holds** (apps/pantry/README.md): a lend
/// under `kokoro` refuses what demo/runs/models/2026-10-22-scope/SCOPE.md
/// §1.4 kept back, its lender typing none. Planted, watched failing:
/// kokoro.txt's `keeps:` row removed.
#[test]
fn kokoro_is_lent_without_the_paths_its_file_keeps() {
    let kept = kept_of(&Program::parse("kokoro").unwrap(), "");
    let kept: Vec<&str> = kept.iter().map(RefusedPath::as_str).collect();
    assert_eq!(kept, ["/docs", "/openapi.json"]);
}

/// Row order is the file names', never the filesystem's listing (build.rs):
/// a lender reads it in `NoKind`'s list, and `unrecognized` takes the first
/// slip it finds. Planted: build.rs's sorted paths walked in reverse.
#[test]
fn the_programs_are_read_in_file_name_order() {
    let names: Vec<&str> = PROGRAMS.iter().map(|(word, _)| *word).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    let mut words: Vec<String> = super::rows().map(|(_, k, _)| k.to_string()).collect();
    words.dedup();
    // A file may hold only comments, its place kept until its clause can be
    // read (minecraft.txt, factorio.txt): no row, so no word on either list.
    let with_rows: Vec<&str> = names
        .iter()
        .copied()
        .filter(|name| words.iter().any(|word| word == name))
        .collect();
    assert_eq!(words, with_rows, "the rows come in the files' order");
}

/// The kind is the one given, else the port's; the name, else the kind;
/// the limit, else none. Nothing listening, or no kind to be had, is
/// refused by name.
#[test]
fn a_lend_is_decided_from_what_was_typed_and_what_listens() {
    let jf = Program::parse("jellyfin").unwrap();
    let lend = lend_of(&share(8096), |_| true, Some(jf.clone()), "").unwrap();
    assert_eq!(
        (lend.name.as_str(), lend.kind.as_str(), lend.limit),
        ("jellyfin", "jellyfin", None)
    );
    let mut typed = share(5000);
    typed.kind = Some(Program::parse("notes").unwrap());
    typed.name = Some(LendName::parse("pad").unwrap());
    typed.limit = Some(port(7));
    let lend = lend_of(&typed, |_| true, None, "").unwrap();
    assert_eq!(
        (lend.name.as_str(), lend.kind.as_str(), lend.limit),
        ("pad", "notes", Some(port(7)))
    );
    assert_eq!(
        lend_of(&share(5000), |_| true, None, ""),
        Err(ShareRefusal::NoKind(port(5000)))
    );
    assert_eq!(
        lend_of(&share(8096), |_| false, Some(jf), ""),
        Err(ShareRefusal::NothingListens(port(8096)))
    );
}

/// The table's third column, the kinds that keep their own accounts: one
/// for jellyfin, in DESIGN §1's words, and none for a kind without it or
/// with no row.
#[test]
fn a_kind_keeps_its_own_accounts_only_if_the_table_says_how() {
    let Some(Signs::Accounts(how)) = accounts_of(&Program::parse("jellyfin").unwrap(), "") else {
        panic!("jellyfin keeps its own")
    };
    assert!(
        how.starts_with("your picture at the top right, then Dashboard"),
        "{how}"
    );
    for word in svrngs_node::link::KEPT_WORDS {
        assert!(!how.to_lowercase().contains(word), "{how} says {word}");
    }
    assert_eq!(accounts_of(&Program::parse("pad").unwrap(), ""), None);
    assert_eq!(well_known_kind(port(8096)).unwrap().as_str(), "jellyfin");
}

/// The table's header rows (bash-person (d)): navidrome on 4533 and
/// calibre-web on 8083 take the person from the door's header, each row
/// naming the header the door sets, by the door's own constant, and
/// neither saying a word a person never reads.
#[test]
fn navidrome_and_calibre_web_take_the_person_from_the_door_s_header() {
    for (p, kind) in [(4533, "navidrome"), (8083, "calibre-web")] {
        let k = well_known_kind(port(p)).expect(kind);
        assert_eq!(k.as_str(), kind);
        let Some(Signs::Header(what)) = accounts_of(&k, "") else {
            panic!("{kind} takes the header")
        };
        let header = svrngs_node::door::USER_HEADER;
        assert!(what.to_lowercase().contains(header), "{kind}: {what}");
        for word in svrngs_node::link::KEPT_WORDS {
            assert!(!what.to_lowercase().contains(word), "{what} says {word}");
        }
    }
    let Some(Signs::Header(nd)) = accounts_of(&Program::parse("navidrome").unwrap(), "") else {
        unreachable!()
    };
    assert!(
        nd.contains("ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32"),
        "{nd}"
    );
}

/// A lend shared with no limit lets in everyone in its groups at once —
/// a fourth person too — where it once let in three, a number nobody
/// chose (bash-limit (a); the operator, 2026-10-12). Planted: the limit
/// given, else three.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the time is the counts' argument"
)]
fn a_lend_shared_with_no_limit_lets_a_fourth_in_at_once() {
    use std::time::Instant;
    use svrngs_node::door::limit::{Counts, Verdict};
    use svrngs_node::network::Whereabouts;
    let jf = Program::parse("jellyfin").unwrap();
    let lend = lend_of(&share(8096), |_| true, Some(jf), "").unwrap();
    let (mut counts, now) = (Counts::default(), Instant::now());
    for who in ["ana", "bo", "cy", "dee"] {
        let verdict = counts.ask(&lend, who, Whereabouts::Away, now);
        assert!(matches!(verdict, Verdict::In { .. }), "{who}: {verdict:?}");
    }
}

/// llama-server is known by its lender's word, never by a port (8080 is
/// Open WebUI's too): `share 8080` alone is refused, naming the kinds svrngs
/// knows, llama-server among them; a lend of it keeps back the table's
/// paths, a path its lender also typed once, and one they typed besides
/// (start-llama (a)). Planted: the table's `keeps:` row given port 8080 —
/// `well_known_kind(8080)` then names it.
#[test]
fn llama_server_is_known_by_its_name_and_lent_without_the_table_s_paths() {
    assert_eq!(well_known_kind(port(8080)), None);
    let said = ShareRefusal::NoKind(port(8080)).to_string();
    assert!(
        said.ends_with("audiobookshelf, calibre-web, factorio, immich, jellyfin, kokoro, llama-server, minecraft, navidrome"),
        "{said}"
    );
    let mut typed = share(8080);
    typed.kind = Some(Program::parse("llama-server").unwrap());
    typed.refused = ["/slots", "/admin"]
        .map(|p| RefusedPath::parse(p).unwrap())
        .to_vec();
    let lend = lend_of(&typed, |_| true, None, "").unwrap();
    let refused: Vec<&str> = lend.refused.iter().map(RefusedPath::as_str).collect();
    assert_eq!(
        refused,
        [
            "/slots",
            "/admin",
            "/v1/stream",
            "/v1/streams",
            "/v1/chat/completions/control",
            "/lora-adapters",
            "/metrics",
            "/tools",
            "/cors-proxy",
            "/models"
        ]
    );
    assert_eq!(lend.name.as_str(), "llama-server");
    assert!(kept_of(&Program::parse("jellyfin").unwrap(), "").is_empty());
}

/// **A key row may name no header the door keeps** (CONTRACTS §4,
/// `readers: key <header>`): a key can go in `x-api-key` or `authorization`,
/// and a row naming a person's cookie, the address, the length, anything
/// the door says itself (`x-svrngs-`), or a hop-by-hop header is refused
/// where the row is read, by [`key_header_fits`], so `key_of` answers none and the
/// door never sends it. Plants, each watched failing: the cookie arm of
/// `matches!` dropped; the door check dropped; the name check dropped.
#[test]
fn a_key_row_may_name_only_a_header_of_the_service_s_own() {
    for ok in ["x-api-key", "authorization", "X-Api-Key"] {
        assert!(key_header_fits(ok), "{ok}");
    }
    for no in [
        "cookie",
        "COOKIE",
        "host",
        "content-length",
        "x-svrngs-user",
        "X-Svrngs-Anything",
        "connection",
        "keep-alive",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
        "x api key",
        "a/b",
        "",
    ] {
        assert!(!key_header_fits(no), "a key row naming {no:?} is refused");
    }
    let immich = Program::parse("immich").unwrap();
    assert_eq!(
        key_of(&immich, ""),
        Some(("x-api-key", "{}", "/api/users/me"))
    );
    let books = Program::parse("audiobookshelf").unwrap();
    assert_eq!(
        key_of(&books, ""),
        Some(("authorization", "Bearer {}", "/api/me"))
    );
    assert_eq!(key_of(&Program::parse("jellyfin").unwrap(), ""), None);
}

// ── A group's own programs/<word>.txt (CONTRACTS §2, §6; row 12) ───────

fn program(word: &str) -> Program {
    Program::parse(word).unwrap()
}

/// The refusal [`fits`] gives `text` as `word`'s terms, or none.
fn refused(word: &str, text: &str) -> Option<String> {
    fits(&program(word), text).err()
}

/// **A group's file can say only what the door enforces** (CONTRACTS §6.2):
/// a clause outside the closed set, `carries:` beside `keeps:` or
/// `readers:`, `carries:` of anything but tcp or udp, a row of another
/// program, a line that is no row, a kept "path" with no `/`: each refused
/// where the file is read, the line named. A file of the clauses the door
/// holds fits. Planted, watched failing: `fits` answering `Ok(())` for every
/// file (each refusal below then reads `None`).
#[test]
fn a_group_s_file_says_only_what_the_door_enforces() {
    for (text, named) in [
        ("- notes opens: /admin", "`notes opens: /admin` is no term"),
        (
            "- notes keeps: /admin\n- notes carries: tcp",
            "`carries:` lends notes as bytes",
        ),
        (
            "- notes readers: key x-api-key\n- notes carries: udp",
            "`carries:` lends notes as bytes",
        ),
        ("- notes carries: quic", "carries tcp or udp"),
        (
            "- pad keeps: /admin",
            "`pad keeps: /admin` is another program's",
        ),
        ("notes keeps /admin", "`notes keeps /admin` is not a row"),
        ("- notes keeps: admin", "is not a path"),
        ("- notes keeps:", "says nothing after its colon"),
    ] {
        let said = refused("notes", text);
        assert!(
            said.as_deref().is_some_and(|s| s.contains(named)),
            "{text:?}: {said:?}"
        );
    }
    let mixed =
        "# Sam's notes\n- notes keeps: /admin\n2000 notes accounts: Settings\n- notes carries: tcp";
    assert_eq!(
        refused(
            "notes",
            "- notes keeps: /admin /export\n2000 notes accounts: Settings"
        ),
        None
    );
    assert!(
        refused("notes", mixed).is_some(),
        "carries beside keeps, in any order"
    );
    assert_eq!(
        refused("notes", "- notes carries: udp\n- notes header: set X to Y"),
        None
    );
    assert_eq!(refused("notes", ""), None, "no rows is no terms");
}

/// **A group's file holds no credential and routes none** (CONTRACTS §6.3):
/// a `readers:` sign-in whose path is not the lender's own service
/// (`@evil.example/x`, which `sign_in_readers` would write after the port
/// and so post the typed password to evil.example), one with no `{}` for
/// its token, and a `key` row naming the person's cookie, are refused where
/// the file is read; a key row svrngs can send fits, and is what `share`
/// keeps a pasted key as. Planted, watched failing: `readers_in` without
/// its `path.starts_with('/')` (the `@evil` row then fits).
#[test]
fn a_group_s_readers_row_sends_its_sign_in_only_to_its_own_service() {
    let neither = "is neither `key <header>` nor a sign-in on its own service";
    for row in [
        "- notes readers: @evil.example/x Username Pw AccessToken Bearer {}",
        "- notes readers: /login Username Pw AccessToken Bearer",
        "- notes readers: key cookie",
        "- notes readers: key x-svrngs-user",
    ] {
        let said = refused("notes", row);
        assert!(
            said.as_deref().is_some_and(|s| s.contains(neither)),
            "{row}: {said:?}"
        );
    }
    let key = "- notes readers: key x-api-key";
    assert_eq!(refused("notes", key), None);
    let notes = program("notes");
    assert_eq!(key_of(&notes, key), Some(("x-api-key", "{}", "")));
    assert_eq!(key_of(&notes, ""), None, "no file, no key");
    let sign_in = "- notes readers: /login Username Pw AccessToken Bearer {}";
    assert_eq!(refused("notes", sign_in), None);
    assert_eq!(readers_of(&notes, sign_in).map(|r| r.path), Some("/login"));
}

/// **A group's file only adds to what svrngs ships** (CONTRACTS §2, §6.5):
/// for llama-server, a file that changes anything but `keeps:` is refused,
/// and so is an empty `keeps:`; and even a file read with no check at all
/// leaves every path svrngs keeps back kept, with the group's added. A
/// word svrngs ships nothing for takes the group's `keeps:` alone.
/// Planted, watched failing: `kept_of` taking the last `keeps:` row alone,
/// so a group's row replaces what ships (the lend then refuses `/props`
/// alone, `/slots` open).
#[test]
fn a_group_s_file_only_adds_to_what_svrngs_ships() {
    let ships = "changes the terms svrngs ships for it";
    for row in [
        "- llama-server readers: key x-api-key",
        "- llama-server accounts: anyone",
        "- llama-server carries: tcp",
    ] {
        let said = refused("llama-server", row);
        assert!(
            said.as_deref().is_some_and(|s| s.contains(ships)),
            "{row}: {said:?}"
        );
    }
    assert!(refused("llama-server", "- llama-server keeps:").is_some());
    assert_eq!(
        refused("llama-server", "- llama-server keeps: /props"),
        None
    );

    let mut asked = share(8080);
    asked.kind = Some(program("llama-server"));
    let refused_of = |terms| {
        let lend = lend_of(&asked, |_| true, None, terms).unwrap();
        lend.refused
            .iter()
            .map(|p| p.as_str().to_string())
            .collect::<Vec<_>>()
    };
    let shipped = refused_of("");
    assert!(shipped.contains(&"/slots".to_string()), "{shipped:?}");
    for terms in ["- llama-server keeps:", "- llama-server keeps: /props"] {
        let kept = refused_of(terms);
        assert!(
            shipped.iter().all(|p| kept.contains(p)),
            "{terms}: {kept:?}"
        );
    }
    assert!(refused_of("- llama-server keeps: /props").contains(&"/props".to_string()));
    let mut notes = share(5000);
    notes.kind = Some(program("notes"));
    let lend = lend_of(&notes, |_| true, None, "- notes keeps: /admin").unwrap();
    assert_eq!(lend.refused, [RefusedPath::parse("/admin").unwrap()]);
}

/// **The lend is the yes** (CONTRACTS §6.1): what a group's file keeps is
/// copied into the lend's own declaration, which is all its door reads, so
/// a file changed after changes nothing until it is lent again. Here the
/// lend declared under one file and the file then changed: the declaration
/// read back is the lend as decided, the change nowhere in it. Planted,
/// watched failing: `lend_of` keeping only the shipped paths (`kept_of(&kind,
/// "")`), so the door would have to read the file to hold its terms.
#[test]
fn a_lend_keeps_the_terms_it_was_lent_under() {
    let root = tempfile::tempdir().unwrap();
    let mut asked = share(5000);
    asked.kind = Some(program("notes"));
    let lend = lend_of(&asked, |_| true, None, "- notes keeps: /admin").unwrap();
    svrngs_node::lend::declare(root.path(), &lend, None).unwrap();
    let _changed = lend_of(&asked, |_| true, None, "- notes keeps: /other").unwrap();
    let read = svrngs_node::lend::read(root.path(), &lend.name)
        .unwrap()
        .unwrap();
    assert_eq!(read.refused, [RefusedPath::parse("/admin").unwrap()]);
}

/// **A file that says its program is lent as bytes is held to it**
/// (CONTRACTS §2 `carries:`): a group's `- minecraft carries: tcp` refuses
/// a lend of it as a page, or as UDP, saying the command that lends it; a
/// lend as TCP is decided. Planted, watched failing: `lend_of` without its
/// `carries` check (the page lend is decided).
#[test]
fn a_program_carried_as_bytes_is_lent_only_so() {
    let terms = "- minecraft carries: tcp";
    let mut asked = share(25565);
    asked.kind = Some(program("minecraft"));
    let page = lend_of(&asked, |_| true, None, terms);
    assert_eq!(
        page,
        Err(ShareRefusal::Carries(program("minecraft"), "tcp"))
    );
    assert!(page
        .unwrap_err()
        .to_string()
        .contains("--tcp --program minecraft"));
    asked.raw = Some(svrngs_node::lend::Raw::Udp);
    assert!(lend_of(&asked, |_| true, None, terms).is_err());
    asked.raw = Some(svrngs_node::lend::Raw::Tcp);
    assert!(lend_of(&asked, |_| true, None, terms).is_ok());
}
