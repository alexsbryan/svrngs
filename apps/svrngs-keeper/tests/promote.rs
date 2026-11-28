//! Which sockets hold a port, and whose, as `ss` names them.

use svrngs_keeper::promote::{Listen, listens};

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
