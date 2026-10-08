// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one owner of every person's account and tokens: a task the
//! forwards ask by message (RUST_ARCH: ownership is a tree, no shared
//! mutable state). Asks are answered one at a time, so two first requests
//! from one person make one account, not two.

use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::sync::{mpsc, oneshot};

use crate::person::{Device, PersonKey, Who};
use crate::viewer::{self, Viewer};

/// What a forward holds to ask the owner.
#[derive(Clone)]
pub(crate) struct People {
    ask: mpsc::Sender<Ask>,
}

struct Ask {
    who: Who,
    device: Option<Device>,
    answer: oneshot::Sender<Result<Option<String>, String>>,
}

/// A person this process has made or found, and the token each of their
/// devices is signed in with, by `DeviceId`.
struct Known {
    viewer: Viewer,
    tokens: HashMap<String, String>,
}

impl People {
    /// Start the owner. `elevated` is the lender's key, the only thing it
    /// makes accounts with; it never leaves the owner but to Jellyfin.
    pub(crate) fn start(jellyfin: SocketAddr, elevated: Vec<(String, String)>) -> People {
        let (ask, mut asked) = mpsc::channel::<Ask>(64);
        tokio::spawn(async move {
            let mut known: HashMap<PersonKey, Known> = HashMap::new();
            while let Some(a) = asked.recv().await {
                let token = token(&mut known, jellyfin, &elevated, &a.who, &a.device).await;
                let _ = a.answer.send(token);
            }
        });
        People { ask }
    }

    /// The token `who` is signed in with on `device`, their account made
    /// and the device signed in first if this process has not yet; with no
    /// device, their account made and no token.
    pub(crate) async fn token(&self, who: Who, device: Option<Device>) -> Result<Option<String>, String> {
        let (answer, answered) = oneshot::channel();
        self.ask
            .send(Ask { who, device, answer })
            .await
            .map_err(|_| "the owner of the accounts has stopped".to_string())?;
        answered
            .await
            .map_err(|_| "the owner of the accounts dropped the ask".to_string())?
    }
}

async fn token(
    known: &mut HashMap<PersonKey, Known>,
    jellyfin: SocketAddr,
    elevated: &[(String, String)],
    who: &Who,
    device: &Option<Device>,
) -> Result<Option<String>, String> {
    if !known.contains_key(&who.key) {
        let name = viewer::user_name(&who.name, who.key.as_str());
        tracing::info!(key = %who.key.as_str(), person = %who.name, user = %name,
            "mw: a key not seen by this process — finding or making its person's account");
        let viewer = viewer::provision(jellyfin, elevated, &name).await?;
        known.insert(
            who.key.clone(),
            Known {
                viewer,
                tokens: HashMap::new(),
            },
        );
    }
    let person = known.get_mut(&who.key).expect("inserted above when absent");
    let Some(device) = device else {
        return Ok(None);
    };
    if let Some(token) = person.tokens.get(&device.id) {
        tracing::debug!(key = %who.key.as_str(), device = %device.id, "mw: the person's device already signed in");
        return Ok(Some(token.clone()));
    }
    tracing::info!(key = %who.key.as_str(), user = %person.viewer.name, device = %device.id,
        "mw: a device of the person's not yet signed in — signing it in");
    let token = viewer::sign_in(jellyfin, &person.viewer, &device.header()).await?;
    person.tokens.insert(device.id.clone(), token.clone());
    Ok(Some(token))
}
