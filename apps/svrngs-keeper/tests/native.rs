//! A native card runs a program installed on this computer, not services
//! (`x-svrngs.native`). It is read by the same rules as every card, and each
//! refusal names its field. No systemd and no podman here: these read text.

use svrngs_keeper::card::{Card, Native, Protocol, Refused};
use svrngs_keeper::podman::Podman;
use svrngs_keeper::runtime::{Agreed, App, Name, RunError};

const CODER: &str = r#"
x-svrngs:
  title: coder
  about: llama-server with qwen2.5-7b
  door: { port: 8101, protocol: http }
  state: []
  native:
    command: [/usr/bin/llama-server, -m, ~/models/qwen2.5-7b.gguf, --port, 8101]
    folder: ~/models
    environment: [HIP_VISIBLE_DEVICES, _X1]
"#;

fn refused(card: &str) -> (String, String) {
    match Card::parse(card) {
        Err(Refused::Field { field, why }) => (field, why),
        other => panic!("expected a refusal by field, got {other:?}"),
    }
}

fn with(from: &str, to: &str) -> String {
    assert!(CODER.contains(from), "the card has no {from:?}");
    CODER.replacen(from, to, 1)
}

#[test]
fn a_native_card_is_read_whole() {
    let card = Card::parse(CODER).unwrap();
    assert_eq!(
        card.native,
        Some(Native {
            command: [
                "/usr/bin/llama-server",
                "-m",
                "~/models/qwen2.5-7b.gguf",
                "--port",
                "8101"
            ]
            .map(String::from)
            .to_vec(),
            folder: "~/models".into(),
            environment: vec!["HIP_VISIBLE_DEVICES".into(), "_X1".into()],
        })
    );
    assert!(card.services.is_empty() && card.volumes.is_empty() && card.asks.is_empty());
    assert_eq!(
        (card.door.port, card.door.protocol),
        (8101, Protocol::Http),
        "its door is its program's own port, which no service lists"
    );
}

#[test]
fn its_environment_is_optional_and_its_folder_may_be_absolute() {
    let card = Card::parse(&with(
        "    folder: ~/models\n    environment: [HIP_VISIBLE_DEVICES, _X1]\n",
        "    folder: /srv/models\n",
    ))
    .unwrap();
    let native = card.native.unwrap();
    assert_eq!((native.folder.as_str(), native.environment.len()), ("/srv/models", 0));
}

#[test]
fn a_native_card_runs_alone() {
    let services = format!("{CODER}services:\n  web:\n    image: x\n");
    assert_eq!(refused(&services).0, "services");
    let volumes = format!("{CODER}volumes:\n  data: {{}}\n");
    assert_eq!(refused(&volumes).0, "volumes");
    let asks = with(
        "  state: []\n",
        "  state: []\n  asks: [{ say: s, link: \"https://x\", sets: { A: b } }]\n",
    );
    assert_eq!(refused(&asks).0, "x-svrngs.asks");
}

/// Each value becomes a line of a unit file: a newline in one would begin a
/// line of the card's choosing.
#[test]
fn a_control_character_is_refused_in_the_command_and_the_folder() {
    let (field, why) = refused(&with("-m,", "\"-m\\nExecStartPre=/bin/evil\","));
    assert_eq!(field, "x-svrngs.native.command");
    assert!(why.contains("control character"), "{why}");
    let (field, _) = refused(&with("folder: ~/models", "folder: \"/srv\\rx\""));
    assert_eq!(field, "x-svrngs.native.folder");
}

#[test]
fn each_refusal_of_a_native_field_names_it() {
    let cases = [
        (with("folder: ~/models", "folder: models"), "x-svrngs.native.folder"),
        (with("folder: ~/models", "folder: ~models"), "x-svrngs.native.folder"),
        (with("HIP_VISIBLE_DEVICES", "HIP=1"), "x-svrngs.native.environment"),
        (with("_X1", "1X"), "x-svrngs.native.environment"),
        (
            with(
                "command: [/usr/bin/llama-server, -m, ~/models/qwen2.5-7b.gguf, --port, 8101]",
                "command: []",
            ),
            "x-svrngs.native.command",
        ),
        (
            with(
                "command: [/usr/bin/llama-server, -m, ~/models/qwen2.5-7b.gguf, --port, 8101]",
                "command: \"/usr/bin/llama-server -m x\"",
            ),
            "x-svrngs.native.command",
        ),
        (
            with("    folder: ~/models\n", "    folder: ~/models\n    user: sam\n"),
            "x-svrngs.native.user",
        ),
        (with("    folder: ~/models\n", ""), "x-svrngs.native"),
    ];
    for (card, field) in cases {
        assert_eq!(refused(&card).0, field, "{card}");
    }
}

#[test]
fn a_card_with_neither_services_nor_native_is_refused_as_before() {
    let none = CODER.replace(
        "  native:\n    command: [/usr/bin/llama-server, -m, ~/models/qwen2.5-7b.gguf, --port, 8101]\n    folder: ~/models\n    environment: [HIP_VISIBLE_DEVICES, _X1]\n",
        "",
    );
    assert_eq!(refused(&none), ("the card".into(), "has no services".into()));
}

/// Give runs a card in podman, and a native card's program is this
/// computer's own: the runtime refuses it before podman is asked anything.
#[test]
fn give_s_runtime_refuses_a_native_card() {
    let card = Card::parse(CODER).unwrap();
    let podman = Podman::at("/nonexistent/podman");
    let app = App::new(&podman, &card, Name::new("svrngs-test-native").unwrap());
    match app.start(18101, &Agreed::to(&[])) {
        Err(RunError::Native(title)) => assert_eq!(title, "coder"),
        other => panic!("expected the native refusal, got {other:?}"),
    }
}
