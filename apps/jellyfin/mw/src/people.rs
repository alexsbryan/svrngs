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
    on: On,
    answer: oneshot::Sender<Result<Signed, String>>,
}

/// Which device a request is to be signed in on.
#[derive(Clone, Debug)]
pub(crate) enum On {
    /// The device the request names.
    Device(Device),
    /// The person's last device that named itself: a request asks to be
    /// signed in by its query's credential and names no device, as
    /// jellyfin-web 12's socket does (`/socket?ApiKey=…`, mw-2's first run:
    /// refused 403, 17 times for each person, while signed in as nobody;
    /// demo/runs/mw/2026-10-08/mw-2/README.md). One person on
    /// two browsers at once has their socket on the last one's session.
    Last,
    /// None: signed in as nobody.
    Nobody,
}

/// What a request is signed in as: the person's account, and the device
/// and the token it is signed in on, if any.
#[derive(Clone, Debug)]
pub(crate) struct Signed {
    /// The account's id, which jellyfin-web's stored sign-in names (page.rs).
    pub(crate) user: String,
    pub(crate) on: Option<(Device, String)>,
}

/// A person this process has made or found, each of their devices and the
/// token it is signed in with, by `DeviceId`, and the last that named itself.
struct Known {
    viewer: Viewer,
    devices: HashMap<String, (Device, String)>,
    last: Option<String>,
}

impl People {
    /// Start the owner. `elevated` is the lender's key, the only thing it
    /// makes accounts with; it never leaves the owner but to Jellyfin.
    pub(crate) fn start(jellyfin: SocketAddr, elevated: Vec<(String, String)>) -> People {
        let (ask, mut asked) = mpsc::channel::<Ask>(64);
        tokio::spawn(async move {
            let mut known: HashMap<PersonKey, Known> = HashMap::new();
            while let Some(a) = asked.recv().await {
                let signed = signed(&mut known, jellyfin, &elevated, &a.who, a.on).await;
                let _ = a.answer.send(signed);
            }
        });
        People { ask }
    }

    /// The account `who` is signed in as, and the device and token they are
    /// signed in on, their account made and the device signed in first if
    /// this process has not yet; on no device, their account made and no
    /// token.
    pub(crate) async fn signed(&self, who: Who, on: On) -> Result<Signed, String> {
        let (answer, answered) = oneshot::channel();
        self.ask
            .send(Ask { who, on, answer })
            .await
            .map_err(|_| "the owner of the accounts has stopped".to_string())?;
        answered
            .await
            .map_err(|_| "the owner of the accounts dropped the ask".to_string())?
    }
}

async fn signed(
    known: &mut HashMap<PersonKey, Known>,
    jellyfin: SocketAddr,
    elevated: &[(String, String)],
    who: &Who,
    on: On,
) -> Result<Signed, String> {
    if !known.contains_key(&who.key) {
        let name = viewer::user_name(&who.name, who.key.as_str());
        tracing::info!(key = %who.key.as_str(), person = %who.name, user = %name,
            "mw: a key not seen by this process — finding or making its person's account");
        let viewer = viewer::provision(jellyfin, elevated, &name).await?;
        known.insert(
            who.key.clone(),
            Known {
                viewer,
                devices: HashMap::new(),
                last: None,
            },
        );
    }
    let person = known.get_mut(&who.key).expect("inserted above when absent");
    let user = person.viewer.id.clone();
    let device = match on {
        On::Device(d) => d,
        On::Last => {
            let last = person.last.as_ref().and_then(|id| person.devices.get(id));
            tracing::debug!(key = %who.key.as_str(), device = ?last.map(|(d, _)| &d.id),
                "mw: a request naming no device asks to be signed in — on the person's last device, if any");
            return Ok(Signed {
                user,
                on: last.cloned(),
            });
        }
        On::Nobody => return Ok(Signed { user, on: None }),
    };
    person.last = Some(device.id.clone());
    if let Some(on) = person.devices.get(&device.id) {
        tracing::debug!(key = %who.key.as_str(), device = %device.id, "mw: the person's device already signed in");
        return Ok(Signed {
            user,
            on: Some(on.clone()),
        });
    }
    tracing::info!(key = %who.key.as_str(), user = %person.viewer.name, device = %device.id,
        "mw: a device of the person's not yet signed in — signing it in");
    let token = viewer::sign_in(jellyfin, &person.viewer, &device.header()).await?;
    person
        .devices
        .insert(device.id.clone(), (device.clone(), token.clone()));
    Ok(Signed {
        user,
        on: Some((device, token)),
    })
}
