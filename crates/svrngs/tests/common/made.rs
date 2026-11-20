//! `svrngs new`, `link` and `join` on a node a test started, as svrngs
//! running here does each when the command asks it (cli.rs `asked`): the
//! node's own methods its routes call, `Node::made`, `Node::linked` and
//! `Host::join`, said in the command's types. A `join` first waits for a
//! relay the link carries, as the command does before it asks (cli.rs
//! `serve`).

#![allow(dead_code)]

use svrngs::cli::{self, Printed, Refusal, Refused, Said, Unjudged};
use svrngs::group::{Name, Seed};
use svrngs_node::group::NewAsk;
use svrngs_node::home::{Linking, Unlinked};
use svrngs_node::invite::Unminted;
use svrngs_node::iroh::RelayUrl;
use svrngs_node::join::Entered;
use svrngs_node::link::{Link, Origin};
use svrngs_node::node::Node;
use svrngs_node::Person;

/// A link as the command prints it: the page is the https link's origin.
fn printed(client: String, https: Option<String>) -> Printed {
    let page = https
        .as_deref()
        .and_then(Link::parse)
        .and_then(|l| l.origin);
    Printed {
        client,
        https,
        page,
    }
}

/// `svrngs new`: `name` made with this computer as `person`, its https
/// links on `page` when given, met at `kept`, else where this computer is
/// met; and its first link.
pub async fn new_group(
    node: &Node,
    name: Name,
    person: Name,
    page: Option<&Origin>,
    kept: &[RelayUrl],
) -> Result<(Seed, Printed), Refusal> {
    let asked = NewAsk {
        name: name.as_str().into(),
        person: person.as_str().into(),
        origin: page.map(|p| p.as_str().into()),
        relays: kept.to_vec(),
    };
    let made = node.made(asked).await;
    let made = made.map_err(|_| Unjudged::Keep(name.to_string()))?;
    let seed = Seed::from_text(&made.seed).expect("the seed as seed.json holds it");
    Ok((seed, printed(made.client, made.https)))
}

/// `svrngs link`: a new link to the group `reference` names, on `page`,
/// kept from now on, when given; a device link with `device`, and whose;
/// the group met at `relays` from now on, when any.
pub async fn link_to(
    node: &Node,
    reference: &str,
    page: Option<&Origin>,
    device: bool,
    relays: &[RelayUrl],
) -> Result<(Seed, Printed, Option<Person>), Refusal> {
    let seed = cli::group_named(node.rail().root(), reference)?;
    let asked = Linking {
        group: seed.id(),
        device,
        relays: relays.to_vec(),
        origin: page.map(|p| p.as_str().into()),
        ..Linking::default()
    };
    let made = node.linked(asked).await.map_err(|e| match e {
        Unlinked::Closed(why) => Refusal::from(Refused::Meet(why)),
        Unlinked::Unminted(Unminted::NotIn) => Refused::NotIn(seed.name().clone()).into(),
        Unlinked::Unminted(_) => Unjudged::Keep(seed.name().to_string()).into(),
    })?;
    Ok((seed, printed(made.client, made.https), made.device))
}

/// `svrngs join`: `link` redeemed as `person`, once a relay it carries is
/// waited for.
pub async fn join_group(node: &Node, link: &Link, person: &Name) -> Result<Said, Refusal> {
    node.host().meet_at(&link.token.relays).await;
    node.host().online().await;
    Ok(match node.host().join(link, person).await? {
        Entered::In { seed, person } => Said::Joined { seed, person },
        Entered::AlreadyIn(seed) => Said::AlreadyIn(seed),
    })
}
