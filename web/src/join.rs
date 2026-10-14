// SPDX-License-Identifier: AGPL-3.0-or-later
//! Joining from a phone (DESIGN §3, "Joining"; §6, "A link admits each
//! person once"): what the page signs and believes, by svrngs-core's
//! bodies, the ones `svrngs join` uses (call 5). The page dials the link's
//! issuer on the redeem protocol ([`crate::connect_to_join`]), posts
//! [`ask`]'s body to `/ask`, believes the answer by [`believe`], and posts
//! [`redemption`]'s body to `/redeem`, naming itself a browser (call 2).
//! The HTTP is the page's, over `http.rs`, as every request it makes.
//!
//! Every function here is pure: the key and the link in, JSON out. Each
//! refusal is the error, in a person's words, and traced as the dial's
//! layers are.

use svrngs_core::joining::{AskRequest, Asked, RedeemRequest};
use svrngs_core::link::Token;
use svrngs_core::seed::answered;
use svrngs_core::{Device, Name, Person, SigningKey};
use wasm_bindgen::prelude::*;

use crate::emit;

/// A refusal: traced, and the error.
fn refuse(line: &str) -> JsError {
    emit(&format!("JOIN: REFUSED — {line}"));
    JsError::new(line)
}

/// The person's key from the page's 32 bytes.
fn person(key: &[u8]) -> Result<SigningKey, JsError> {
    <[u8; 32]>::try_from(key)
        .map(|bytes| SigningKey::from_bytes(&bytes))
        .map_err(|_| refuse(&format!("the page holds {} bytes, not a key's 32", key.len())))
}

fn link(token: &str) -> Result<Token, JsError> {
    Token::decode(token).ok_or_else(|| refuse("the fragment is not a link's token"))
}

/// The body of `POST /ask`: the link's key and the person's, signed by
/// the link's key, as `svrngs join` asks.
#[wasm_bindgen]
pub fn ask(key: Vec<u8>, token: String) -> Result<String, JsError> {
    let req = AskRequest::sign(&link(&token)?.key(), &person(&key)?);
    emit(&format!("JOIN: asking the issuer what the link is for, as {}", req.key));
    Ok(serde_json::to_string(&req).expect("an ask serializes"))
}

/// A group the issuer answered, believed: its id and its name, a device
/// link's person, whom the phone joins as (bash-devices), and who made it,
/// so two of one name are told apart (v1ui-door (d)).
#[wasm_bindgen]
pub struct Group {
    id: String,
    name: String,
    person: Option<String>,
    maker: String,
}

#[wasm_bindgen]
impl Group {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn person(&self) -> Option<String> {
        self.person.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn maker(&self) -> String {
        self.maker.clone()
    }
}

/// The issuer's answer to an ask, believed only when its seed hashes to
/// the group it names (`svrngs_core::seed::answered`), as `svrngs join`
/// believes it.
#[wasm_bindgen]
pub fn believe(asked: String) -> Result<Group, JsError> {
    let asked: Asked = serde_json::from_str(&asked).map_err(|e| refuse(&format!("the answer is not an ask's: {e}")))?;
    let Some(seed) = answered(&asked.group, &asked.seed) else {
        return Err(refuse("the answer's seed does not hash to the group it names"));
    };
    emit(&format!(
        "JOIN: the answer is believed — the group {}, a device link of {:?}",
        asked.group, asked.person
    ));
    Ok(Group {
        id: asked.group,
        name: seed.name().to_string(),
        person: asked.person.map(|p| p.to_string()),
        maker: seed.founder().to_string(),
    })
}

/// The body of `POST /redeem`: the person's key joins the group `asked`
/// names as `name`, through the link, held by a browser (DESIGN §3, call
/// 2). A name that is not one is refused here, with why, before anything
/// is sent.
#[wasm_bindgen]
pub fn redemption(key: Vec<u8>, token: String, asked: String, name: String) -> Result<String, JsError> {
    let joining = person(&key)?;
    let link = link(&token)?;
    let asked: Asked = serde_json::from_str(&asked).map_err(|e| refuse(&format!("the answer is not an ask's: {e}")))?;
    let name = Name::parse(&name).map_err(|why| refuse(&why.to_string()))?;
    let req = RedeemRequest::sign(
        &link.key(),
        &joining,
        &asked.group,
        &asked.invite,
        &Person::from(name),
        Device::Browser,
    );
    emit(&format!(
        "JOIN: redeeming the link as {:?}, {}, a browser's key",
        req.person, req.key
    ));
    Ok(serde_json::to_string(&req).expect("a redemption serializes"))
}
