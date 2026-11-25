//! The read-only prefill reads how the listener was started: its own
//! arguments and folder, whatever the program, only where it is ours.

use svrngs_keeper::promote::{Listen, cmdline, listens, procargs};

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
fn a_process_s_arguments_read_back_as_it_was_given_them() {
    let given = ["llama-server", "--alias", "Sam's model", "", "--port", "8101"];
    let linux: Vec<u8> = given.iter().flat_map(|a| [a.as_bytes(), b"\0"].concat()).collect();
    assert_eq!(cmdline(&linux).unwrap(), given, "every argument, an empty one too");
    assert_eq!(cmdline(b"nginx: master process").unwrap(), ["nginx: master process"]);
    assert!(cmdline(b"").is_err(), "an exited process gives none");
    assert!(cmdline(b"ok\0\xff\0").is_err(), "an argument that is not text");

    let mac = |argc: i32, after: &[u8]| {
        let mut b = argc.to_ne_bytes().to_vec();
        b.extend_from_slice(b"/opt/homebrew/bin/llama-server\0\0\0\0");
        b.extend_from_slice(after);
        b
    };
    let args = b"llama-server\0--alias\0Sam's model\0--port\08101\0HOME=/Users/sam\0SECRET=x\0";
    assert_eq!(
        procargs(&mac(5, args)).unwrap(),
        ["llama-server", "--alias", "Sam's model", "--port", "8101"],
        "argc arguments, and none of the environment after them"
    );
    assert!(procargs(&mac(9, args)).is_err(), "fewer arguments than argc");
    assert!(procargs(&mac(0, args)).is_err());
    assert!(procargs(&mac(-1, args)).is_err());
    assert!(procargs(&[1, 0]).is_err(), "no argument count");
}

#[test]
fn another_account_s_process_is_not_read() {
    assert_eq!(
        svrngs_keeper::host::started(1).unwrap_err(),
        svrngs_keeper::host::THEIRS,
        "pid 1 is the system's"
    );
}

/// A program started by hand, with arguments, from a folder: facts reads
/// both back, whatever the program, and a quoted argument stays one.
#[test]
fn facts_reads_back_any_program_s_own_command_and_folder() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    let scratch = tempfile::tempdir().unwrap();
    let folder = scratch.path().canonicalize().unwrap().join("Sam's files");
    std::fs::create_dir(&folder).unwrap();
    let folder = folder.to_str().unwrap();
    let tail = [
        "-u",
        "-m",
        "http.server",
        "0",
        "--bind",
        "127.0.0.1",
        "--directory",
        folder,
    ];
    let mut listener = Command::new("python3")
        .args(tail)
        .current_dir(folder)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut said = String::new();
    BufReader::new(listener.stdout.take().unwrap())
        .read_line(&mut said)
        .unwrap();
    let port = said
        .split(" port ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .unwrap()
        .to_owned();
    let facts = Command::new(env!("CARGO_BIN_EXE_svrngs-keeper"))
        .args(["facts", &port, "--name", "files"])
        .output();
    listener.kill().unwrap();
    listener.wait().unwrap();
    let facts = facts.unwrap();
    assert!(facts.status.success(), "{}", String::from_utf8_lossy(&facts.stderr));
    let facts: serde_json::Value = serde_json::from_slice(&facts.stdout).unwrap();
    assert_eq!(facts["kind"], "prefill", "{facts}");
    assert_eq!(facts["name"], "files");
    assert_eq!(facts["port"], port.parse::<u16>().unwrap());
    assert_eq!(facts["folder"], folder);
    let argv = shlex::split(facts["command"].as_str().expect("a command")).unwrap();
    // A Mac's framework Python starts again as its app, under that path.
    let program = std::path::Path::new(&argv[0]).file_name().unwrap().to_str().unwrap();
    assert!(program.to_lowercase().starts_with("python"), "{argv:?}");
    assert_eq!(argv[1..], tail, "{facts}");
    println!("{facts}");
}
