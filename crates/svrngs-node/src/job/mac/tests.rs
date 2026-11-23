use super::*;

#[test]
fn only_the_installed_agent_and_the_asking_daemon_match() {
    let path = Path::new("/home/Sam/Library/LaunchAgents/sh.svrnme.svrngs.plist");
    let good = format!(
        "gui/502/sh.svrnme.svrngs = {{\n\tpath = {}\n\tpid = 37\n}}",
        path.display()
    );
    assert!(registered(&good, path, Some(37)));
    assert!(registered(&good, path, None));
    assert!(!registered(&good, path, Some(38)));
    assert!(!registered(&good, Path::new("/tmp/copied.plist"), None));
}
