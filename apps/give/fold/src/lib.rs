//! Who keeps each app: give's one fold (svrn-docs rings/current/GIVE.md §7,
//! §7a decision 2). The keeper program folds it to know whether to lend an
//! app; the carried page folds it to say who keeps one, and later to offer a
//! takeover when the keeper is gone. Two implementations would be two
//! deciders (AGENTS.md 8), so this one is built twice: linked by the keeper
//! program, and to plain JavaScript for the page (apps/give/build.sh).
//!
//! It reads give's own acts, `give/<kind>` with the prefix taken off, in the
//! record's order, as `GET /svrngs/acts` gives them to the page and the
//! keeper program reads them from `/v1/rail/log`:
//!
//! - `ready`: the computer that wrote it can keep apps (it has Podman and
//!   the keeper program). Any computer of any member may say it (who may
//!   keep is the operator's to answer, CHARTER; any member until then).
//! - `started {app, card, agreed?}`: the writer's computer keeps an app
//!   nobody keeps, from fresh state.
//! - `moved {app, card, over, from, sum}`: the computer that keeps it, on
//!   a yes to an offer, hands it to the computer offered, which starts from
//!   the snapshot `from` once its copy there sums to `sum`. Every move is a
//!   person's yes (CHARTER), so a move with none counts nowhere.
//! - `taken {app, card, over, from, agreed?}`: the writer's computer keeps
//!   it from the snapshot `from`, the keeper gone.
//! - `offered {app, over, to}`: the person whose computer keeps it offers
//!   it to the computer `to` (GIVE.md §3 scene 3, "Move it to…").
//! - `accepted {app, offer, agreed?}` and `declined {app, offer}`: the
//!   person of that computer says yes or no to the offer, a yes with their
//!   own answer to the card's asks, since a game's terms bind whoever runs
//!   it. A no leaves everything as it was.
//!
//! Each of `started`, `moved` and `taken` names the keeping it follows,
//! `over` (none for a start), so two at once stand by admission order: the first follows the
//! keeping there was, and the later follows one that is gone (`stale`).
//! `card` is the card's version and `from` the snapshot each starts from.
//! An act refused is said with why, never dropped (AGENTS.md 1), and each
//! caller traces it.

use std::collections::BTreeMap;

use serde_json::{Value, json};

/// How an app came to its keeper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    Started,
    Moved,
    Taken,
}

impl How {
    fn as_str(self) -> &'static str {
        match self {
            How::Started => "started",
            How::Moved => "moved",
            How::Taken => "taken",
        }
    }
}

/// An offer of an app to another computer, and its answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    pub act: String,
    /// The key of the computer it is offered to, and that computer's person.
    pub to: String,
    pub person: String,
    pub person_key: String,
    pub since: i64,
    pub answer: Answer,
    /// Whether that person said yes to the card's asks with their yes.
    pub agreed: bool,
}

/// What the person an offer names has said to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Waiting,
    Yes,
    No,
}

impl Answer {
    fn as_str(self) -> &'static str {
        match self {
            Answer::Waiting => "waiting",
            Answer::Yes => "yes",
            Answer::No => "no",
        }
    }
}

/// Who keeps one app now, and from what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keeping {
    /// The key of the computer that keeps it.
    pub keeper: String,
    pub person: String,
    pub person_key: String,
    /// The act that made it the keeper.
    pub act: String,
    pub how: How,
    /// The snapshot it starts from, a path in the group's folder; none for
    /// fresh state.
    pub from: Option<String>,
    /// What the snapshot's files sum to, for a move: the keeper starts from
    /// it only once its copy there sums the same.
    pub sum: Option<String>,
    /// The card's version it runs.
    pub card: String,
    /// Whether whoever keeps it said yes to the card's asks.
    pub agreed: bool,
    pub since: i64,
    /// The last offer of it to another computer, while this keeping lasts.
    pub offer: Option<Offer>,
}

/// A computer that said it can keep apps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Computer {
    pub person: String,
    pub person_key: String,
    pub since: i64,
}

/// Why an act counts nowhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Voided,
    /// A correction counts through the act it voids, which the record marks.
    Correction {
        corrects: String,
    },
    NotGives {
        kind: String,
    },
    Missing {
        field: &'static str,
        kind: &'static str,
    },
    /// The writer, or the computer a move names, never said it can keep.
    CannotKeep {
        computer: String,
    },
    /// It followed a keeping that is not the one there was.
    Stale {
        kept: Option<String>,
        over: Option<String>,
    },
    NotTheKeeper {
        writer: String,
        keeper: String,
    },
    /// An offer from a person other than the one whose computer keeps it.
    NotYours {
        writer: String,
        keeper: String,
    },
    /// An offer to the computer that keeps it.
    KeptThere {
        computer: String,
    },
    /// An offer while one said yes to stands: the app is moving.
    Moving {
        offer: String,
    },
    /// An answer to an offer that is not the one waiting.
    NotOffered {
        answers: String,
        waiting: Option<String>,
    },
    /// An answer from a person other than the one the offer names.
    NotAsked {
        writer: String,
        asked: String,
    },
    /// A move that follows no yes to an offer.
    NoYes,
}

/// An act the fold refused, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub act: String,
    pub app: Option<String>,
    pub refusal: Refusal,
}

impl Refused {
    /// The refusal's name, for a caller that acts on one (the page, on its
    /// own start refused as `cannot_keep`).
    pub fn code(&self) -> &'static str {
        match self.refusal {
            Refusal::Voided => "voided",
            Refusal::Correction { .. } => "correction",
            Refusal::NotGives { .. } => "not_give_s",
            Refusal::Missing { .. } => "missing",
            Refusal::CannotKeep { .. } => "cannot_keep",
            Refusal::Stale { .. } => "stale",
            Refusal::NotTheKeeper { .. } => "not_the_keeper",
            Refusal::NotYours { .. } => "not_yours",
            Refusal::KeptThere { .. } => "kept_there",
            Refusal::Moving { .. } => "moving",
            Refusal::NotOffered { .. } => "not_offered",
            Refusal::NotAsked { .. } => "not_asked",
            Refusal::NoYes => "no_yes",
        }
    }
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (act, app) = (&self.act, self.app.as_deref().unwrap_or_default());
        match &self.refusal {
            Refusal::Voided => write!(f, "{act} was voided, so it counts nowhere"),
            Refusal::Correction { corrects } => {
                write!(f, "{act} corrects {corrects}, and counts through the act it voids")
            }
            Refusal::NotGives { kind } => write!(f, "{act} is of kind {kind}, which give does not read"),
            Refusal::Missing { field, kind } => write!(f, "{act} names no {field}, which a {kind} act needs"),
            Refusal::CannotKeep { computer } => write!(
                f,
                "{computer} has not said it can keep apps: it has no Podman or no keeper program"
            ),
            Refusal::Stale { kept, over } => {
                let kept = kept.as_ref().map_or("by nobody".into(), |k| format!("since {k}"));
                let over = over.as_deref().unwrap_or("nothing");
                write!(f, "{app} was kept {kept} when {act} followed {over}")
            }
            Refusal::NotTheKeeper { writer, keeper } => write!(
                f,
                "{writer} wrote a move of {app}, which {keeper} keeps: only the computer that keeps an app moves it"
            ),
            Refusal::NotYours { writer, keeper } => {
                write!(
                    f,
                    "{writer} offered {app}, which {keeper}’s computer keeps: only {keeper} offers it"
                )
            }
            Refusal::KeptThere { computer } => write!(f, "{act} offers {app} to {computer}, which keeps it"),
            Refusal::Moving { offer } => write!(f, "{app} is moving already: {offer} was said yes to"),
            Refusal::NotOffered { answers, waiting } => {
                let waiting = waiting.as_deref().unwrap_or("none");
                write!(
                    f,
                    "{act} answers {answers}, and the offer of {app} waiting is {waiting}"
                )
            }
            Refusal::NotAsked { writer, asked } => write!(
                f,
                "{writer} answered an offer of {app} to {asked}’s computer: only {asked} answers it"
            ),
            Refusal::NoYes => write!(
                f,
                "{act} moves {app} without a yes from the person of the computer it goes to"
            ),
        }
    }
}

/// What the fold says: who keeps each app, which computers can keep, and
/// every act it refused, in the record's order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kept {
    pub apps: BTreeMap<String, Keeping>,
    pub can_keep: BTreeMap<String, Computer>,
    pub refused: Vec<Refused>,
}

/// Give's acts, in the record's order, folded.
pub fn fold(ops: &[Value]) -> Kept {
    let mut kept = Kept::default();
    for op in ops {
        if let Err(refused) = step(&mut kept, op) {
            kept.refused.push(refused);
        }
    }
    kept
}

/// One act folded into `kept`, or why it counts nowhere.
fn step(kept: &mut Kept, op: &Value) -> Result<(), Refused> {
    let text = |v: &Value| v.as_str().map(String::from);
    let act = text(&op["id"]).unwrap_or_default();
    let payload = &op["payload"];
    let app = text(&payload["app"]);
    let refuse = |refusal| Refused {
        act: act.clone(),
        app: app.clone(),
        refusal,
    };
    if op["voided"] == true {
        return Err(Refused {
            app: None,
            ..refuse(Refusal::Voided)
        });
    }
    if let Some(corrects) = text(&op["corrects"]) {
        return Err(Refused {
            app: None,
            ..refuse(Refusal::Correction { corrects })
        });
    }
    let (writer, kind) = (text(&op["actor"]).unwrap_or_default(), payload["kind"].as_str());
    let (how, kind) = match kind {
        Some("ready") => {
            let computer = Computer {
                person: text(&op["person"]).unwrap_or_default(),
                person_key: text(&op["person_key"]).unwrap_or_default(),
                since: op["ts_unix"].as_i64().unwrap_or_default(),
            };
            kept.can_keep.entry(writer).or_insert(computer);
            return Ok(());
        }
        Some(kind @ ("offered" | "accepted" | "declined")) => {
            let missing = Refusal::Missing {
                field: "app",
                kind: own(kind),
            };
            let app = text(&payload["app"]).ok_or_else(|| refuse(missing))?;
            let person = (
                text(&op["person"]).unwrap_or_default(),
                text(&op["person_key"]).unwrap_or_default(),
            );
            let at = op["ts_unix"].as_i64().unwrap_or_default();
            return offer(kept, (&act, kind, at), (&app, payload), person).map_err(refuse);
        }
        Some("started") => (How::Started, "started"),
        Some("moved") => (How::Moved, "moved"),
        Some("taken") => (How::Taken, "taken"),
        other => {
            let kind = other.unwrap_or_default().to_string();
            return Err(Refused {
                app: None,
                ..refuse(Refusal::NotGives { kind })
            });
        }
    };
    let needs = |field: &'static str| text(&payload[field]).ok_or_else(|| refuse(Refusal::Missing { field, kind }));
    let app = needs("app")?;
    let card = needs("card")?;
    let over = text(&payload["over"]);
    let from = match how {
        How::Started => None,
        How::Moved | How::Taken => Some(needs("from")?),
    };
    let sum = match how {
        How::Moved => Some(needs("sum")?),
        How::Started | How::Taken => None,
    };
    let now = kept.apps.get(&app);
    if now.map(|k| &k.act) != over.as_ref() {
        let kept = now.map(|k| k.act.clone());
        return Err(refuse(Refusal::Stale { kept, over }));
    }
    let (mut to, mut agreed) = (writer.clone(), payload["agreed"] == true);
    if let (How::Moved, Some(now)) = (how, now) {
        if now.keeper != writer {
            let keeper = now.keeper.clone();
            return Err(refuse(Refusal::NotTheKeeper { writer, keeper }));
        }
        // Where it goes, and its keeper's answer to the card's asks, are the yes's.
        let yes = (now.offer.as_ref()).filter(|o| o.answer == Answer::Yes);
        let yes = yes.ok_or_else(|| refuse(Refusal::NoYes))?;
        (to, agreed) = (yes.to.clone(), yes.agreed);
    }
    let computer = (kept.can_keep.get(&to)).ok_or_else(|| refuse(Refusal::CannotKeep { computer: to.clone() }))?;
    let keeping = Keeping {
        keeper: to,
        person: computer.person.clone(),
        person_key: computer.person_key.clone(),
        act,
        how,
        from,
        sum,
        card,
        agreed,
        since: op["ts_unix"].as_i64().unwrap_or_default(),
        offer: None,
    };
    kept.apps.insert(app, keeping);
    Ok(())
}

/// An offer's kind, as a refusal names it.
fn own(kind: &str) -> &'static str {
    match kind {
        "offered" => "offered",
        "accepted" => "accepted",
        _ => "declined",
    }
}

/// An offer of `app`, or an answer to one, folded into its keeping: `act`
/// being the act's id, kind and time, and `person` its writer's name and
/// key. Who may write each is a person, not a computer: a page says "your
/// computer" by its person's key, and a write reaches the record only from
/// that person's own computer (crates/svrngs-node/src/carried.rs `acts`).
fn offer(
    kept: &mut Kept,
    (act, kind, at): (&str, &str, i64),
    (app, payload): (&str, &Value),
    (writer, writer_key): (String, String),
) -> Result<(), Refusal> {
    let text = |field: &'static str| {
        let missing = Refusal::Missing { field, kind: own(kind) };
        payload[field].as_str().map(String::from).ok_or(missing)
    };
    let Some(keeping) = kept.apps.get_mut(app) else {
        let over = text(if kind == "offered" { "over" } else { "offer" }).ok();
        return Err(Refusal::Stale { kept: None, over });
    };
    if kind == "offered" {
        let (over, to) = (text("over")?, text("to")?);
        if over != keeping.act {
            let kept = Some(keeping.act.clone());
            return Err(Refusal::Stale { kept, over: Some(over) });
        }
        if writer_key != keeping.person_key {
            let keeper = keeping.person.clone();
            return Err(Refusal::NotYours { writer, keeper });
        }
        if to == keeping.keeper {
            return Err(Refusal::KeptThere { computer: to });
        }
        if let Some(yes) = keeping.offer.as_ref().filter(|o| o.answer == Answer::Yes) {
            return Err(Refusal::Moving { offer: yes.act.clone() });
        }
        let computer = kept
            .can_keep
            .get(&to)
            .ok_or(Refusal::CannotKeep { computer: to.clone() })?;
        keeping.offer = Some(Offer {
            act: act.into(),
            to,
            person: computer.person.clone(),
            person_key: computer.person_key.clone(),
            since: at,
            answer: Answer::Waiting,
            agreed: false,
        });
        return Ok(());
    }
    let answers = text("offer")?;
    let waiting = keeping.offer.as_mut().filter(|o| o.answer == Answer::Waiting);
    let Some(offer) = waiting.filter(|o| o.act == answers) else {
        let waiting = (keeping.offer.as_ref()).filter(|o| o.answer == Answer::Waiting);
        let waiting = waiting.map(|o| o.act.clone());
        return Err(Refusal::NotOffered { answers, waiting });
    };
    if writer_key != offer.person_key {
        let asked = offer.person.clone();
        return Err(Refusal::NotAsked { writer, asked });
    }
    (offer.answer, offer.agreed) = match kind {
        "accepted" => (Answer::Yes, payload["agreed"] == true),
        _ => (Answer::No, false),
    };
    Ok(())
}

impl Kept {
    /// The fold as JSON, the shape cases.json holds and the page reads.
    pub fn to_json(&self) -> Value {
        let apps: serde_json::Map<_, _> = (self.apps.iter())
            .map(|(app, k)| {
                let k = json!({
                    "keeper": k.keeper, "person": k.person, "person_key": k.person_key,
                    "act": k.act, "how": k.how.as_str(), "from": k.from, "card": k.card,
                    "agreed": k.agreed, "since": k.since, "sum": k.sum,
                    "offer": k.offer.as_ref().map(|o| json!({
                        "act": o.act, "to": o.to, "person": o.person, "person_key": o.person_key,
                        "since": o.since, "answer": o.answer.as_str(), "agreed": o.agreed,
                    })),
                });
                (app.clone(), k)
            })
            .collect();
        let can_keep: serde_json::Map<_, _> = (self.can_keep.iter())
            .map(|(key, c)| {
                let c = json!({ "person": c.person, "person_key": c.person_key, "since": c.since });
                (key.clone(), c)
            })
            .collect();
        let refused: Vec<_> = (self.refused.iter())
            .map(|r| json!({ "act": r.act, "app": r.app, "refusal": r.code(), "why": r.to_string() }))
            .collect();
        json!({ "apps": apps, "can_keep": can_keep, "refused": refused })
    }
}

/// The JSON boundary both builds answer: `{"ops": [...]}` in, `{"ok":
/// kept}` or `{"error": why}` out.
pub fn answer(request: &str) -> String {
    let asked: Option<Value> = serde_json::from_str(request).ok();
    let answer = match asked.as_ref().and_then(|a| a["ops"].as_array()) {
        Some(ops) => json!({ "ok": fold(ops).to_json() }),
        None => json!({ "error": "a request is {\"ops\": [...]}, the acts in the record's order" }),
    };
    answer.to_string()
}

/// The page's way in, for wasm32 alone: every value an i32, so wasm2js has
/// no i64 to split at the edge, as apps/canon/build/src/lib.rs does it.
/// `give_input(len)` gives where to write a request's UTF-8 bytes,
/// `give_call()` answers it and returns the answer's length, and
/// `give_output()` gives where the answer is (vendor/call.mjs).
#[cfg(target_arch = "wasm32")]
mod page {
    use std::cell::RefCell;

    thread_local! {
        static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn give_input(len: usize) -> *mut u8 {
        INPUT.with_borrow_mut(|b| {
            *b = vec![0; len];
            b.as_mut_ptr()
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn give_call() -> usize {
        let out = INPUT.with_borrow(|b| super::answer(&String::from_utf8_lossy(b)));
        OUTPUT.with_borrow_mut(|o| {
            *o = out.into_bytes();
            o.len()
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn give_output() -> *const u8 {
        OUTPUT.with_borrow(|o| o.as_ptr())
    }
}
