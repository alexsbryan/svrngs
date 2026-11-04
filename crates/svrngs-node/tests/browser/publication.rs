//! The host's controls, with real native receipts and process restarts.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

const PHRASE: &str = "acorn-basil-cabin-daisy-eagle";

#[test]
fn onboarding_host_controls_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    for phase in ["pending", "open", "closing", "closed", "inspect"] {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "publication::onboarding_host_controls_child",
                "--nocapture",
            ])
            .env("SVRNGS_HOST_CONTROLS_DIR", dir.path())
            .env("SVRNGS_HOST_CONTROLS_PHASE", phase)
            .spawn()
            .unwrap();
        let began = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "host controls phase {phase}: {status}");
                break;
            }
            if began.elapsed() > Duration::from_secs(90) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("host controls phase {phase} timed out");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn onboarding_host_controls_child() {
    let Ok(dir) = std::env::var("SVRNGS_HOST_CONTROLS_DIR") else {
        return;
    };
    let phase = std::env::var("SVRNGS_HOST_CONTROLS_PHASE").unwrap();
    let dir = Path::new(&dir);
    let saved = std::fs::read_to_string(dir.join("join-resolver")).ok();
    let at = saved
        .as_deref()
        .unwrap_or("http://127.0.0.1:0/")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let listener = tokio::net::TcpListener::bind(at).await.unwrap();
    let resolver = format!("http://{}/", listener.local_addr().unwrap());
    std::fs::write(dir.join("join-resolver"), &resolver).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let failing = matches!(phase.as_str(), "pending" | "closing");
    let service = axum::Router::new().route(
        "/",
        axum::routing::post(move |body: axum::body::Bytes| {
            let seen = seen.clone();
            async move {
                seen.fetch_add(1, Ordering::SeqCst);
                let body: Value = serde_json::from_slice(&body).unwrap();
                if failing {
                    return (
                        axum::http::StatusCode::TOO_MANY_REQUESTS,
                        axum::Json(json!({"error":"quota"})),
                    );
                }
                let answer = if body["action"] == "close" {
                    json!({"id":body["id"], "closed":true})
                } else {
                    json!({"id":body["id"], "phrase":PHRASE})
                };
                (axum::http::StatusCode::OK, axum::Json(answer))
            }
        }),
    );
    let task = tokio::spawn(async move { axum::serve(listener, service).await.unwrap() });
    let relay = relay().await;
    let node = seeded(dir, relay_url(&relay), 1, "ada", 1).await;
    let before = node.rail().current_invite(NS).unwrap();
    let seen = drive(
        Engine::Chromium,
        "host-controls.mjs",
        json!({
            "ask":format!("http://svrngs.localhost:{}/", node.api_addr().port()),
            "phase":phase, "address":format!("{resolver}{PHRASE}")
        }),
        None,
    )
    .await;
    eprintln!("host controls {phase}: {seen:#}");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        usize::from(phase != "inspect"),
        "no automatic mutation retries"
    );
    let current = node.rail().current_invite(NS).unwrap();
    if matches!(phase.as_str(), "open" | "closing" | "inspect") {
        assert_eq!(
            before, current,
            "retry/failure/status must preserve the invitation"
        );
    } else if phase == "closed" {
        assert_ne!(
            before, current,
            "close must invalidate the copied underlying invitation"
        );
    }
    assert!(node
        .rail()
        .standing(NS)
        .unwrap()
        .member(&actor_of(&signing(1)))
        .is_some());
    node.endpoint().close().await;
    task.abort();
}
