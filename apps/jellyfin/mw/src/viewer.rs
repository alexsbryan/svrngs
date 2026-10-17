// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai cmnwlth/crates/sovereign-cli-mesh/src/mesh_media/viewer.rs@d4356181c
//! The read-only account each person reaches this library as: one per
//! person the door names, where upstream made one for every member
//! (demo/runs/mw/2026-10-08/README.md §1; u5-3 finding 4, its first piece).
//!
//! # The defect this closes
//!
//! Until this landed, a holder declared an `/Auth/Keys` API key — and that key
//! is ADMIN-EQUIVALENT: measured 2026-09-19 against a live Jellyfin 12.0.0, a
//! key minted that way has a zero `UserId`, passes elevation, and answers
//! `GET /Auth/Keys` with 200. So every housemate's request arrived at the
//! origin holding the run of the place: delete a library, add a collection,
//! mint more keys. `media_allow` decided WHO reached the origin and nothing
//! decided WHAT they could do once there.
//!
//! An API key cannot be minted FOR a user, so the read-only credential has to
//! be a user's own `AccessToken`. [`provision`] FINDS that user or creates it,
//! gives it a policy that says no to everything a viewer has no business
//! doing, and READS THE POLICY BACK; [`sign_in`] then gets the token. A
//! policy that does not read back as asked is a refusal, not a warning: an
//! unchecked write here is a library handed over on the strength of a 200.
//!
//! Two shapes of the origin's API decide how this is written, and both were
//! learned by being refused (2026-09-19, run under `RING_ROOM_TOPOLOGY=room`):
//! `POST /Users/{id}/Policy` REPLACES a policy rather than patching it, so
//! what is sent must be the origin's own document with this verb's keys
//! written over it ([`merge_read_only`]); and a user's name is fixed — the
//! door's `x-svrngs-user` ([`crate::person::Who`]), unique by the person's
//! key — so `POST /Users/New` collides on the second provision and the
//! account must be looked up first ([`viewer_id_in`]).
//!
//! # What the person types
//!
//! Nothing. The elevated credential is the API key the lender gave the
//! middleware once, as it started; it stays in this process, for the next
//! person, and is never sent anywhere but to Jellyfin. Upstream spent it once
//! and declared the viewer's token in its place, so it had to prove a viewer
//! already made by that token (`already_provisioned`, upstream `:216-296`);
//! here the key is never replaced, and that path is gone.
//!
//! The password is invented here and never written to disk, but it is KEPT
//! in memory beside the account (upstream used it once and forgot it): a
//! person signs in once per device (Jellyfin holds a session to the device a
//! token was signed in under, whatever device later asks with it), and
//! resetting the password to sign a second device in revokes the first's
//! token (both measured against Jellyfin 12.0.0 on this host, 2026-10-08,
//! `demo/runs/mw/2026-10-08/mw-1/README.md`).
//!
//! # What the person sees
//!
//! Every library, unless the lender keeps some back (`--keep-back`,
//! main.rs; the mw README §7, item 5): then every one but those, by the
//! ids Jellyfin's own list of libraries gives, written into the policy
//! with the rest and read back as written ([`folders`], [`folders_are`]).

use std::net::SocketAddr;
use std::time::Duration;

/// A person's account, made or found, and what signs a device in to it.
pub(crate) struct Viewer {
    /// The account's id in the origin's own user space.
    pub(crate) id: String,
    /// The name it was made under: the door's `x-svrngs-user`.
    pub(crate) name: String,
    /// The password this process invented: kept in memory only, so each
    /// device the person comes on can be signed in without a reset.
    pub(crate) password: String,
}

/// Every policy key a viewer must not have. Read back after the write and
/// compared one by one, so a key the origin silently ignored is caught here
/// rather than by a housemate deleting a film.
const MUST_BE_FALSE: [&str; 5] = [
    "IsAdministrator",
    "EnableContentDeletion",
    "EnableCollectionManagement",
    "EnableSubtitleManagement",
    "EnableLyricManagement",
];

/// The account's id in a `GET /Users` listing, if this origin already holds
/// it. Matched on `Name` because that is what `POST /Users/New` collides on.
fn viewer_id_in(users: &serde_json::Value, name: &str) -> Option<String> {
    users.as_array()?.iter().find_map(|u| {
        (u.get("Name").and_then(serde_json::Value::as_str)? == name)
            .then(|| u.get("Id").and_then(serde_json::Value::as_str))
            .flatten()
            .map(str::to_string)
    })
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("could not build an HTTP client: {e}"))
}

/// One request to the origin carrying `credential`'s headers.
async fn call(
    client: &reqwest::Client,
    credential: &[(String, String)],
    method: reqwest::Method,
    origin: SocketAddr,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<String, String> {
    let mut req = client.request(method.clone(), format!("http://{origin}{path}"));
    for (name, value) in credential {
        req = req.header(name, value);
    }
    if let Some(body) = body {
        req = req.json(&body);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("{method} {path} could not be sent: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("{method} {path} answered {status}: {text}"));
    }
    Ok(text)
}

/// Every key this verb decides, and what it decides them to. Applied ON TOP
/// of the policy the origin already holds, never sent alone — see
/// [`merge_read_only`].
fn read_only_policy() -> serde_json::Value {
    let mut policy = serde_json::Map::new();
    for key in MUST_BE_FALSE {
        policy.insert(key.to_string(), serde_json::Value::Bool(false));
    }
    policy.insert("EnableLiveTvManagement".to_string(), serde_json::Value::Bool(false));
    policy.insert("EnableMediaPlayback".to_string(), serde_json::Value::Bool(true));
    serde_json::Value::Object(policy)
}

/// Which libraries a person's account may open: every one, or every one
/// but those the lender keeps back. `media` is `GET /Library/MediaFolders`'s
/// answer, read at each provision, so a library made since is open to the
/// next person made. A name kept back that Jellyfin has no library by is
/// refused, never passed over: a lender who misspelled it, or renamed the
/// library since, would otherwise show it to everyone.
pub(crate) fn folders(media: &serde_json::Value, keep_back: &[String]) -> Result<serde_json::Value, String> {
    let libraries: Vec<(&str, &str)> = media
        .get("Items")
        .and_then(serde_json::Value::as_array)
        .ok_or("Jellyfin's list of libraries names none")?
        .iter()
        .filter_map(|l| Some((l.get("Name")?.as_str()?, l.get("Id")?.as_str()?)))
        .collect();
    if let Some(name) = keep_back.iter().find(|k| !libraries.iter().any(|(n, _)| n == k)) {
        let names: Vec<&str> = libraries.iter().map(|(n, _)| *n).collect();
        tracing::debug!(kept_back = %name, libraries = ?names, "mw: folders — a library kept back is not Jellyfin's, refused");
        return Err(format!(
            "Jellyfin has no library called \"{name}\" to keep back; its libraries are: {}",
            names.join(", ")
        ));
    }
    if keep_back.is_empty() {
        tracing::debug!("mw: folders — no library kept back, every one open");
        return Ok(serde_json::json!({ "EnableAllFolders": true, "EnabledFolders": [] }));
    }
    let open: Vec<&str> = libraries
        .iter()
        .filter(|(n, _)| !keep_back.iter().any(|k| k == n))
        .map(|(_, id)| *id)
        .collect();
    tracing::debug!(kept_back = ?keep_back, open = ?open, "mw: folders — every library open but those kept back");
    Ok(serde_json::json!({ "EnableAllFolders": false, "EnabledFolders": open }))
}

/// Check the libraries a policy the origin handed back opens are the ones
/// [`folders`] wrote: `EnableAllFolders` as written, and when false, the
/// same ids, whatever their order or dashes.
fn folders_are(user: &serde_json::Value, written: &serde_json::Value) -> Result<(), String> {
    let policy = user
        .get("Policy")
        .ok_or_else(|| "the origin returned a user with no Policy to check".to_string())?;
    let all = policy.get("EnableAllFolders").and_then(serde_json::Value::as_bool);
    if all != written["EnableAllFolders"].as_bool() {
        return Err(format!(
            "the origin's policy has EnableAllFolders = {all:?}, where {} was written",
            written["EnableAllFolders"]
        ));
    }
    let ids = |v: &serde_json::Value| -> Option<std::collections::BTreeSet<String>> {
        v.as_array()?
            .iter()
            .map(|i| i.as_str().map(|s| s.replace('-', "").to_lowercase()))
            .collect()
    };
    if all == Some(false) && ids(&policy["EnabledFolders"]) != ids(&written["EnabledFolders"]) {
        return Err(format!(
            "the origin's policy opens the libraries {}, where {} was written",
            policy["EnabledFolders"], written["EnabledFolders"]
        ));
    }
    Ok(())
}

/// Fields `UserPolicy` declares `required` with `minLength: 1`, so a partial
/// document is refused whole. Read from the running origin's own OpenAPI
/// (`target/ralph/jf-inventory/openapi.json`, Jellyfin 12.0.0:
/// `components.schemas.UserPolicy.required`), not from memory.
///
/// They are provider NAMES — which authentication and password-reset plugin
/// owns this account. This verb has no business choosing them, so it carries
/// the origin's own values forward and refuses if the origin did not state
/// them: inventing one would silently re-home the account on a provider
/// nobody asked for.
const PROVIDER_IDS: [&str; 2] = ["AuthenticationProviderId", "PasswordResetProviderId"];

/// The document `POST /Users/{id}/Policy` actually takes: the policy the
/// origin handed back, with [`read_only_policy`]'s keys and the libraries
/// [`folders`] opens written over it.
///
/// `POST .../Policy` REPLACES a user's whole policy — it is not a patch. Sent
/// the bare read-only keys, Jellyfin 12 answers 400 naming the two required
/// provider ids and writes nothing, which is what shipped until now: the
/// account stayed at its creation defaults and `offer` declared no viewer at
/// all (measured 2026-09-19, `target/ring-room-rr2-demo/room-holder-setup-little.out:10`).
fn merge_read_only(base: &serde_json::Value, folders: &serde_json::Value) -> Result<serde_json::Value, String> {
    let base = base
        .get("Policy")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "the origin returned a user with no Policy to build on".to_string())?;
    for key in PROVIDER_IDS {
        match base.get(key).and_then(serde_json::Value::as_str) {
            Some(v) if !v.is_empty() => {}
            _ => {
                return Err(format!(
                    "the origin's policy does not state {key}, which it requires on the way \
                     back in — this verb will not invent one"
                ))
            }
        }
    }
    let mut merged = base.clone();
    for (key, value) in read_only_policy()
        .as_object()
        .expect("read_only_policy builds an object")
        .iter()
        .chain(folders.as_object().into_iter().flatten())
    {
        merged.insert(key.clone(), value.clone());
    }
    Ok(serde_json::Value::Object(merged))
}

/// Check a policy document the origin handed back. Every key in
/// [`MUST_BE_FALSE`] must be present AND false: a key the origin did not
/// echo is "did not answer", which is not "answered: no".
fn policy_is_read_only(user: &serde_json::Value) -> Result<(), String> {
    let policy = user
        .get("Policy")
        .ok_or_else(|| "the origin returned a user with no Policy to check".to_string())?;
    for key in MUST_BE_FALSE {
        match policy.get(key).and_then(serde_json::Value::as_bool) {
            Some(false) => {}
            Some(true) => return Err(format!("the origin kept {key} = true on the viewer")),
            None => {
                return Err(format!(
                    "the origin's policy does not name {key} — it cannot be read as false"
                ))
            }
        }
    }
    Ok(())
}

/// Jellyfin's list of its libraries, `GET /Library/MediaFolders`, asked
/// with the lender's key: what [`folders`] reads.
pub(crate) async fn libraries(origin: SocketAddr, elevated: &[(String, String)]) -> Result<serde_json::Value, String> {
    let media = call(
        &client()?,
        elevated,
        reqwest::Method::GET,
        origin,
        "/Library/MediaFolders",
        None,
    )
    .await?;
    serde_json::from_str(&media).map_err(|e| format!("the origin's list of libraries is not JSON ({e})"))
}

/// Create (or find) the read-only account `name` on `origin`, opening every
/// library but those in `keep_back`, and prove its policy.
///
/// `elevated` is the key the lender gave the middleware — the only thing
/// here that can create a user. It is used on Jellyfin's API alone.
pub(crate) async fn provision(
    origin: SocketAddr,
    elevated: &[(String, String)],
    name: &str,
    keep_back: &[String],
) -> Result<Viewer, String> {
    if elevated.is_empty() {
        return Err("no key was given to the middleware, so no account can be created — \
             start it with the key from Jellyfin's Dashboard, API Keys"
            .to_string());
    }
    let client = client()?;
    // A password this process invents and keeps in memory, never on disk: a
    // password on disk would be a second way in that nobody asked for.
    let password: String = uuid::Uuid::new_v4().simple().to_string();

    // FIND, then create. The name is fixed per person so a second provision
    // reuses the one account -- but `POST /Users/New` answers 400 on a name
    // that already exists, so an unconditional create makes every provision
    // after the first fail (measured 2026-09-19,
    // `target/ring-room-rr2-demo/room-offer.out:4`).
    let existing = call(&client, elevated, reqwest::Method::GET, origin, "/Users", None).await?;
    let existing: serde_json::Value =
        serde_json::from_str(&existing).map_err(|e| format!("the origin's user list is not JSON ({e})"))?;
    let found = viewer_id_in(&existing, name);

    let id = match &found {
        Some(id) => {
            tracing::debug!(
                viewer_user = %id,
                name,
                "mw: provision — the person's account found, reusing it"
            );
            id.clone()
        }
        None => {
            let created = call(
                &client,
                elevated,
                reqwest::Method::POST,
                origin,
                "/Users/New",
                Some(serde_json::json!({ "Name": name, "Password": password })),
            )
            .await?;
            let created: serde_json::Value = serde_json::from_str(&created)
                .map_err(|e| format!("the origin's new-user answer is not JSON ({e}): {created}"))?;
            let id = created
                .get("Id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("the origin created a user with no Id: {created}"))?
                .to_string();
            tracing::debug!(
                viewer_user = %id,
                name,
                "mw: provision — no account for this person on this origin, created one"
            );
            id
        }
    };

    if found.is_some() {
        tracing::debug!(
            viewer_user = %id,
            "mw: provision — resetting the reused account's password: an earlier run of this \
             process made it, and its password went with that run"
        );
        // The elevated credential sets the one invented above so `sign_in`
        // has something to spend. It revokes the account's other tokens
        // (measured, module docs), which only that earlier run held.
        call(
            &client,
            elevated,
            reqwest::Method::POST,
            origin,
            &format!("/Users/Password?userId={id}"),
            Some(serde_json::json!({ "NewPw": password, "ResetPassword": false })),
        )
        .await?;
    }

    // The policy to send is the origin's own, with this verb's keys written
    // over it -- `POST .../Policy` replaces rather than patches. See
    // [`merge_read_only`].
    let before = call(
        &client,
        elevated,
        reqwest::Method::GET,
        origin,
        &format!("/Users/{id}"),
        None,
    )
    .await?;
    let before: serde_json::Value =
        serde_json::from_str(&before).map_err(|e| format!("the origin's user document is not JSON ({e}): {before}"))?;
    let open = folders(&libraries(origin, elevated).await?, keep_back)?;

    call(
        &client,
        elevated,
        reqwest::Method::POST,
        origin,
        &format!("/Users/{id}/Policy"),
        Some(merge_read_only(&before, &open)?),
    )
    .await?;

    // READ IT BACK. The write answering 204 says the request was accepted,
    // not that the policy is what was asked for.
    let back = call(
        &client,
        elevated,
        reqwest::Method::GET,
        origin,
        &format!("/Users/{id}"),
        None,
    )
    .await?;
    let back: serde_json::Value =
        serde_json::from_str(&back).map_err(|e| format!("the origin's user document is not JSON ({e}): {back}"))?;
    policy_is_read_only(&back)?;
    folders_are(&back, &open)?;

    tracing::info!(
        %origin,
        viewer_user = %id,
        name,
        "mw: provision — the person's read-only account made and its policy read back"
    );
    Ok(Viewer {
        id,
        name: name.to_string(),
        password,
    })
}

/// Sign `viewer` in on the device `device` names — a whole `authorization`
/// value with no token, `MediaBrowser Client="…", Device="…", DeviceId="…",
/// Version="…"` — and return the token Jellyfin holds to that device. No
/// elevation: `AuthenticateByName` is anyone's.
pub(crate) async fn sign_in(origin: SocketAddr, viewer: &Viewer, device: &str) -> Result<String, String> {
    let client = client()?;
    let authed = call(
        &client,
        &[("authorization".to_string(), device.to_string())],
        reqwest::Method::POST,
        origin,
        "/Users/AuthenticateByName",
        Some(serde_json::json!({ "Username": viewer.name, "Pw": viewer.password })),
    )
    .await?;
    let authed: serde_json::Value =
        serde_json::from_str(&authed).map_err(|e| format!("the origin's authenticate answer is not JSON ({e})"))?;
    let token = authed
        .get("AccessToken")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "the origin authenticated the viewer without an AccessToken".to_string())?;
    tracing::debug!(%origin, viewer_user = %viewer.id, "mw: sign in — a device signed in for the person");
    Ok(token.to_string())
}

// Upstream's stand-in origin records what it was asked behind a mutex, a
// test fixture's; the middleware itself shares nothing mutable (people.rs).
#[cfg(test)]
#[allow(clippy::disallowed_types)]
mod tests {
    use super::*;

    /// A stand-in origin that routes by path and records what was asked of
    /// it. A raw listener rather than a test-server dependency: the contract
    /// under test is which requests this verb makes, which is exactly what
    /// the recording answers.
    struct Origin {
        addr: SocketAddr,
        asked: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    /// `routes` maps an EXACT path to the document it answers with. Anything
    /// else answers `403 "Invalid user or password entered."` — which is what
    /// the live origin answers a non-admin caller, so a verb that reaches for
    /// something a viewer's own token cannot have fails here exactly as it
    /// failed there. Exact, not prefix: `/Users` as a prefix would quietly
    /// serve `/Users/Password` too, and the mint path would walk right past
    /// the request that is the whole defect.
    async fn origin(routes: Vec<(&'static str, serde_json::Value)>) -> Origin {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let asked = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorder = asked.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    return;
                };
                let routes = routes.clone();
                let recorder = recorder.clone();
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 8192];
                    let n = sock.read(&mut buf).await.unwrap_or(0);
                    let head = String::from_utf8_lossy(&buf[..n]).to_string();
                    let line = head.lines().next().unwrap_or_default().to_string();
                    recorder.lock().unwrap().push(line.clone());
                    let path = line.split_whitespace().nth(1).unwrap_or("").to_string();
                    let (status, body) = match routes.iter().find(|(p, _)| &path == p) {
                        Some((_, body)) => ("200 OK", body.to_string()),
                        None => ("403 Forbidden", "\"Invalid user or password entered.\"".to_string()),
                    };
                    let resp = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: \
                         {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = sock.write_all(resp.as_bytes()).await;
                    let _ = sock.shutdown().await;
                });
            }
        });
        Origin { addr, asked }
    }

    fn declared(value: &str) -> Vec<(String, String)> {
        vec![("authorization".to_string(), value.to_string())]
    }

    /// A user document as the origin hands one back after the write: the
    /// provider ids it requires, and every key this verb decides as decided.
    fn read_back(id: &str) -> serde_json::Value {
        let mut policy = origin_policy()["Policy"].clone();
        for (key, value) in read_only_policy().as_object().unwrap() {
            policy[key] = value.clone();
        }
        policy["EnableAllFolders"] = serde_json::Value::Bool(true);
        serde_json::json!({ "Id": id, "Policy": policy })
    }

    /// Jellyfin's list of libraries, as `GET /Library/MediaFolders` answers:
    /// Sam's films, and home videos Sam may keep back.
    fn media() -> serde_json::Value {
        serde_json::json!({ "Items": [
            { "Name": "Movies", "Id": "f137a2dd21bbc1b99aa5c0f6bf02a805" },
            { "Name": "Home Videos", "Id": "0c8a2c2a6b9f4e0e8f1d3c5b7a9e1d2f" },
        ], "TotalRecordCount": 2 })
    }

    /// A person new to the origin is made an account under their own name,
    /// its policy written and read back, and no password reset: there is no
    /// earlier token to revoke.
    #[tokio::test]
    async fn a_new_person_is_made_an_account_of_their_own() {
        let o = origin(vec![
            ("/Users", serde_json::json!([{ "Name": "sam", "Id": "s" }])),
            ("/Users/New", serde_json::json!({ "Id": "u1" })),
            ("/Users/u1", read_back("u1")),
            ("/Users/u1/Policy", serde_json::json!({})),
            ("/Library/MediaFolders", media()),
        ])
        .await;
        let viewer = provision(o.addr, &declared("key"), "Ana-0123456789abcdef", &[])
            .await
            .expect("a new person provisions");
        assert_eq!(viewer.id, "u1");
        assert_eq!(viewer.name, "Ana-0123456789abcdef");
        let asked = o.asked.lock().unwrap().clone();
        assert!(asked.iter().any(|l| l.starts_with("POST /Users/New")), "{asked:?}");
        assert!(!asked.iter().any(|l| l.contains("/Users/Password")), "{asked:?}");
    }

    /// An account an earlier run made is found by its name, not made again,
    /// and its password is reset, since that run's went with it.
    #[tokio::test]
    async fn a_person_an_earlier_run_made_is_found_and_their_password_reset() {
        let o = origin(vec![
            (
                "/Users",
                serde_json::json!([{ "Name": "Ana-0123456789abcdef", "Id": "u1" }]),
            ),
            ("/Users/Password?userId=u1", serde_json::json!({})),
            ("/Users/u1", read_back("u1")),
            ("/Users/u1/Policy", serde_json::json!({})),
            ("/Library/MediaFolders", media()),
        ])
        .await;
        let viewer = provision(o.addr, &declared("key"), "Ana-0123456789abcdef", &[])
            .await
            .expect("a found person provisions");
        assert_eq!(viewer.id, "u1");
        let asked = o.asked.lock().unwrap().clone();
        assert!(!asked.iter().any(|l| l.contains("/Users/New")), "{asked:?}");
        assert!(
            asked.iter().any(|l| l.contains("/Users/Password?userId=u1")),
            "{asked:?}"
        );
    }

    /// A library the lender keeps back is in no account's policy: every
    /// library is no longer open, and the others are, by Jellyfin's ids.
    #[test]
    fn a_library_kept_back_is_left_out_of_the_account() {
        let open = folders(&media(), &["Home Videos".to_string()]).expect("a library Jellyfin has");
        assert_eq!(open["EnableAllFolders"], serde_json::json!(false));
        assert_eq!(
            open["EnabledFolders"],
            serde_json::json!(["f137a2dd21bbc1b99aa5c0f6bf02a805"])
        );
        let merged = merge_read_only(&origin_policy(), &open).expect("a stated policy merges");
        assert_eq!(merged["EnableAllFolders"], serde_json::json!(false));
        assert_eq!(merged["EnabledFolders"], open["EnabledFolders"]);
    }

    /// None kept back, every library is open, as before `--keep-back`.
    #[test]
    fn no_library_kept_back_opens_every_one() {
        let open = folders(&media(), &[]).expect("nothing to refuse");
        assert_eq!(open["EnableAllFolders"], serde_json::json!(true));
    }

    /// Negative: a name Jellyfin has no library by is refused by name, with
    /// the ones it has, never passed over to open everything.
    #[test]
    fn a_library_kept_back_that_jellyfin_has_not_is_refused() {
        let got = folders(&media(), &["Home Video".to_string()]).unwrap_err();
        assert!(
            got.contains("\"Home Video\"") && got.contains("Movies, Home Videos"),
            "{got}"
        );
    }

    /// Negative: an origin whose policy, read back, still opens the library
    /// kept back is refused, whether it kept every library open or listed it.
    #[test]
    fn an_origin_that_opens_a_library_kept_back_is_refused() {
        let open = folders(&media(), &["Home Videos".to_string()]).unwrap();
        let all = serde_json::json!({ "Policy": { "EnableAllFolders": true, "EnabledFolders": [] } });
        assert!(folders_are(&all, &open).unwrap_err().contains("EnableAllFolders"));
        let listed = serde_json::json!({ "Policy": { "EnableAllFolders": false,
            "EnabledFolders": ["f137a2dd21bbc1b99aa5c0f6bf02a805", "0c8a2c2a6b9f4e0e8f1d3c5b7a9e1d2f"] } });
        assert!(folders_are(&listed, &open).unwrap_err().contains("0c8a2c2a"));
        let dashed = serde_json::json!({ "Policy": { "EnableAllFolders": false,
            "EnabledFolders": ["F137A2DD-21BB-C1B9-9AA5-C0F6BF02A805"] } });
        assert_eq!(folders_are(&dashed, &open), Ok(()));
    }

    /// Kept back, a person's account whose policy the origin left opening
    /// every library is no account to answer with: provision refuses it.
    #[tokio::test]
    async fn a_person_whose_account_still_opens_every_library_is_refused() {
        let o = origin(vec![
            ("/Users", serde_json::json!([])),
            ("/Users/New", serde_json::json!({ "Id": "u1" })),
            ("/Users/u1", read_back("u1")),
            ("/Users/u1/Policy", serde_json::json!({})),
            ("/Library/MediaFolders", media()),
        ])
        .await;
        let got = provision(
            o.addr,
            &declared("key"),
            "Ana-0123456789abcdef",
            &["Home Videos".to_string()],
        )
        .await
        .err()
        .expect("an account opening what is kept back is refused");
        assert!(got.contains("EnableAllFolders"), "{got}");
    }

    /// Positive: the policy this verb writes says false to every key the
    /// read-back checks, so the two cannot drift apart.
    #[test]
    fn the_policy_written_satisfies_the_policy_checked() {
        let user = serde_json::json!({ "Policy": read_only_policy() });
        assert_eq!(policy_is_read_only(&user), Ok(()));
    }

    /// Negative: an origin that kept administrator rights is refused by name.
    #[test]
    fn an_administrator_viewer_is_refused() {
        let mut policy = read_only_policy();
        policy["IsAdministrator"] = serde_json::Value::Bool(true);
        let got = policy_is_read_only(&serde_json::json!({ "Policy": policy }));
        assert!(
            got.unwrap_err().contains("IsAdministrator"),
            "the refusal must name the key that stayed true"
        );
    }

    /// Negative: a key the origin did not echo is NOT read as false —
    /// "did not answer" is not "answered: no" (ARCH principle 6).
    #[test]
    fn a_policy_key_the_origin_omitted_is_not_read_as_false() {
        let mut policy = read_only_policy();
        policy.as_object_mut().unwrap().remove("EnableContentDeletion");
        let got = policy_is_read_only(&serde_json::json!({ "Policy": policy }));
        assert!(
            got.unwrap_err().contains("EnableContentDeletion"),
            "an omitted key must be refused by name"
        );
    }

    /// Negative: a user document with no policy at all is a refusal, not a
    /// pass.
    #[test]
    fn a_user_with_no_policy_is_refused() {
        let got = policy_is_read_only(&serde_json::json!({ "Id": "x" }));
        assert!(got.is_err());
    }

    /// A policy document as Jellyfin 12 actually hands one back: the keys
    /// this verb decides, plus the two it requires and this verb does not own.
    fn origin_policy() -> serde_json::Value {
        serde_json::json!({ "Policy": {
            "AuthenticationProviderId":
                "Jellyfin.Server.Implementations.Users.DefaultAuthenticationProvider",
            "PasswordResetProviderId":
                "Jellyfin.Server.Implementations.Users.DefaultPasswordResetProvider",
            "IsAdministrator": true,
            "EnableContentDeletion": true,
            "EnableCollectionManagement": true,
            "EnableSubtitleManagement": true,
            "EnableLyricManagement": true,
            "EnableMediaPlayback": false,
            "EnableLiveTvManagement": true,
            "MaxActiveSessions": 3
        }})
    }

    /// The document sent to `POST /Users/{id}/Policy` carries the origin's own
    /// provider ids forward. Sent without them, Jellyfin 12 answers 400 and
    /// writes NOTHING -- the account keeps whatever rights it had, and the
    /// offer publishes no viewer. That is the defect measured on
    /// 2026-09-19 (`room-holder-setup-little.out:10`), and this is the
    /// assertion that goes red if the partial document comes back.
    #[test]
    fn the_merged_policy_carries_the_origins_required_provider_ids() {
        let merged = merge_read_only(&origin_policy(), &serde_json::json!({})).expect("a stated policy merges");
        for key in PROVIDER_IDS {
            assert_eq!(
                merged.get(key).and_then(serde_json::Value::as_str),
                origin_policy()["Policy"][key].as_str(),
                "{key} must reach the origin unchanged, or the write is refused whole"
            );
        }
    }

    /// The merge OVERRULES an administrator: `POST .../Policy` replaces the
    /// document, so every key this verb decides must be in what it sends.
    #[test]
    fn the_merged_policy_overrules_every_right_a_viewer_must_not_have() {
        let merged = merge_read_only(&origin_policy(), &serde_json::json!({})).expect("a stated policy merges");
        policy_is_read_only(&serde_json::json!({ "Policy": merged.clone() }))
            .expect("what is sent must satisfy what is checked");
        assert_eq!(merged.get("EnableMediaPlayback"), Some(&serde_json::json!(true)));
        assert_eq!(merged.get("EnableLiveTvManagement"), Some(&serde_json::json!(false)));
        // Keys this verb does not decide survive: replacing the document must
        // not quietly drop the holder's own settings.
        assert_eq!(merged.get("MaxActiveSessions"), Some(&serde_json::json!(3)));
    }

    /// Negative: an origin that did not state a provider id is refused by
    /// name rather than sent a guess (ARCH principle 6).
    #[test]
    fn a_policy_missing_a_provider_id_is_refused_not_invented() {
        let mut doc = origin_policy();
        doc["Policy"].as_object_mut().unwrap().remove("PasswordResetProviderId");
        let got = merge_read_only(&doc, &serde_json::json!({})).unwrap_err();
        assert!(
            got.contains("PasswordResetProviderId"),
            "the refusal must name the field the origin left unstated: {got}"
        );
    }

    /// A second provision FINDS the account the first one made. Without
    /// this, `POST /Users/New` collides on the name and answers 400 -- every
    /// offer after the first fails (`room-offer.out:4`, 2026-09-19).
    #[test]
    fn a_viewer_left_by_an_earlier_offer_is_found_not_recreated() {
        let users = serde_json::json!([
            { "Name": "alex", "Id": "aaa" },
            { "Name": "Ana-0123456789abcdef", "Id": "bbb" },
        ]);
        assert_eq!(viewer_id_in(&users, "Ana-0123456789abcdef").as_deref(), Some("bbb"));
    }

    /// Negative: an origin holding no such account reports absence, so the
    /// create path runs. "Not found" is not "found something else".
    #[test]
    fn an_origin_without_the_viewer_reports_absence() {
        let users = serde_json::json!([{ "Name": "alex", "Id": "aaa" }]);
        assert_eq!(viewer_id_in(&users, "Ana-0123456789abcdef"), None);
    }
}
