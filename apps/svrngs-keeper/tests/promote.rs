//! The read-only prefill uses the listener's model path, not its command.

use std::path::Path;

use svrngs_keeper::promote::{Listen, listens, llama_command, props};

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

    let argv = |model: &str| shlex::split(&llama_command(folder, 8101, model).unwrap()).unwrap();
    assert_eq!(
        argv(&model),
        [
            "llama-server",
            "--model",
            "/home/sam/models/qwen.gguf",
            "--host",
            "127.0.0.1",
            "--port",
            "8101"
        ],
        "a relative model path is its folder's"
    );
    let model = props(r#"{"model_path":"/home/sam/.cache/llama.cpp/q.gguf"}"#).unwrap();
    assert_eq!(argv(&model)[2], "/home/sam/.cache/llama.cpp/q.gguf");
}

#[cfg(target_os = "linux")]
#[test]
fn facts_suggests_fields_only_for_llama_server() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    let dir = tempfile::tempdir().unwrap();
    for program in ["llama-server", "other-listener"] {
        let executable = dir.path().join(program);
        std::fs::copy("/usr/bin/python3", &executable).unwrap();
        let mut listener = Command::new(executable)
            .args(["-u", "-c", include_str!("props_fixture.py"), "0"])
            .current_dir(dir.path())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut port = String::new();
        BufReader::new(listener.stdout.take().unwrap())
            .read_line(&mut port)
            .unwrap();
        let facts = Command::new(env!("CARGO_BIN_EXE_svrngs-keeper"))
            .args(["facts", port.trim(), "--name", "coder"])
            .env("HOME", dir.path())
            .output();
        listener.kill().unwrap();
        listener.wait().unwrap();
        let facts = facts.unwrap();
        assert!(facts.status.success(), "{}", String::from_utf8_lossy(&facts.stderr));
        let facts: serde_json::Value = serde_json::from_slice(&facts.stdout).unwrap();
        if program == "llama-server" {
            assert_eq!(facts["kind"], "prefill");
            assert_eq!(facts["name"], "coder");
            assert_eq!(facts["port"], port.trim().parse::<u16>().unwrap());
            assert_eq!(facts["folder"], dir.path().to_str().unwrap());
            assert_eq!(
                shlex::split(facts["command"].as_str().expect("a command field")),
                Some(vec![
                    program.into(),
                    "--model".into(),
                    dir.path().join("Sam's model.gguf").to_str().unwrap().into(),
                    "--host".into(),
                    "127.0.0.1".into(),
                    "--port".into(),
                    port.trim().into()
                ]),
            );
        } else {
            assert_eq!(facts["kind"], "unavailable");
            for field in ["name", "folder", "command"] {
                assert!(facts.get(field).is_none(), "{facts}");
            }
        }
        println!("{program}: {}", facts["kind"]);
    }
}
