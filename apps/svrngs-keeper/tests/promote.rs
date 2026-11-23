//! The read-only prefill uses the listener's model path, not its command.

use std::path::Path;

use svrngs_keeper::card::{Card, Protocol};
use svrngs_keeper::promote::{Listen, listens, llama_card, props};

const HOME: &str = "/home/sam";

#[test]
fn ss_lines_give_each_socket_on_the_port_and_who_holds_it() {
    let ss = "LISTEN 0 512 127.0.0.1:8101 0.0.0.0:* users:((\"llama-server\",pid=3091384,fd=10)) cgroup:/user.slice/coder.service\n\
              LISTEN 0 512 [::1]:8101 [::]:* users:((\"gunicorn\",pid=10,fd=5),(\"gunicorn\",pid=11,fd=5))\n\
              LISTEN 0 4096 127.0.0.53%lo:8101 0.0.0.0:*\n\
              LISTEN 0 5 127.0.0.1:18101 0.0.0.0:* users:((\"x\",pid=4,fd=3))\n";
    assert_eq!(
        listens(ss, 8101),
        vec![
            Listen {
                addr: "127.0.0.1".into(),
                pids: vec![3091384],
                cgroup: Some("/user.slice/coder.service".into()),
            },
            Listen {
                addr: "[::1]".into(),
                pids: vec![10, 11],
                cgroup: None,
            },
            Listen {
                addr: "127.0.0.53".into(),
                pids: vec![],
                cgroup: None,
            },
        ],
        "port 18101 is not port 8101, and another account's socket names no pid"
    );
}

#[test]
fn llama_server_is_prefilled_from_the_model_path_it_reports() {
    let folder = Path::new("/home/sam/models");
    let model = props(r#"{"model_path":"qwen.gguf","default_generation_settings":{"n_ctx":4096}}"#).unwrap();
    assert!(props(r#"{"default_generation_settings":{"n_ctx":4096}}"#).is_err());

    let card = llama_card("coder", folder, 8101, &model, HOME);
    let read = Card::parse(&card).unwrap();
    assert_eq!(
        (read.title.as_str(), read.door.port, read.door.protocol),
        ("coder", 8101, Protocol::Http)
    );
    let native = read.native.unwrap();
    assert_eq!(
        native.command,
        [
            "llama-server",
            "--model",
            "~/models/qwen.gguf",
            "--host",
            "127.0.0.1",
            "--port",
            "8101"
        ],
        "a relative model path is its folder's, under ~/"
    );
    assert_eq!(native.folder, "~/models");
    assert!(native.environment.is_empty());

    let model = props(r#"{"model_path":"/home/sam/.cache/llama.cpp/q.gguf"}"#).unwrap();
    let card = llama_card("coder", folder, 8101, &model, HOME);
    let native = Card::parse(&card).unwrap().native.unwrap();
    assert_eq!(native.command[2], "~/.cache/llama.cpp/q.gguf");
}
