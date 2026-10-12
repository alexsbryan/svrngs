// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one owner of every person's account: a task the answers ask by
//! message (RUST_ARCH: ownership is a tree, no shared mutable state). Asks
//! are answered one at a time, so two asks for one person make one
//! account, not two.
//!
//! Each ask signs the person's device in afresh. The door asks again only
//! after its own restart, or after Jellyfin answered `401` to the token it
//! kept (crates/svrngs-node/src/door/sign_in.rs, `forget`): either way the
//! token answered before is lost to it or revoked, and a sign-in on the
//! same device puts a new one in its place.

use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::sync::{mpsc, oneshot};

use crate::person::{Device, PersonKey, Who};
use crate::viewer::{self, Viewer};
use crate::Plant;

/// What an answer holds to ask the owner.
#[derive(Clone)]
pub(crate) struct People {
    ask: mpsc::Sender<Ask>,
}

struct Ask {
    who: Who,
    answer: oneshot::Sender<Result<Signed, String>>,
}

/// A person signed in: their account's id, which jellyfin-web's stored
/// sign-in names (page.rs), the device, and the token Jellyfin holds to it.
#[derive(Clone, Debug)]
pub(crate) struct Signed {
    pub(crate) user: String,
    pub(crate) device: Device,
    pub(crate) token: String,
}

impl People {
    /// Start the owner. `elevated` is the lender's key, the only thing it
    /// makes accounts with; it never leaves the owner but to Jellyfin.
    /// `keep_back` names the libraries no account it makes may open.
    pub(crate) fn start(
        jellyfin: SocketAddr,
        elevated: Vec<(String, String)>,
        keep_back: Vec<String>,
        plant: Option<Plant>,
    ) -> People {
        let (ask, mut asked) = mpsc::channel::<Ask>(64);
        tokio::spawn(async move {
            let mut known: HashMap<PersonKey, Viewer> = HashMap::new();
            // Under `one-device`, the one device's sign-in, made once and
            // answered to everyone, as the spike's plant did.
            let mut one: Option<Signed> = None;
            while let Some(a) = asked.recv().await {
                let signed = match (&one, plant) {
                    (Some(s), Some(Plant::OneDevice)) => Ok(s.clone()),
                    _ => signed(&mut known, jellyfin, &elevated, &keep_back, &a.who, plant).await,
                };
                if let (Ok(s), Some(Plant::OneDevice)) = (&signed, plant) {
                    one = Some(s.clone());
                }
                let _ = a.answer.send(signed);
            }
        });
        People { ask }
    }

    /// `who` signed in: their account made first if this process has not
    /// yet, then their own device signed in.
    pub(crate) async fn signed(&self, who: Who) -> Result<Signed, String> {
        let (answer, answered) = oneshot::channel();
        self.ask
            .send(Ask { who, answer })
            .await
            .map_err(|_| "the owner of the accounts has stopped".to_string())?;
        answered
            .await
            .map_err(|_| "the owner of the accounts dropped the ask".to_string())?
    }
}

async fn signed(
    known: &mut HashMap<PersonKey, Viewer>,
    jellyfin: SocketAddr,
    elevated: &[(String, String)],
    keep_back: &[String],
    who: &Who,
    plant: Option<Plant>,
) -> Result<Signed, String> {
    if !known.contains_key(&who.key) {
        let name = viewer::user_name(&who.name, who.key.as_str());
        tracing::info!(key = %who.key.as_str(), person = %who.name, user = %name,
            "mw: a key not seen by this process — finding or making its person's account");
        let viewer = viewer::provision(jellyfin, elevated, &name, keep_back).await?;
        known.insert(who.key.clone(), viewer);
    }
    let viewer = known.get(&who.key).expect("inserted above when absent");
    let device = match plant {
        Some(Plant::OneDevice) => Device::one(),
        _ => Device::of(who),
    };
    tracing::info!(key = %who.key.as_str(), user = %viewer.name, device = %device.id,
        "mw: signing the person's device in");
    let token = viewer::sign_in(jellyfin, viewer, &device.header()).await?;
    Ok(Signed {
        user: viewer.id.clone(),
        device,
        token,
    })
}
