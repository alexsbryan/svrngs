//! page-open's v2-only tests (main.rs): the phone page opens into its
//! group, and the home page reads its groups.

use serde_json::json;
use svrngs_core::link::Token;
use svrngs_node::SigningKey;

use crate::drive::drive;
use crate::harness::{relay, relay_url, site_of, Engine};

in_chromium!(
    /// The group's link on a computer, on v2 (page-open (a)): the line
    /// that gets svrngs, composed from the link's `#` part as v1's is
    /// (web/app.js `offer`), and Copy (ui/copy.js), whose press writes the
    /// line and says "Copied" in the button's place, its machine traced as
    /// `COPY:`; under the page's one policy, with no violation and no page
    /// error. Watched failing on v1 (`UI` flipped), which traces no COPY.
    the_computer_s_line_is_offered_and_copied_on_v2
);
async fn the_computer_s_line_is_offered_and_copied_on_v2(engine: Engine) {
    let relay = relay().await;
    let token = Token {
        issuer: SigningKey::from_bytes(&[1; 32]).verifying_key(),
        secret: [0; 16],
        relay: Some(relay_url(&relay).parse().expect("a URL")),
    }
    .encode();
    let args = json!({ "site": site_of(crate::UI), "token": token });
    let seen = drive(engine, "v2/open.mjs", args, None).await;
    let origin = seen["origin"].as_str().unwrap();
    let line = format!("curl -fsSL {origin}install.sh | sh -s -- '{origin}#{token}'");
    assert_eq!(seen["line"], line, "{seen:#}");
    assert!(
        seen["words"]
            .as_str()
            .unwrap()
            .starts_with("On this computer, paste this line"),
        "{seen:#}"
    );
    assert_eq!(seen["button"], "Copied", "{seen:#}");
    assert_eq!(seen["written"], line, "{seen:#}");
    assert_eq!(seen["copied"], line, "{seen:#}");
    assert_eq!(
        seen["violations"],
        json!([]),
        "the page's policy held: {seen:#}"
    );
    assert_eq!(seen["errors"], json!([]), "{seen:#}");
    let lines: Vec<&str> = seen["lines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l.as_str())
        .collect();
    let copy: Vec<&&str> = lines.iter().filter(|l| l.starts_with("COPY: ")).collect();
    assert_eq!(
        copy,
        [
            &"COPY: copy: idle -> copying, running write",
            &"COPY: copied: copying -> copied"
        ],
        "{seen:#}"
    );
}
