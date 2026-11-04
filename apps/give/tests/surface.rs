//! Give speaks only svrngs's public surface (CHARTER, fence 4): it links no
//! svrngs crate and takes nothing from the svrngs workspace's tables, so
//! the boundary is structural and moving give out costs nothing.

#[test]
fn give_links_no_svrngs_crate() {
    let manifest = include_str!("../Cargo.toml");
    let deps = manifest.split("[dependencies]").nth(1).expect("a [dependencies] table");
    for line in deps
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        assert!(!line.starts_with("svrngs"), "give links a svrngs crate: {line}");
        assert!(
            !line.contains("crates/"),
            "give links a crate of svrngs's by path: {line}"
        );
        assert!(
            !line.contains("workspace"),
            "give takes a dependency from the svrngs workspace: {line}"
        );
    }
    assert!(
        !manifest.contains(".workspace"),
        "give takes its package fields from the svrngs workspace"
    );
}
