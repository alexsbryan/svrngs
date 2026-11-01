// SPDX-License-Identifier: AGPL-3.0-or-later
//! The home's v2 page, in a test build only (page-scaffold (f);
//! ralph/next/page/CHARTER.md, "What is settled" 5). A node chosen v2 by
//! [`Node::ui`], or by `--ui v2` in the binary (cli.rs), answers `/`, `/ask`,
//! `/question.js` and each file of `home/v2/`, `home/ui/` and `web/vendor/`
//! at `/v2/`, `/ui/` and `/vendor/`, read from the sources as they are now,
//! so a row adds a file and never a route. A name is
//! `^[a-z0-9-]+\.(js|css|html)$`, so nothing beside those files is reached.
//! It is a layer inside home.rs's `only_itself`, so the page's one rule and
//! its one policy hold for it, and any other request, or any to a node left
//! v1, goes on to v1's routes, `/home.js` and `/look.css` among them. The
//! release refuses a binary that carries this file's trace, `svrngs-ui-v2`
//! (scripts/build-release.sh).

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::node::{Asking, Node};

/// Which page a node's home is: v1, the live one, unless chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ui {
    V1,
    V2,
}

/// A node's one choice, made once by its owner before its page is asked.
pub(crate) type Chosen = std::sync::Arc<std::sync::OnceLock<Ui>>;

impl Ui {
    /// `--ui v1|v2` taken out of the binary's `args` (cli.rs): what it
    /// chose, v1 when absent, and the rest; or why it is neither.
    pub fn asked(args: impl IntoIterator<Item = String>) -> Result<(Ui, Vec<String>), String> {
        let mut args: Vec<String> = args.into_iter().collect();
        let Some(at) = args.iter().position(|a| a == "--ui") else {
            return Ok((Ui::V1, args));
        };
        let ui = match args.drain(at..(at + 2).min(args.len())).nth(1).as_deref() {
            Some("v1") => Ui::V1,
            Some("v2") => Ui::V2,
            _ => return Err("--ui is v1 or v2".into()),
        };
        Ok((ui, args))
    }
}

impl Node {
    /// This node's home page chosen `ui`, once: a second choice is refused.
    pub fn ui(self, ui: Ui) -> Self {
        let chosen = self.asking.ui.set(ui).is_ok();
        tracing::debug!(target: "home", ?ui, chosen, "home: chosen once (svrngs-ui-v2)");
        self
    }
}

/// The layer: v2's file, read now, at a node chosen v2; else v1's routes.
pub(crate) async fn serve(at: State<Asking>, req: Request, next: Next) -> Response {
    let file = match req.uri().path().rsplit_once('/') {
        _ if at.ui.get() != Some(&Ui::V2) => None,
        Some(("", "" | "ask")) => Some("src/home/v2/index.html".into()),
        Some(("", "question.js")) => Some("src/home/question.js".into()),
        Some((d @ ("/v2" | "/ui"), name)) if named(name) => Some(format!("src/home{d}/{name}")),
        Some(("/vendor", name)) if named(name) => Some(format!("../../web/vendor/{name}")),
        _ => None,
    };
    tracing::debug!(target: "home", path = %req.uri().path(), ?file, "home: v2's file, else v1's");
    let Some(file) = file else {
        return next.run(req).await;
    };
    let kind = match file.rsplit_once('.') {
        Some((_, "html")) => "text/html; charset=utf-8",
        Some((_, "css")) => "text/css",
        _ => "text/javascript",
    };
    let body = tokio::fs::read(format!("{}/{file}", env!("CARGO_MANIFEST_DIR"))).await;
    tracing::debug!(target: "home", file, read = body.is_ok(), "home: v2's file read, else 404");
    let Ok(body) = body else {
        return (StatusCode::NOT_FOUND, "There is no such file on this page.").into_response();
    };
    ([("content-type", kind)], body).into_response()
}

/// Whether `name` is `^[a-z0-9-]+\.(js|css|html)$`.
fn named(name: &str) -> bool {
    let allowed = |b: u8| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-');
    name.split_once('.').is_some_and(|(stem, kind)| {
        !stem.is_empty() && stem.bytes().all(allowed) && matches!(kind, "js" | "css" | "html")
    })
}
