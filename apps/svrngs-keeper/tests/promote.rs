//! `keep <port>`, decided (src/promote.rs): text the host gave in, the card
//! and the declines out. No process is read or stopped here; the host half is
//! `tests/promote.sh`.

use svrngs_keeper::card::{Card, Protocol};
use svrngs_keeper::promote::{
    Declined, Listen, Proc, Props, Stat, added, adopts, copy_card, declines, homed, known, listens, llama_card,
    name_for, not_carried, props, shown, stat,
};

const HOME: &str = "/home/sam";

fn proc(pid: u32, session: u32, argv: &[&str]) -> Proc {
    Proc {
        pid,
        stat: Stat {
            ppid: 1,
            session,
            start: 7,
        },
        exe: "/usr/bin/llama-server".into(),
        argv: argv.iter().map(|w| w.to_string()).collect(),
        cwd: "/home/sam/models".into(),
        env: Ok(vec![("PATH".into(), "/usr/bin".into())]),
        cgroup: "/user.slice/user-1000.slice/user@1000.service/app.slice/app-ptyxis-1.scope".into(),
        typing: false,
    }
}

fn words(card: &str) -> Vec<String> {
    Card::parse(card).unwrap().native.unwrap().command
}

#[test]
fn ss_lines_give_each_socket_on_the_port_and_who_holds_it() {
    let ss = "LISTEN 0 512 127.0.0.1:8101 0.0.0.0:* users:((\"llama-server\",pid=3091384,fd=10))\n\
              LISTEN 0 512 [::1]:8101 [::]:* users:((\"gunicorn\",pid=10,fd=5),(\"gunicorn\",pid=11,fd=5))\n\
              LISTEN 0 4096 127.0.0.53%lo:8101 0.0.0.0:*\n\
              LISTEN 0 5 127.0.0.1:18101 0.0.0.0:* users:((\"x\",pid=4,fd=3))\n";
    assert_eq!(
        listens(ss, 8101),
        vec![
            Listen {
                addr: "127.0.0.1".into(),
                pids: vec![3091384]
            },
            Listen {
                addr: "[::1]".into(),
                pids: vec![10, 11]
            },
            Listen {
                addr: "127.0.0.53".into(),
                pids: vec![]
            },
        ],
        "port 18101 is not port 8101, and another account's socket names no pid"
    );
}

#[test]
fn stat_counts_its_fields_from_the_last_parenthesis() {
    let line =
        "3092066 (a) b (c) S 3085228 3092066 18284534 34817 3092066 4194560 1 2 3 4 5 6 7 8 20 0 1 0 99887766 1000";
    assert_eq!(
        stat(line),
        Some(Stat {
            ppid: 3085228,
            session: 18284534,
            start: 99887766
        })
    );
    assert_eq!(stat("12 (cut short) S 1"), None);
}

#[test]
fn a_parent_comes_with_it_unless_it_is_the_terminal_s_shell() {
    let script = proc(20, 5, &["sh", "./serve.sh", "8101"]);
    assert!(adopts(&script, 5), "a wrapper script is adopted");
    assert!(adopts(&proc(20, 5, &["bash", "-c", "llama-server …; sleep 1"]), 5));
    assert!(
        adopts(&proc(20, 5, &["python3", "run.py"]), 5),
        "so is a program that started it"
    );
    assert!(
        !adopts(&proc(5, 5, &["bash", "./serve.sh"]), 5),
        "the session's leader is the terminal's"
    );
    assert!(!adopts(&proc(20, 5, &["bash"]), 5), "a shell typed in a terminal");
    assert!(!adopts(&proc(20, 5, &["-zsh"]), 5), "a login shell");
    assert!(!adopts(&proc(20, 5, &["/bin/bash", "-i", "-l"]), 5));
    assert!(!adopts(&script, 6), "another session's");
    assert!(!adopts(&proc(1, 5, &["systemd"]), 5));
}

#[test]
fn each_decline_names_what_it_cannot_carry() {
    let port = 8101;
    let ok = proc(30, 5, &["llama-server", "--port", "8101"]);
    assert_eq!(declines(std::slice::from_ref(&ok), port, (32768, 60999)), None);

    let mut kept = ok.clone();
    kept.cgroup = "/user.slice/user@1000.service/app.slice/svrngs-keeper-coder.service".into();
    assert_eq!(
        declines(&[kept], port, (32768, 60999)),
        Some(Declined::Kept {
            port,
            unit: "svrngs-keeper-coder.service".into()
        })
    );

    let mut typing = ok.clone();
    typing.typing = true;
    let wrapper = proc(20, 5, &["sh", "serve.sh"]);
    assert_eq!(
        declines(
            &[ok.clone(), {
                let mut w = wrapper.clone();
                w.typing = true;
                w
            }],
            port,
            (32768, 60999)
        ),
        Some(Declined::Typed("sh (pid 20)".into())),
        "the adopted wrapper waits for typing"
    );
    assert!(matches!(declines(&[typing], port, (1, 2)), Some(Declined::Typed(_))));

    let mut sealed = wrapper.clone();
    sealed.env = Err("Permission denied".into());
    assert!(
        matches!(declines(&[ok.clone(), sealed], port, (1, 2)), Some(Declined::Environment(who, _)) if who == "sh (pid 20)"),
        "what is started again on a failure is the adopted process, so its environment is the one read"
    );

    let bare = proc(30, 5, &["python3", "-m", "http.server", "--bind", "127.0.0.1", "0"]);
    assert_eq!(
        declines(std::slice::from_ref(&bare), 41234, (32768, 60999)),
        Some(Declined::Changes(41234, 32768, 60999))
    );
    assert_eq!(
        declines(std::slice::from_ref(&bare), 8000, (32768, 60999)),
        None,
        "a port below the range is a program's own default"
    );
    let named = proc(30, 5, &["serve", "--listen=127.0.0.1:41234"]);
    assert_eq!(
        declines(&[named], 41234, (32768, 60999)),
        None,
        "a port its command names stays"
    );
    let mut by_env = bare.clone();
    by_env.env = Ok(vec![("PORT".into(), "41234".into())]);
    assert_eq!(
        declines(&[by_env], 41234, (32768, 60999)),
        None,
        "so does one its environment names"
    );
    assert!(
        matches!(
            declines(&[proc(30, 5, &["serve", "--port", "141234"])], 41234, (32768, 60999)),
            Some(Declined::Changes(..))
        ),
        "a port is named as a whole number"
    );

    let mut gone = ok.clone();
    gone.exe = "/usr/bin/llama-server (deleted)".into();
    assert!(matches!(declines(&[gone], port, (1, 2)), Some(Declined::Cannot(why)) if why.contains("replaced")));
    let tilde = proc(30, 5, &["serve", "~/data", "8101"]);
    assert!(matches!(declines(&[tilde], port, (1, 2)), Some(Declined::Cannot(why)) if why.contains("~/data")));
}

#[test]
fn the_settings_named_are_those_the_manager_lacks_or_has_otherwise() {
    let env: Vec<(String, String)> = [
        ("PATH", "/usr/bin:/home/sam/bin"),
        ("HOME", "/home/sam"),
        ("HIP_VISIBLE_DEVICES", "0"),
        ("PWD", "/home/sam/models"),
        ("SHLVL", "2"),
        ("_", "/usr/bin/llama-server"),
        ("INVOCATION_ID", "4f1c"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let manager = "HOME=/home/sam\nPATH=/usr/bin\nLANG=en_US.UTF-8\n";
    assert_eq!(added(&env, manager), ["HIP_VISIBLE_DEVICES", "PATH"]);
}

#[test]
fn llama_server_is_composed_from_what_it_reports() {
    let p = props(r#"{"model_path":"qwen.gguf","default_generation_settings":{"n_ctx":4096,"params":{}}}"#).unwrap();
    assert_eq!(
        p,
        Props {
            model_path: "qwen.gguf".into(),
            n_ctx: 4096
        }
    );
    assert!(props(r#"{"model_path":"q.gguf"}"#).is_err(), "no context, no card");

    let listener = proc(
        30,
        5,
        &["llama-server", "-m", "qwen.gguf", "--port", "8101", "-ngl", "99"],
    );
    assert!(known(std::slice::from_ref(&listener)));
    assert!(
        !known(&[listener.clone(), proc(20, 5, &["sh", "serve.sh"])]),
        "under a wrapper, it is copied"
    );

    let card = llama_card("coder", &listener, "127.0.0.1", 8101, &p, HOME);
    let read = Card::parse(&card).unwrap();
    assert_eq!(
        (read.title.as_str(), read.door.port, read.door.protocol),
        ("coder", 8101, Protocol::Http)
    );
    assert_eq!(read.about, "llama-server with your model ~/models/qwen.gguf");
    assert_eq!(
        words(&card),
        [
            "/usr/bin/llama-server",
            "--model",
            "~/models/qwen.gguf",
            "--host",
            "127.0.0.1",
            "--port",
            "8101",
            "--ctx-size",
            "4096"
        ],
        "a relative model path is its folder's, under ~/"
    );
    assert_eq!(read.native.unwrap().folder, "~/models");

    let tilde = Props {
        model_path: "/home/sam/.cache/llama.cpp/q.gguf".into(),
        n_ctx: 2048,
    };
    let card = llama_card("coder", &listener, "*", 8101, &tilde, HOME);
    assert_eq!(words(&card)[2..5], ["~/.cache/llama.cpp/q.gguf", "--host", "0.0.0.0"]);
    let v6 = llama_card("coder", &listener, "[::1]", 8101, &tilde, HOME);
    assert_eq!(words(&v6)[4], "::1");

    assert_eq!(
        not_carried(&listener.argv),
        ["-ngl"],
        "said, since the kept one starts without it"
    );
}

#[test]
fn anything_else_is_copied_with_its_name_its_words_and_its_folder() {
    let mut py = proc(30, 5, &["coder-py", "-m", "http.server", "--bind", "127.0.0.1", "8101"]);
    py.exe = "/usr/bin/python3.15".into();
    py.cwd = "/srv/site".into();
    let card = copy_card("coder-py", &py, 8101, true, HOME);
    let native = Card::parse(&card).unwrap().native.unwrap();
    assert_eq!(native.argv0.as_deref(), Some("coder-py"));
    assert_eq!(
        native.command,
        [
            "/usr/bin/python3.15",
            "-m",
            "http.server",
            "--bind",
            "127.0.0.1",
            "8101"
        ]
    );
    assert_eq!(
        (native.folder.as_str(), native.environment.len()),
        ("/srv/site", 0),
        "no environment is carried"
    );

    let mut odd = proc(
        30,
        5,
        &["/usr/bin/llama-server", "--alias", "50% \"$HOME\" #x: [y]", "8101"],
    );
    odd.cwd = HOME.into();
    let card = copy_card("odd", &odd, 8101, false, HOME);
    let read = Card::parse(&card).unwrap();
    let native = read.native.unwrap();
    assert_eq!(
        native.argv0, None,
        "a first argument that is the program is no other name"
    );
    assert_eq!(
        native.command[2], "50% \"$HOME\" #x: [y]",
        "every word comes through as it was"
    );
    assert_eq!((native.folder.as_str(), read.door.protocol), ("~", Protocol::Tcp));
}

#[test]
fn a_name_comes_from_the_program_or_its_script() {
    let argv = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(
        name_for(&argv(&["/usr/bin/llama-server", "-m", "q.gguf"])),
        "llama-server"
    );
    assert_eq!(name_for(&argv(&["sh", "./Serve Model.sh", "8101"])), "serve-model");
    assert_eq!(name_for(&argv(&["coder-py", "-m", "http.server"])), "coder-py");
    assert_eq!(homed("/home/sam", HOME), "~");
    assert_eq!(homed("/home/samuel/x", HOME), "/home/samuel/x");
    assert_eq!(shown(&argv(&["sh", "-c", "a b", "it's"])), r"sh -c 'a b' 'it'\''s'");
}
