// SPDX-License-Identifier: AGPL-3.0-or-later
//! The front channel: the path on a lend's own origin where the app's
//! server sends the person to sign in (SIGN_IN_WITH_SVRNGS §5 "The front
//! channel").
//!
//! **The door answers it**, after the asker is known and before the lend's
//! path refusals and the limit, and never carries it to the service. It
//! checks the stock ask — `response_type=code`, the declaration's client,
//! a redirect on the app's own address (`http://localhost:<port>`, or the
//! door's other own hosts) or on the lend's own origin (the browser's
//! `scheme://host`, which the door tells the app as
//! `X-Forwarded-Host`/`-Proto`), read as a [`LendPath`] so none a browser
//! would read as another host's passes, `openid` in the scope, and PKCE
//! S256 when a challenge is sent — refusing each in a person's words and at
//! debug. It mints a one-time code for the person [`crate::door`] resolved
//! (a guest as a guest), bound to the client, the redirect, the challenge
//! and the nonce, and answers a relative `Location` back to the app's
//! callback, so each viewer's browser resolves it on its own address for
//! the lend.

use std::collections::BTreeMap;
use std::num::NonZeroU16;

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use svrngs_core::token::Subject;

use super::provider::S256Challenge;
use super::{Code, Provider};
use crate::door::{LendPath, LendRefusal};
use crate::lend::{Lend, SignInWithSvrngs};

/// Answer the person's browser at the authorize path, or refuse the ask;
/// the code is minted for `who`, the door's asker, never the ask.
pub(crate) fn authorize(
    provider: &Provider,
    lend: &Lend,
    client: &SignInWithSvrngs,
    port: NonZeroU16,
    browser: Option<&str>,
    who: Subject,
    query: Option<&str>,
) -> Response {
    let asked = match asked(query) {
        Ok(asked) => asked,
        Err(why) => return refused(lend, &why),
    };
    let asked = |name: &str| asked.get(name).map(String::as_str);
    if asked("response_type") != Some("code") {
        return refused(lend, "response_type is not code");
    }
    let Some(client_id) = asked("client_id") else {
        return refused(lend, "no client_id was sent");
    };
    if client_id != client.client_id {
        return refused(lend, "the client_id is not this lend's");
    }
    let Some(redirect) = asked("redirect_uri") else {
        return refused(lend, "no redirect_uri was sent");
    };
    let back = match LendPath::parse(redirect, port, browser) {
        Ok(back) => back,
        Err(why) => return refused(lend, &format!("the redirect_uri {why}")),
    };
    if !asked("scope").is_some_and(|s| s.split_whitespace().any(|part| part == "openid")) {
        return refused(lend, "the scope does not ask for openid");
    }
    let challenge =
        match S256Challenge::parse(asked("code_challenge"), asked("code_challenge_method")) {
            Ok(challenge) => challenge,
            Err(why) => return refused(lend, why),
        };
    let (sub, pkce) = (who.sub.clone(), challenge.is_some());
    let code = provider.mint(Code {
        lend: lend.name.clone(),
        client: client.client_id.clone(),
        who,
        redirect: redirect.into(),
        challenge,
        nonce: asked("nonce").map(Into::into),
    });
    let query = [("code", code.as_str())].into_iter();
    let location = back.location(query.chain(asked("state").map(|state| ("state", state))));
    tracing::debug!(target: "oidc", lend = %lend.name, client = %client.client_id, sub = %sub,
        pkce, "oidc: the authorize path answered the app's ask with a one-time code, relative to the lend's own origin");
    (StatusCode::FOUND, [(header::LOCATION, location)], ()).into_response()
}

/// The query's parameters, refusing a second one of a name.
fn asked(query: Option<&str>) -> Result<BTreeMap<String, String>, String> {
    let mut asked = BTreeMap::new();
    for (name, value) in form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
        if asked.contains_key(name.as_ref()) {
            return Err(format!("{name} is asked twice"));
        }
        asked.insert(name.into_owned(), value.into_owned());
    }
    Ok(asked)
}

/// The ask refused, in a person's words, with the fact that refused it.
fn refused(lend: &Lend, why: &str) -> Response {
    tracing::debug!(target: "oidc", lend = %lend.name, why, "oidc: refused — the app's ask at the authorize path");
    LendRefusal::Closed.say(
        StatusCode::BAD_REQUEST,
        format!(
            "{} asked to sign you in in a way svrngs does not answer: {why}",
            lend.name
        ),
    )
}
