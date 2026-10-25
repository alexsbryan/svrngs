//! Removal through the node, as the command asks it (U7; DESIGN §6
//! "Removal has one decider"): `POST /v1/removals` and `/v1/handovers` on
//! the record API, three nodes in one process, no relay, peers reached by
//! seeded addresses as `group_not_mesh.rs` reaches them.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;
use std::time::Duration;

use tracing_subscriber::layer::SubscriberExt;

use svrngs_node::iroh::{PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::removal::{Outcome, RemovalAsk, HANDOVERS_ROUTE, REMOVALS_ROUTE};
use svrngs_node::{actor_of, Person, RailAct, Roster, SigningKey};

const NS: &str = "trip";

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from_bytes(&signing(seed).verifying_key().to_bytes()).unwrap()
}

fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
}

fn no_seed(_: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}

async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
    plant();
    Node::start(
        dir.path().to_path_buf(),
        signing(seed),
        &no_relay(),
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// Sam's group: Sam's key (1) its seed, so Sam decides removals.
fn sams() -> Roster {
    Roster::new(
        [(Person::parse("sam").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    )
}

fn admit(person: &str, joining: u8) -> RailAct {
    RailAct::Admit {
        person: Person::parse(person).unwrap(),
        key: actor_of(&signing(joining)),
        invite: None,
    }
}

/// What the command asks `node` at `route`, and the node's answer.
async fn ask(node: &Node, route: &str, person: &str) -> (u16, Outcome) {
    let answer = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{}{route}", node.api_addr()))
        .json(&RemovalAsk {
            group: NS.into(),
            person: person.into(),
            joined: None,
        })
        .send()
        .await
        .unwrap();
    let status = answer.status().as_u16();
    (status, answer.json().await.unwrap())
}

fn stands(node: &Node, seed: u8) -> bool {
    node.rail()
        .register(NS)
        .unwrap()
        .members
        .contains_key(&actor_of(&signing(seed)))
}

/// Sam, Ana and Bo, each a computer holding the group, Sam's copy the one
/// that admitted the others; each reaches the others by address.
async fn three(dirs: &[tempfile::TempDir; 3]) -> [Node; 3] {
    let nodes = [
        start(1, &dirs[0]).await,
        start(2, &dirs[1]).await,
        start(3, &dirs[2]).await,
    ];
    knit(&nodes, NS, &sams(), [("ana", 2), ("bo", 3)]).await;
    nodes
}

/// `nodes` made one group, `ns`: `seed` its seed on every copy, `admitted`
/// admitted on the first's, each node reaching the others by address, and
/// the others holding the group by one round each.
async fn knit(nodes: &[Node; 3], ns: &str, seed: &Roster, admitted: [(&str, u8); 2]) {
    for n in nodes {
        n.rail().set_seed(ns, seed).unwrap();
    }
    let fixture = |act| svrngs_core::tests_support::Batch(vec![act]);
    for (person, k) in admitted {
        nodes[0]
            .rail()
            .commit(ns, &fixture(admit(person, k)))
            .unwrap()
            .unwrap();
    }
    for (i, n) in nodes.iter().enumerate() {
        for (j, m) in nodes.iter().enumerate() {
            if i != j {
                n.transport()
                    .add_known_peer(key(j as u8 + 1), loopback_sockets(m.endpoint()));
            }
        }
    }
    for n in &nodes[1..] {
        tokio::time::timeout(Duration::from_secs(30), n.round())
            .await
            .unwrap();
    }
}

/// **Only the person who decides makes a removal count, and the removed
/// computer is told** (U7 (e)). Ana asks to remove Bo: written, Bo stands,
/// the answer names Sam as who decides. Sam asks the same: Bo is out, and
/// Bo's own computer, told the cut, holds Bo out too; then Bo's
/// removal of Ana is refused there, as Bo is not in the group. Red with the
/// fold's keeper check out (membership.rs, `let keeps = true`): Ana's ask
/// is the cut.
#[tokio::test]
async fn the_keeper_removes_and_the_removed_computer_is_told() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [sam, ana, bo] = three(&dirs).await;
    assert!(
        stands(&ana, 3) && stands(&bo, 3),
        "control: bo stands on every copy"
    );

    let (status, asked) = ask(&ana, REMOVALS_ROUTE, "bo").await;
    assert_eq!(status, 200);
    assert_eq!(
        asked,
        Outcome::Asked {
            decides: Some(Person::parse("sam").unwrap())
        }
    );
    assert!(stands(&ana, 3), "Ana's ask cuts nobody");

    assert_eq!(ask(&sam, REMOVALS_ROUTE, "bo").await, (200, Outcome::Out));
    assert!(!stands(&sam, 3), "Bo is out on Sam's copy");
    let told = tokio::time::timeout(Duration::from_secs(10), async {
        while stands(&bo, 3) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(told.is_ok(), "Bo's computer was told its cut within 10 s");
    assert!(bo
        .rail()
        .register(NS)
        .unwrap()
        .removed
        .contains_key(&actor_of(&signing(3))));
    assert_eq!(ask(&bo, REMOVALS_ROUTE, "ana").await, (409, Outcome::NotIn));
    assert!(stands(&sam, 2), "Ana stands");
}

/// **Only the person who decides hands deciding on** (U7 (e)). Ana's
/// hand-on is refused, naming Sam; Sam's to Ana is done, and then Ana's
/// removal of Sam is the cut. A person not in the group is said so.
#[tokio::test]
async fn only_the_keeper_hands_on() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [sam, ana, _bo] = three(&dirs).await;
    assert_eq!(
        ask(&ana, HANDOVERS_ROUTE, "ana").await,
        (
            409,
            Outcome::NotDecider {
                decides: Some(Person::parse("sam").unwrap())
            }
        )
    );
    assert_eq!(
        ask(&sam, REMOVALS_ROUTE, "dee").await,
        (409, Outcome::NotInGroup)
    );
    assert_eq!(
        ask(&sam, REMOVALS_ROUTE, "sam").await,
        (409, Outcome::Decides)
    );
    assert_eq!(
        ask(&sam, HANDOVERS_ROUTE, "ana").await,
        (200, Outcome::Handed)
    );
    tokio::time::timeout(Duration::from_secs(30), ana.round())
        .await
        .unwrap();
    assert_eq!(ask(&ana, REMOVALS_ROUTE, "sam").await, (200, Outcome::Out));
}

/// Whether `node`'s copy holds `seed`'s key cut, within `within`.
async fn holds_cut(node: &Node, seed: u8, within: Duration) -> bool {
    tokio::time::timeout(within, async {
        while !node
            .rail()
            .register(NS)
            .unwrap()
            .removed
            .contains_key(&actor_of(&signing(seed)))
        {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .is_ok()
}

/// **A removal goes now, not at the next round** (U7 (e); u7-removal (f)'s
/// run, demo/runs/u7/2026-10-11/README.md, take 1: Ana's lend refused Bo
/// 43.3 s after Sam's command, at Ana's next round). Ana's ask reaches Sam,
/// Sam's cut reaches Ana, and Sam's hand-on to Ana reaches her, each within
/// 5 s, with every round 60 s apart and none asked for. Red with the
/// removal route waking no round (removal.rs at 7412588).
#[tokio::test]
async fn a_removal_reaches_every_computer_without_waiting_for_a_round() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [sam, ana, _bo] = three(&dirs).await;
    let soon = Duration::from_secs(5);
    // Ana's ask is her first act, so any act of hers held is it.
    let asks = |node: &Node| {
        node.rail()
            .digest(NS)
            .unwrap()
            .get(&actor_of(&signing(2)))
            .is_some()
    };

    assert_eq!(
        ask(&ana, REMOVALS_ROUTE, "bo").await.1,
        Outcome::Asked {
            decides: Some(Person::parse("sam").unwrap())
        }
    );
    let reached = tokio::time::timeout(soon, async {
        while !asks(&sam) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(
        reached.is_ok(),
        "Ana's ask reached Sam's computer within 5 s"
    );

    assert_eq!(ask(&sam, REMOVALS_ROUTE, "bo").await, (200, Outcome::Out));
    assert!(
        holds_cut(&ana, 3, soon).await,
        "Sam's cut reached Ana's computer within 5 s"
    );

    assert_eq!(
        ask(&sam, HANDOVERS_ROUTE, "ana").await,
        (200, Outcome::Handed)
    );
    let handed = tokio::time::timeout(soon, async {
        while ana.rail().register(NS).unwrap().keeper != Some(actor_of(&signing(2))) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(
        handed.is_ok(),
        "Sam's hand-on reached Ana's computer within 5 s"
    );
}

/// `node`'s copy of the group's seed, the file the register reads first.
fn seed_path(node: &Node) -> PathBuf {
    node.rail().group_dir(NS).unwrap().join("roster.json")
}

/// The plant for #123: armed with a thread and a seed, a layer that makes
/// the seed unreadable as the core gives that thread its hand-on verdict,
/// so the commit reads the register and the read after it cannot.
static ARMED: Mutex<Option<(ThreadId, PathBuf)>> = Mutex::new(None);

struct Unreadable;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Unreadable {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        use std::os::unix::fs::PermissionsExt;
        struct Message(String);
        impl tracing::field::Visit for Message {
            fn record_debug(&mut self, f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
                if f.name() == "message" {
                    self.0 = format!("{v:?}");
                }
            }
        }
        let mut m = Message(String::new());
        event.record(&mut m);
        if m.0 != "hand-on: the fold's verdict" {
            return;
        }
        match &*ARMED.lock().unwrap() {
            Some((thread, seed)) if *thread == std::thread::current().id() => {
                std::fs::set_permissions(seed, std::fs::Permissions::from_mode(0o000)).unwrap()
            }
            _ => {}
        }
    }
}

/// The plant is the process's one subscriber, set before any node starts.
/// Set per test with `set_default`, it missed the verdict in 5 of 60 runs
/// under load, only beside the file's other tests: a line another thread
/// met first as it was set was cached as never wanted (tracing-core
/// 0.1.36 callsite.rs `register`, `DISPATCHERS.rebuilder()`).
fn plant() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        tracing::subscriber::set_global_default(tracing_subscriber::registry().with(Unreadable))
            .expect("the plant is the process's one subscriber");
    });
}

/// **A register that cannot be read is said as itself** (REVIEW_FINDINGS
/// #123). Ana's hand-on is refused, and her computer cannot read the
/// register to name who decides: she is told so, not told the words of a
/// group nobody decides. The seed is made unreadable between the core's
/// refusal and the read that names who decides (the plant, [`Unreadable`]).
#[tokio::test]
async fn an_unreadable_register_is_said_on_a_hand_on() {
    use std::os::unix::fs::PermissionsExt;
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [_sam, ana, _bo] = three(&dirs).await;
    let seed = seed_path(&ana);
    let readable = || std::fs::set_permissions(&seed, std::fs::Permissions::from_mode(0o644));

    *ARMED.lock().unwrap() = Some((std::thread::current().id(), seed.clone()));
    let after = ask(&ana, HANDOVERS_ROUTE, "ana").await;
    *ARMED.lock().unwrap() = None;
    let unread = ana.rail().register(NS).is_err();
    readable().unwrap();
    assert!(
        unread,
        "the plant: Ana's register turned unreadable after the refusal"
    );
    assert_eq!(after, (409, Outcome::NotDeciderUnread));
}

/// #211, a leave that outruns its leaver's last act (the survey,
/// demo/runs/leave/2026-10-21/SURVEY.md, whose `leave_race_node.rs` this
/// is): three computers, each one's round a day apart (`Node::start_every`),
/// so only a wake or a round the test asks moves anything, and no sleep
/// decides what a copy holds, only polls for what a wake already sent.
mod leave_race {
    use super::*;
    use svrngs_core::tests_support::{applied, record, Batch};
    use svrngs_node::removal::{LeaveAsk, LEAVES_ROUTE};

    const NS: &str = "oak";
    const DAY: Duration = Duration::from_secs(24 * 60 * 60);

    async fn start(seed: u8, dir: &tempfile::TempDir) -> Node {
        let api = "127.0.0.1:0".parse().unwrap();
        Node::start_every(
            dir.path().to_path_buf(),
            signing(seed),
            &no_relay(),
            no_seed,
            api,
            DAY,
        )
        .await
        .expect("a node starts")
    }

    /// Oak: Ana's key (1) its seed, so Ana keeps it; Kim (2) and Bo (3)
    /// admitted by Ana; every copy holding all three, by one round each.
    async fn oak(dirs: &[tempfile::TempDir; 3]) -> [Node; 3] {
        let nodes = [
            start(1, &dirs[0]).await,
            start(2, &dirs[1]).await,
            start(3, &dirs[2]).await,
        ];
        let seed =
            Roster::new([(Person::parse("ana").unwrap(), vec![actor_of(&signing(1))])].into());
        knit(&nodes, NS, &seed, [("kim", 2), ("bo", 3)]).await;
        nodes
    }

    /// `node` writes a money entry, `what`, and the lines it wrote; no round
    /// is woken (`RingRail::commit` wakes none, as a Visit's write wakes none,
    /// #212).
    fn writes(node: &Node, what: &str) -> Vec<svrngs_node::Op<svrngs_node::SignedOp>> {
        let done = node
            .rail()
            .commit(NS, &Batch(vec![record(what)]))
            .unwrap()
            .unwrap();
        done.lines().to_vec()
    }

    fn http() -> reqwest::Client {
        reqwest::Client::builder().no_proxy().build().unwrap()
    }

    async fn leaves(node: &Node) -> Outcome {
        let answer = http()
            .post(format!("http://{}{LEAVES_ROUTE}", node.api_addr()))
            .json(&LeaveAsk { group: NS.into() })
            .send()
            .await
            .unwrap();
        answer.json().await.unwrap()
    }

    async fn removes(node: &Node, person: &str) -> Outcome {
        let ask = RemovalAsk {
            group: NS.into(),
            person: person.into(),
            joined: None,
        };
        let answer = http()
            .post(format!("http://{}{REMOVALS_ROUTE}", node.api_addr()))
            .json(&ask)
            .send()
            .await
            .unwrap();
        answer.json().await.unwrap()
    }

    /// Whether `node`'s copy holds `seed`'s key cut, polled for up to 10 s:
    /// the cut was already sent, by the wake its write made.
    async fn holds_cut(node: &Node, seed: u8) -> bool {
        let cut = || {
            node.rail()
                .register(NS)
                .unwrap()
                .removed
                .contains_key(&actor_of(&signing(seed)))
        };
        tokio::time::timeout(Duration::from_secs(10), async {
            while !cut() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .is_ok()
    }

    /// Every copy rounds, three times over: each exchange there is to have.
    async fn rounds(nodes: &[&Node]) {
        for _ in 0..3 {
            for n in nodes {
                tokio::time::timeout(Duration::from_secs(30), n.round())
                    .await
                    .unwrap();
            }
        }
    }

    /// What a copy's person reads of the record: `GET /v1/rail/log`'s
    /// completeness and each gap's sentence, and the money entries that apply.
    async fn says(name: &str, node: &Node) -> Vec<String> {
        let log: serde_json::Value = http()
            .get(format!(
                "http://{}/v1/rail/log?namespace={NS}",
                node.api_addr()
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let gaps: Vec<&str> = (log["gaps"].as_array().unwrap().iter())
            .map(|g| g["message"].as_str().unwrap())
            .collect();
        let entries = applied(&node.rail().admission(NS).unwrap());
        eprintln!(
            "  {name:<4} entries {entries:?}; complete {}; gaps {gaps:?}",
            log["complete"]
        );
        entries
    }

    /// **A leave reaches every copy with the acts it covers** (#211). Kim's
    /// first entry reaches everyone by a round; her last is written with no
    /// round, and she leaves. Only her leave's telling can carry that entry:
    /// after it her round tells only the leave, nobody dials a key cut
    /// (ring_sync.rs, `standing.names`), and the sync route takes none of a
    /// cut asker's acts (ring_routes.rs `told_cut`). Red at 153ce69d, the
    /// leave told as its `Remove` alone (removal.rs `tell`): Ana's and Bo's
    /// copies applied `[]` for good. Planted with the tell's acts the leave's
    /// lines alone, or `told_with` keeping only the `Remove`s: red here.
    #[tokio::test]
    async fn a_leave_reaches_every_copy_with_the_acts_it_covers() {
        let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
        let [ana, kim, bo] = oak(&dirs).await;
        writes(&kim, "kim-paid");
        rounds(&[&kim]).await;
        writes(&kim, "kim-last");
        assert_eq!(leaves(&kim).await, Outcome::Left);
        assert!(
            holds_cut(&ana, 2).await && holds_cut(&bo, 2).await,
            "the leave reached both"
        );
        rounds(&[&ana, &bo, &kim]).await;
        eprintln!("after Kim's leave and three rounds of every computer:");
        let kims = says("Kim", &kim).await;
        let (anas, bos) = (says("Ana", &ana).await, says("Bo", &bo).await);
        assert_eq!(kims, ["kim-paid", "kim-last"], "control: Kim's copy");
        assert_eq!(anas, kims, "Ana's copy agrees with Kim's");
        assert_eq!(bos, kims, "Bo's copy agrees with Kim's");
    }

    /// **It heals when a copy that stands holds the act** (passes before
    /// #211's fix and after). Bo's copy took Kim's last entry before her
    /// leave; Ana's did not. After Ana's round with Bo (sync.rs:
    /// author-blind), Ana's copy agrees with Kim's. Before #211's fix the
    /// round was what healed it, Kim's entries counting nowhere on Ana's copy
    /// until then; with it the leave reaches Ana with them, so the copy is
    /// not read between.
    #[tokio::test]
    async fn a_leave_heals_where_a_standing_copy_holds_its_last_act() {
        let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
        let [ana, kim, bo] = oak(&dirs).await;
        writes(&kim, "kim-paid");
        rounds(&[&kim]).await;
        let last = writes(&kim, "kim-last");
        bo.rail().ingest_all(NS, &last).unwrap();
        assert_eq!(leaves(&kim).await, Outcome::Left);
        assert!(holds_cut(&ana, 2).await, "the leave reached Ana");
        rounds(&[&ana]).await;
        eprintln!("after Ana's round with Bo:");
        assert_eq!(says("Ana", &ana).await, ["kim-paid", "kim-last"]);
        assert_eq!(says("Kim", &kim).await, ["kim-paid", "kim-last"]);
    }

    /// **A keeper's removal carries what it names** (HEAD; passes). Ana keeps
    /// Oak and holds Kim's entry `kim-seen`; Bo's copy does not; `kim-unseen`
    /// reached nobody. Ana removes Kim: her round's exchange gives Bo the cut
    /// and what it names together, and `kim-unseen`, above what the cut names,
    /// counts nowhere on any copy, Kim's included. The copies agree.
    #[tokio::test]
    async fn a_keepers_removal_reaches_every_copy_with_what_it_names() {
        let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
        let [ana, kim, bo] = oak(&dirs).await;
        let seen = writes(&kim, "kim-seen");
        ana.rail().ingest_all(NS, &seen).unwrap();
        writes(&kim, "kim-unseen");
        assert_eq!(removes(&ana, "kim").await, Outcome::Out);
        assert!(
            holds_cut(&bo, 2).await && holds_cut(&kim, 2).await,
            "the cut reached both"
        );
        rounds(&[&ana, &bo, &kim]).await;
        eprintln!("after Ana removed Kim and three rounds of every computer:");
        for (name, node) in [("Ana", &ana), ("Bo", &bo), ("Kim", &kim)] {
            assert_eq!(says(name, node).await, ["kim-seen"], "{name}'s copy");
        }
    }

    /// **The leaver's computer takes nothing more of the group** (#211;
    /// LEAVING.md §3.4, "no round, no sync"). Bo writes an entry with no
    /// round; Kim leaves. Her tell asks each computer's digest first, and
    /// Bo's answer carries Bo's entry: Kim's copy must not hold it, while
    /// Ana's and Bo's hold Kim's entry. Red with the tell keeping the first
    /// answer's acts (removal.rs `told`, `ingest_all` of the answer).
    #[tokio::test]
    async fn a_leave_takes_nothing_more_of_the_group() {
        let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
        let [ana, kim, bo] = oak(&dirs).await;
        writes(&kim, "kim-last");
        writes(&bo, "bo-unsent");
        assert_eq!(leaves(&kim).await, Outcome::Left);
        assert!(
            holds_cut(&ana, 2).await && holds_cut(&bo, 2).await,
            "the leave reached both"
        );
        eprintln!("after Kim's leave:");
        assert_eq!(
            says("Kim", &kim).await,
            ["kim-last"],
            "Kim's copy takes nothing of Bo's"
        );
        assert_eq!(says("Ana", &ana).await, ["kim-last"]);
        assert_eq!(says("Bo", &bo).await, ["kim-last", "bo-unsent"]);
    }

    /// **A person's other computer is told the leave of the key that signed
    /// it** (#179, #211). Kim stands on two computers (keys 2 and 4, one seed
    /// row); the first cannot reach the second, so the second, off as far as
    /// the first can tell, hears the leave when it asks Ana to sync
    /// (ring_routes.rs `told_cut`). `Leave` writes the second's `Remove`
    /// before the signing key's own (svrngs-core register.rs `Leave`), so the
    /// second must be told through the signer's highest `Remove`: its copy
    /// holds both keys cut. Red with the selection naming the signer's first
    /// `Remove` (ring_routes.rs `told_with`, `.min()` for `.max()`): key 2
    /// stands on the second computer's copy.
    #[tokio::test]
    async fn a_persons_other_computer_is_told_the_signing_keys_leave() {
        let dirs = [(); 4].map(|_| tempfile::tempdir().unwrap());
        let mut nodes = Vec::new();
        for (i, dir) in dirs.iter().enumerate() {
            nodes.push(start(i as u8 + 1, dir).await);
        }
        // Kim's two keys are one person's, as a seed row's keys are.
        let kim = vec![actor_of(&signing(2)), actor_of(&signing(4))];
        let ana = vec![actor_of(&signing(1))];
        let rows = [("ana", ana), ("kim", kim)].map(|(p, k)| (Person::parse(p).unwrap(), k));
        for n in &nodes {
            n.rail()
                .set_seed(NS, &Roster::new(rows.clone().into()))
                .unwrap();
        }
        let bo = Batch(vec![admit("bo", 3)]);
        nodes[0].rail().commit(NS, &bo).unwrap().unwrap();
        for (i, n) in nodes.iter().enumerate() {
            for (j, m) in nodes.iter().enumerate() {
                // Kim's first computer (index 1) never reaches her second (3).
                if i != j && (i, j) != (1, 3) {
                    n.transport()
                        .add_known_peer(key(j as u8 + 1), loopback_sockets(m.endpoint()));
                }
            }
        }
        for n in &nodes[1..] {
            tokio::time::timeout(Duration::from_secs(30), n.round())
                .await
                .unwrap();
        }
        let (ana, kim, other) = (&nodes[0], &nodes[1], &nodes[3]);
        assert_eq!(leaves(kim).await, Outcome::Left);
        assert!(
            holds_cut(ana, 2).await && holds_cut(ana, 4).await,
            "the leave reached Ana"
        );
        assert!(
            !holds_cut(other, 4).await,
            "the first computer did not reach the second"
        );
        tokio::time::timeout(Duration::from_secs(30), other.round())
            .await
            .unwrap();
        let cut = |seed| {
            other
                .rail()
                .register(NS)
                .unwrap()
                .removed
                .contains_key(&actor_of(&signing(seed)))
        };
        assert!(cut(4), "the second computer reads itself out");
        assert!(cut(2), "and the key that signed the leave");
    }
}

/// **The keeper's computer cuts over once** (capability-signed; demo/runs/
/// authorship/2026-10-24/DESIGN.md §5): on its round Sam's computer, which
/// keeps the group, writes its `Signers`; Ana's and Bo's, which do not,
/// write none; a second round writes no other; and Ana's copy holds Sam's
/// after her next round, so her fold refuses what Sam's does. Planted,
/// watched failing: the round not asking for the cutover (node.rs
/// `say_relay`'s `cut_over`).
#[tokio::test]
async fn the_keepers_computer_cuts_over_once() {
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let [sam, ana, _bo] = three(&dirs).await;
    let cutover = |n: &Node| n.rail().register(NS).unwrap().signers;
    let written = |n: &Node| {
        let journal = n.rail().group_dir(NS).unwrap().join("ring_oplog.jsonl");
        let text = std::fs::read_to_string(journal).unwrap_or_default();
        text.matches(r#""op":"signers""#).count()
    };
    assert_eq!(
        cutover(&ana),
        None,
        "a computer not keeping the group writes none"
    );
    assert_eq!(written(&ana), 0);
    for _ in 0..2 {
        tokio::time::timeout(Duration::from_secs(30), sam.round())
            .await
            .unwrap();
    }
    let first = cutover(&sam).expect("the keeper's computer cuts over on its round");
    assert_eq!(written(&sam), 1, "once");
    tokio::time::timeout(Duration::from_secs(30), ana.round())
        .await
        .unwrap();
    assert_eq!(cutover(&ana), Some(first));
    assert_eq!(written(&ana), 1, "carried, and none of Ana's own");
}
