// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a folder app's page asks that the policy on every answer
//! ([`super::carried`]'s `POLICY`, `scripts/trust_floor.txt`) refuses,
//! read from its files before anyone opens it (svrn-docs
//! `rings/current/EXTENSION_POINT.md` §7, G3). The browser refuses it
//! without a word to the page, so the page is blank, and a coding agent
//! sees no console: `svrngs check <folder>` prints this, and the app's
//! card on the home page says it. Read: `index.html`, and the scripts and
//! style sheets it names among its own files. Not judged: what a script
//! asks once it runs, such as a fetch built from a variable.

use std::path::Path;

/// What the policy refuses, a closed set (AGENTS 9), each said one way.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Refused {
    /// A font from another server, by its host.
    Font(String),
    /// A script from another server, by its host.
    Script(String),
    /// A style sheet from another server, by its host.
    Style(String),
    /// A script written inside the page: a `<script>` with no `src`, a
    /// handler such as `onclick=`, or a `javascript:` address.
    Inline,
}

impl Refused {
    /// What it is, why, and what to change, for a person and an agent alike.
    pub fn words(&self) -> String {
        let (host, what, fix) = match self {
            Refused::Inline => return "This page has a script written inside it, and apps run scripts only from their own files. Move it to a .js file beside index.html.".into(),
            Refused::Font(host) => (host, "a font", "save the .woff2 beside index.html and point @font-face at it"),
            Refused::Script(host) => (host, "a script", "save the script beside index.html and load it from there"),
            Refused::Style(host) => (host, "a style sheet", "save the style sheet beside index.html and load it from there"),
        };
        format!("This page asks {host} for {what}, and apps load only their own files. That way, opening an app tells nobody outside your group you were here. To fix it, {fix}.")
    }
}

/// One thing the policy refuses: the file it is in, its line from 1, what.
pub type Finding = (String, usize, Refused);

/// What the policy will refuse in the app at `dir`, by file and line: its
/// `index.html`, and each style sheet and script beside it, which are all
/// it may load; none when it holds no `index.html`. Not judged: `eval`,
/// which the policy refuses too, and what a script asks once it runs.
pub fn check(dir: &Path) -> Option<Vec<Finding>> {
    let page = std::fs::read_to_string(dir.join("index.html")).ok()?;
    // Names, schemes and hosts are any case; a file name is no matter here.
    let low = page.to_ascii_lowercase();
    let mut found = Vec::new();
    for (start, _) in low.match_indices('<') {
        let end = low[start..].find('>').map_or(low.len(), |e| start + e);
        let (tag, line) = (&low[start + 1..end], low[..start].matches('\n').count() + 1);
        let body = &low[(end + 1).min(low.len())..];
        let body = &body[..body.find("</").unwrap_or(0)];
        let script = tag.starts_with("script");
        let sheet = tag.starts_with("link") && tag.contains("stylesheet");
        let on = |a: &str| a.starts_with("on") && a.contains('=');
        let handler = tag.split_ascii_whitespace().any(on) || tag.contains("javascript:");
        // A `<script>` with a `src` loads that; one of JSON is data, not run.
        let elsewhere = tag.contains("src=") || tag.contains("json");
        let inline = script && !elsewhere && !body.trim().is_empty();
        let asked = away(tag, script).filter(|_| script || sheet);
        let inline = (inline || handler).then_some(Refused::Inline);
        let styled = asks(body, line, false).filter(|_| tag.starts_with("style"));
        let said = (asked.into_iter().chain(inline))
            .map(|r| (line, r))
            .chain(styled);
        found.extend(said.map(|(n, r)| ("index.html".to_string(), n, r)));
    }
    let beside = std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path());
    let ours = |e: &std::ffi::OsStr| e == "css" || e == "js" || e == "mjs";
    for path in beside.filter(|p| p.extension().is_some_and(ours)) {
        let css = path.extension().is_some_and(|e| e == "css");
        let file = path.file_name().unwrap_or_default().to_string_lossy();
        let read = std::fs::read_to_string(&path);
        tracing::debug!(target: "folder", %file, read = read.is_ok(), "check: a style sheet or script beside the page, read, or not");
        let text = read.unwrap_or_default();
        found.extend(asks(&text, 1, !css).map(|(n, r)| (file.to_string(), n, r)));
    }
    found.sort();
    found.dedup();
    tracing::debug!(target: "folder", dir = %dir.display(), found = found.len(), "check: what the policy will refuse in the app's page and the styles and scripts beside it");
    Some(found)
}

/// Where an address's host ends.
const ENDS: [char; 7] = ['/', '?', '#', '"', '\'', ')', '>'];
/// What a font's address names.
const FONTS: [&str; 4] = ["font", ".woff", ".ttf", ".otf"];

/// What loading the first address in `text` is refused as, when it names
/// another server: a script where one is asked, else a font where it names
/// one, else a style sheet.
fn away(text: &str, script: bool) -> Option<Refused> {
    let host = text[text.find("//")? + 2..].split(ENDS).next()?.trim();
    let font = FONTS.iter().any(|f| text.contains(f));
    let host =
        Some(host.to_string()).filter(|h| !h.is_empty() && !h.contains(char::is_whitespace))?;
    Some(match (script, font) {
        (true, _) => Refused::Script(host),
        (false, true) => Refused::Font(host),
        (false, false) => Refused::Style(host),
    })
}

/// Each thing `text`, a style sheet or a script, asks of another server,
/// by its line, `first` being its first's: what it imports, and a font a
/// style sheet names; an image it names is said nowhere.
fn asks(text: &str, first: usize, script: bool) -> impl Iterator<Item = (usize, Refused)> + '_ {
    text.lines().zip(first..).filter_map(move |(line, at)| {
        let code = line.trim_start();
        let import =
            code.starts_with("import") || code.contains("import(") || code.contains("@import");
        let font = |r: &Refused| matches!(r, Refused::Font(_)) && code.contains("url(");
        let asked = away(code, script).filter(|r| import || font(r));
        Some((at, asked.filter(|_| !code.starts_with("//"))?))
    })
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
