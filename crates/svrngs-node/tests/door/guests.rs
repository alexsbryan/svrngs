//! A guest at the door (guests-door; svrn-docs `rings/current/GUESTS.md`
//! §3.3-§3.6, §5 "The door"): Sam, Kim's guest, opens Kim's Jellyfin on
//! the pass holders' protocol, which any key may speak, as Kim's guest and
//! never as Kim.

use super::*;
use svrngs_core::alpn::Alpn;
use svrngs_core::pass::{Label, Pass};
use svrngs_node::{OpId, Visit};

/// Sam's visit key, which Sam's page makes fresh for this visit.
const SAM: u8 = 9;
/// The key a lend's origin on Sam's page dials with, holding Sam's pass.
const HOLDER: u8 = 10;

/// Kim's house: Alex keeps it, first by name; Kim's computer, the lender,
/// and Bo's, each holding the group's seed of three.
struct House {
    kim: Node,
    bo: Node,
    dirs: [tempfile::TempDir; 2],
}

impl House {
    async fn new() -> Self {
        let dirs = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
        let kim = start(1, dirs[0].path()).await;
        let bo = start(2, dirs[1].path()).await;
        let people = [("Alex", 7), ("Kim", 1), ("Bo", 2)];
        let seed = Roster::new(
            (people.into_iter())
                .map(|(p, k)| (Person::parse(p).unwrap(), vec![actor_of(&signing(k))]))
                .collect(),
        );
        for node in [&kim, &bo] {
            node.rail().set_seed(NS, &seed).unwrap();
        }
        House { kim, bo, dirs }
    }

    /// Kim has Sam over until `until`, the visit opening `opens`, each
    /// lender's key and lends; Bo's computer is given the act as a round
    /// would give it. The visit's id.
    fn visit(&self, until: i64, opens: &[(&Node, &str)]) -> OpId {
        let mut named =
            std::collections::BTreeMap::<String, std::collections::BTreeSet<String>>::new();
        for (node, lend) in opens {
            let lender = actor_of(node.rail().signer());
            named.entry(lender).or_default().insert(lend.to_string());
        }
        let act = RailAct::Visit(Visit {
            guest: Person::parse("Sam").unwrap(),
            key: actor_of(&signing(SAM)),
            opens: named,
            until,
        });
        let fixture = svrngs_core::tests_support::Fixture {
            act,
            on_behalf_of: None,
        };
        let recorded = self.kim.rail().commit(NS, &fixture).unwrap().unwrap();
        self.bo.rail().ingest_all(NS, recorded.lines()).unwrap();
        recorded.lines()[0].id.clone()
    }

    async fn close(self) {
        self.kim.endpoint().close().await;
        self.bo.endpoint().close().await;
    }
}

/// `to`'s lend `name`, asked on the pass holders' protocol with a pass
/// Sam's visit key signed for it, as a lend's origin on Sam's page asks.
async fn as_sam(to: &Node, name: &str) -> Reply {
    let label = Label::of(to.endpoint().id().as_bytes(), name);
    let holder = signing(HOLDER);
    let pass = Pass::sign(
        &signing(SAM),
        &actor_of(&holder),
        (label.clone(), vec![]),
        unix_now() + 3600,
    );
    let bridge = bridge_as(&holder, to, Alpn::Lend).await;
    raw_to(
        bridge.local_addr(),
        format!(
            "GET /lend/{label}/whoami HTTP/1.1\r\nHost: local\r\nx-svrngs-pass: {}\r\nConnection: close\r\n\r\n",
            pass.encode()
        ),
    )
    .await
}

/// **Sam at Kim's Jellyfin** (part (a); GUESTS §7 bar 1): Kim has Sam over
/// until tomorrow, opening Kim's Jellyfin, and Sam's visit key asks it. Sam
/// is let in, and Jellyfin is told "Sam (Kim's guest)", Sam's visit key as
/// the person and the device, and an account of Sam's own that no member's
/// can be; never Kim, nor Kim's key. Red at HEAD: the door lets a pass's
/// signer in only as a person standing in a group, so Sam is refused
/// `pass`.
#[tokio::test]
async fn sam_opens_kims_jellyfin_as_kims_guest() {
    let house = House::new().await;
    let (jellyfin, heads) = origins::head_origin().await;
    lends_on(house.dirs[0].path(), "jellyfin", jellyfin.port());
    house.visit(unix_now() + 24 * 3600, &[(&house.kim, "jellyfin")]);

    let opened = as_sam(&house.kim, "jellyfin").await;
    assert_eq!(opened.status, 200, "{}", opened.text);
    let head = last(&heads);
    let sam = actor_of(&signing(SAM));
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        ["Sam (Kim's guest)"],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-key"),
        [sam.as_str()],
        "{head}"
    );
    assert_eq!(
        header_values(&head, "x-svrngs-device"),
        [sam.as_str()],
        "{head}"
    );
    let user = header_values(&head, "x-svrngs-user");
    assert_eq!(user, [format!("Sam-{}.guest", &sam[..16])], "{head}");
    let kim = actor_of(&signing(1));
    assert!(!head.contains(&kim), "Kim's key reached Jellyfin: {head}");
    house.close().await;
}
