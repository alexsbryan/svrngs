//! The worlds of svrngs's xenia scenes (xenia `docs/PLAN.md` row 5; xenia
//! `docs/DESIGN.md` §5, "Scene"). Each stands up what its people meet,
//! through the machinery the demo scenes use
//! (`crates/svrngs/tests/scenes/people.rs`): svrngs computers of this
//! build, each with a `HOME` of its own, the house made on the group's
//! page's address and met at a relay on 127.0.0.1, Money brought from the
//! shelf, and the page served on localhost. Then it prints the scene's one
//! JSON line and stays up until its stdin closes, when every computer and
//! server it started is killed on drop.
//!
//! A test target with no harness, run only by name (`test = false` in
//! `crates/svrngs/Cargo.toml`): each scene's command in `demo/xenia/`
//! runs one, and nothing in `scripts/check.sh` does. A world that cannot
//! be stood up prints no line, so xenia's runner says the scene never ran.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
#[path = "../../../svrngs-node/tests/browser/drive.rs"]
#[allow(dead_code)]
mod drive;
#[path = "../../../svrngs-node/tests/browser/load.rs"]
#[allow(dead_code)]
mod load;
#[path = "../scenes/people.rs"]
#[allow(dead_code)]
mod people;

use serde_json::{json, Value};
use svrngs_core::link::KEPT_WORDS;

use crate::harness::{relay_url, Engine};
use crate::people::{Person, Scene};

/// The group's name, as the pages say it ("You’re in House").
const HOUSE: &str = "House";

// What each person was told, the channels' opening messages (xenia DESIGN
// §1, "Channel"). A first cut, which the operator may rewrite once the
// first accounts are read (xenia PLAN §3). Each says only what its sender
// could know, and no word of svrngs's but its name.

/// Sam, a housemate in no scene's cast, to Kim.
const KIM_TOLD: &str = "Kim, you paid for Friday’s pizza, didn’t you? It was $36 for you and Ana. Can you put it in the house’s money, and let Ana know here?";
/// Sam to Ana, before the pizza: the house's link.
fn ana_link(link: &str) -> String {
    format!("Welcome to the house, Ana! Here’s our link, open it on your phone: {link}")
}
/// Kim to Ana, when Kim is not in the cast: what Kim would have said.
const ANA_PIZZA: &str = "Ana, I put Friday’s pizza in the house’s money: $36 for the two of us. Open the link Sam sent to see your half.";
/// Kim to June, who set svrngs up on June's computer when over.
fn june_told(link: &str) -> String {
    format!("June, here’s the link to join the house: {link} I put svrngs on your computer when I was over, it’s the page that’s open there.")
}

/// Kim's pizza on her own Money page, as Money's page draws an expense she
/// paid: "You paid $36.00 for pizza, split between Kim." (apps/expenses/
/// index.js, `spent`), read, as she would read it to know it is in. Its
/// start, since what it was for is her own words.
const PIZZA: &str = "You paid $36.00 for";

/// Money open, from the home page's "Open Money" (question.js `agreed`),
/// in the tab of its own it opens (home.js `opener`): its form's heading,
/// read (index.js).
const MONEY: &str = "Add what you paid";

/// What Ana owes on her Money page, half of $36 (index.js, "You owe"): a
/// goal counts once she has read it (the operator, 2026-10-26).
const OWES: &str = "You owe $18.00";

/// The house's money says someone besides Kim was in on the pizza, folded
/// by Money's own `expenses.js` from the acts its page reads. Asked again
/// at each check: Money's page draws once, at load and after its own
/// writes (index.js `show`), so a screen of it never redraws for another's
/// share.
const SHARED: &str = "Promise.all([import(location.origin + '/expenses.js'), fetch('svrngs/acts').then((r) => r.json())]).then(([m, read]) => Object.values(m.actsOf(read).reduce(m.reduce, m.init()).expenses).some((e) => /pizza/i.test(e.description) && e.amount_cents === 3600 && e.participants.some((p) => p !== e.payer)))";

/// On a computer's home page at `home`, joined: "You’re in House, as …"
/// (home.js, `joined`).
fn in_house_at(home: &str) -> String {
    let at = home.trim_end_matches('/');
    format!("location.origin === '{at}' && document.body.innerText.includes('You’re in {HOUSE}')")
}

/// The faults every scene declares: what the screen shows when the surface
/// is wrong, whatever the person did.
fn faults() -> Value {
    // DESIGN §1's words kept from people, by their one home
    // (svrngs-core link.rs `KEPT_WORDS`), shown on any screen: demo/b.md
    // "After" counts each time a phone asked for one (`key_prompts`), and
    // the evening of 2026-10-07 kept no count of it
    // (demo/runs/b/2026-10-07.json, `key_prompts` null). Shown is wider
    // than asked, on purpose: a word shown is one a person must look up.
    let kept = format!(
        r"/\b({})s?\b/i.test(document.body.innerText)",
        KEPT_WORDS.join("|")
    );
    json!([
        { "expr": kept },
        // A page asking for a password is an account made: demo/b.md
        // "After", `accounts_created`, and NORTH_STAR §2, "no login".
        { "expr": "document.querySelector('input[type=password]') !== null" },
        // The 2026-10-07 evening's one kept artifact is this refusal
        // (demo/runs/b/2026-10-07/data/dee-room.log; its notes in
        // demo/runs/b/2026-10-07.json), as crates/svrngs/src/cli.rs:550
        // says it.
        { "text": "already running on this computer" },
        // The computer that sent the link, or a lender, not answering
        // (web/app.js:115, `unanswered`; :171, :239, :315): demo/b.md's evening
        // holds only if Sam's computer across town answers each phone.
        { "text": "did not answer" },
        // Money's page when the group's money cannot be read
        // (apps/expenses/index.js:29), and the home page's for its groups
        // (crates/svrngs-node/src/home/home.js:178, :245, :297, :391): a refusal
        // on screen, as the evening's one kept artifact was a refusal no
        // one read until after (demo/runs/b/2026-10-07.json, notes).
        { "text": "can’t be read just now" },
        { "text": "cannot read" },
    ])
}

/// The bounds a newcomer's link is held to, from `page`, the group's page:
/// in, and to what they owe.
fn link_bounds(page: &str) -> Vec<Value> {
    vec![
        // "A link in a chat they already use; ten seconds" (NORTH_STAR §1):
        // the surface's own seconds from the link to being in.
        json!({ "from": { "url": page }, "to": { "text": format!("You’re in {HOUSE}") }, "seconds": 10 }),
        // demo/b.md's gate: from the link to what they came for "within
        // three minutes", here what Ana owes.
        json!({ "from": { "url": page }, "to": { "text": "You owe $18.00" }, "seconds": 180 }),
    ]
}

/// Kim's add, from Money's form shown to her pizza on the page: an act's
/// effect within 10 s of the tap, the bar the dry run held pause and seek
/// to (demo/runs/b/2026-10-06/README.md, "Pause and seek from the phone").
fn add_bound() -> Value {
    json!({ "from": { "text": "Add it, paid by you" }, "to": { "text": PIZZA }, "seconds": 10 })
}

/// `url`'s origin, with no trailing slash.
fn origin(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

/// Once `who`'s computer keeps a folder of `group`, within 60 s: the shelf
/// brings an app into it, and refuses before it is made ("this computer
/// keeps no folder of that group", carried.rs `NO_FOLDER`), as Kim's
/// world was once refused a moment after `new` (2026-10-26). The home page
/// reads it as `folders` in /v1/groups (surface.rs), by the same `dirs_of`
/// the shelf asks.
async fn foldered(who: &Person, group: &str) {
    for _ in 0..120 {
        let groups = who.get("/v1/groups").await;
        let held = groups["groups"].as_array().into_iter().flatten();
        let held = held.into_iter().find(|g| g["id"] == group);
        if held.is_some_and(|g| g["folders"].as_array().is_some_and(|f| !f.is_empty())) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    let log = who.log();
    people::late(format!(
        "{}'s computer keeps no folder of {group} within 60 s; its log is {}",
        who.name,
        log.display()
    ))
    .await
}

/// The house, as each scene finds it: Kim's computer makes it on the page's
/// address, met at the scene's relay, as `svrngs new House --as Kim
/// --origin <page> --relay <relay>` does (night.rs), and brings Money from
/// the shelf, as her home page's "Bring the ticked ones" does (money.rs).
struct House {
    scene: Scene,
    kim: Person,
    id: String,
    /// The group's page, `http://localhost:<port>/`.
    page: String,
    _served: tokio::process::Child,
    /// Kim's Money, where her browser opens it.
    money: String,
}

impl House {
    async fn up(name: &str) -> House {
        let scene = Scene::new(&format!("xenia-{name}"), Engine::Chromium).await;
        let (page, served) = scene.page().await;
        let at = relay_url(&scene.relay);
        let args = [
            "new", HOUSE, "--as", "Kim", "--origin", &page, "--relay", &at,
        ];
        let kim = scene.start("Kim", &args).await;
        let id = kim.group_named(HOUSE).await;
        foldered(&kim, &id).await;
        kim.on_home_page("/shelf", &json!([id, ["Money"]])).await;
        let money = kim.opens(&id, "Money").await;
        eprintln!("xenia {name}: the page at {page}, the relay at {at}, Kim's Money at {money}");
        House {
            scene,
            kim,
            id,
            page,
            _served: served,
            money,
        }
    }

    /// The link Kim's home page makes to invite someone, on the page's
    /// address: a phone opens it, and a computer's home page joins by it.
    async fn link(&self) -> String {
        let asked = json!({ "group": self.id, "device": false });
        let links = self.kim.on_home_page("/link", &asked).await;
        links["https"].as_str().expect("an https link").to_string()
    }

    /// Money where a phone opens it through the page: the lend's label
    /// under the page's host (web/app.js, "Its origin is
    /// `<label>.<this page's host>`"), the label Kim's computer gives it.
    fn money_on_page(&self) -> String {
        let label = self.money.trim_start_matches("http://");
        let label = label.split('.').next().unwrap_or_default();
        let port = origin(&self.page)
            .rsplit(':')
            .next()
            .unwrap_or_default()
            .to_string();
        format!("http://{label}.localhost:{port}/")
    }

    /// What a phone reaches: the page, Money through it, and the relay.
    fn phone_reach(&self) -> Vec<String> {
        let relay = relay_url(&self.scene.relay);
        vec![
            origin(&self.page),
            origin(&self.money_on_page()),
            origin(&relay),
        ]
    }

    /// Ana on her phone, from the link in her opening messages to what she
    /// owes, every step a milestone.
    fn ana(&self) -> Value {
        let page = origin(&self.page);
        json!({
            "as": "Ana", "person": "people/ana.md", "device": "phone", "url": null,
            "goal": { "text": OWES },
            "milestones": [
                // At the group's page from the link; in, read; Money open, its heading read (index.js).
                { "url": format!("{page}/") },
                { "text": format!("You’re in {HOUSE}") },
                { "text": "What people paid for" },
            ],
        })
    }
}

/// The scene's line printed, then the world kept up until stdin closes
/// (xenia DESIGN §5, "Scene").
async fn stay_up(line: Value, world: impl Send + 'static) {
    println!("{line}");
    let read = tokio::task::spawn_blocking(|| {
        std::io::Read::read_to_end(&mut std::io::stdin(), &mut Vec::new())
    });
    let _ = read.await;
    eprintln!("xenia: stdin closed, so the world goes down");
    drop(world);
}

/// **Kim and Ana together.** Kim, on her computer, puts the pizza into the
/// house's money and tells Ana in the house chat; Ana, on her phone, opens
/// the house's link and finds what she owes. Kim's errand is done when the
/// house's money has Ana in on the pizza (the scene's goal, on a screen of
/// Kim's Money); Ana's when her own page says what she owes.
async fn together_world() {
    let house = House::up("together").await;
    let link = house.link().await;
    let mut reach = house.phone_reach();
    reach.push(origin(&house.money));
    reach.push(origin(&house.kim.home_page()));
    let bounds = [link_bounds(&house.page), vec![add_bound()]].concat();
    let line = json!({
        "begins": "Kim on a computer at her home page, in the house already; Ana on a phone with only the house chat open; how each got there not seen",
        "cast": [
            { "as": "Kim", "person": "people/kim.md", "device": "computer", "url": house.kim.home_page(),
              "milestones": [{ "text": MONEY }, { "text": PIZZA }] },
            house.ana(),
        ],
        "screens": [{ "as": "the house’s money", "device": "computer", "url": house.money }],
        "channels": [{
            "name": "the house chat", "members": ["Kim", "Ana"],
            "opening": [
                { "from": "Sam", "to": ["Kim"], "text": KIM_TOLD },
                { "from": "Sam", "to": ["Ana"], "text": ana_link(&link) },
            ],
        }],
        "goal": { "on": "the house’s money", "expr": SHARED },
        "faults": faults(),
        "bounds": bounds,
        "reach": reach,
    });
    stay_up(line, house).await
}

/// **Kim alone.** The computer half of the errand: the pizza in the house's
/// money, on her own page. She may tell Ana in the chat; nobody is there
/// to hear it.
async fn kim_world() {
    let house = House::up("kim").await;
    let line = json!({
        "begins": "on a computer at her home page, in the house already; how the house was made not seen",
        "cast": [{
            "as": "Kim", "person": "people/kim.md", "device": "computer", "url": house.kim.home_page(),
            "goal": { "text": PIZZA },
            "milestones": [{ "text": MONEY }],
        }],
        "channels": [{
            "name": "the house chat", "members": ["Kim"],
            "opening": [{ "from": "Sam", "text": KIM_TOLD }],
        }],
        "faults": faults(),
        "bounds": [add_bound()],
        "reach": [origin(&house.money), origin(&house.kim.home_page())],
    });
    stay_up(line, house).await
}

/// **Ana alone.** The phone half: the pizza already in the house's money,
/// written as Money's page writes it from Kim's computer; Ana opens the
/// house's link and finds what she owes.
async fn ana_world() {
    let house = House::up("ana").await;
    let pizza =
        json!({ "op": { "kind": "expense", "amount_cents": 3600, "description": "pizza" } });
    house
        .kim
        .on_lend(&house.money, "/svrngs/acts", &pizza)
        .await;
    let link = house.link().await;
    let line = json!({
        "begins": "on a phone with only the house chat open; the pizza put in the house’s money by Kim on her computer, not seen",
        "cast": [house.ana()],
        "channels": [{
            "name": "the house chat", "members": ["Ana"],
            "opening": [
                { "from": "Sam", "text": ana_link(&link) },
                { "from": "Kim", "text": ANA_PIZZA },
            ],
        }],
        "faults": faults(),
        "bounds": link_bounds(&house.page),
        "reach": house.phone_reach(),
    });
    stay_up(line, house).await
}

/// **The afraid newcomer.** June's computer runs svrngs as the install
/// line leaves it with no link (web/install.sh; home_group.rs): the bare
/// `svrngs`, in no group, its home page open. Kim sends the house's link
/// in the chat. June is done when her home page says she is in the house:
/// led there by the pages, with no terminal (the operator, 2026-10-13: the
/// home page must lead "someone who is a bit afraid technically").
async fn newcomer_world() {
    let house = House::up("newcomer").await;
    house.scene.installs_from("June", &house.page);
    let june = house.scene.start("June", &[]).await;
    let home = june.home_page();
    let link = house.link().await;
    let mut reach = house.phone_reach();
    reach.push(origin(&home));
    let line = json!({
        "begins": "on a computer where svrngs runs as its install line leaves it, its home page open in the browser, in no group yet; the install not seen",
        "cast": [{
            "as": "June", "person": "people/june.md", "device": "computer", "url": home,
            // A check on the page: her home page, not the group's page in her
            // browser, which says the same words (web/app.js:488), so which
            // computer is in can't be read.
            "goal": { "expr": in_house_at(&home) },
            "milestones": [
                // The home page read the link and names the house on its button (home.js, `says`), read.
                { "text": format!("Join {HOUSE}") },
            ],
        }],
        "channels": [{
            "name": "the house chat", "members": ["June"],
            "opening": [{ "from": "Kim", "text": june_told(&link) }],
        }],
        "faults": faults(),
        "bounds": [
            // demo/b.md's gate, a newcomer from the link to in "within
            // three minutes", from the home page she starts on.
            { "from": { "text": "Join a group from a link" }, "to": { "expr": in_house_at(&home) }, "seconds": 180 },
            // NORTH_STAR §1's ten seconds, from the button that joins.
            { "from": { "text": format!("Join {HOUSE}") }, "to": { "expr": in_house_at(&home) }, "seconds": 10 },
        ],
        "reach": reach,
    });
    stay_up(line, (house, june)).await
}

/// The world its scene's command names, by the command's own name, `$0`
/// as `demo/xenia/world/scene.sh` passes it: judged as the demo scenes are
/// ([`people::judged`]), so a wait that ran out past the load bar is said
/// on stderr as could-not-judge, and no line is printed. With no harness,
/// nothing but the world writes to stdout, so its line is the first.
fn main() {
    let named = std::env::args().nth(1).unwrap_or_default();
    let scene = named.rsplit('/').next().unwrap_or_default().to_string();
    let world: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> =
        match scene.as_str() {
            "together" => Box::pin(together_world()),
            "kim" => Box::pin(kim_world()),
            "ana" => Box::pin(ana_world()),
            "newcomer" => Box::pin(newcomer_world()),
            _ => {
                eprintln!("xenia: no world is called {named:?}: together, kim, ana or newcomer");
                std::process::exit(2)
            }
        };
    let place = format!("xenia-{scene}-chromium");
    let run = tokio::runtime::Runtime::new().expect("a runtime");
    run.block_on(people::judged(place, world));
}
