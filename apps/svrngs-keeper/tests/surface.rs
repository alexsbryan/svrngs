//! svrngs-keeper knows no group: it links no svrngs product crate and no Give,
//! and takes nothing from the svrngs workspace's tables, so Give and svrngs
//! can each call it and it moves out at no cost.

#[test]
fn the_keeper_links_no_svrngs_crate_and_no_give() {
    let manifest = include_str!("../Cargo.toml");
    let deps = manifest.split("[dependencies]").nth(1).expect("a [dependencies] table");
    for line in deps
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        assert!(!line.starts_with("svrngs"), "the keeper links a svrngs crate: {line}");
        assert!(!line.starts_with("give"), "the keeper links Give: {line}");
        assert!(
            !line.contains("crates/"),
            "the keeper links a crate of svrngs's by path: {line}"
        );
        assert!(
            !line.contains("workspace"),
            "the keeper takes a dependency from the svrngs workspace: {line}"
        );
    }
    assert!(
        !manifest.contains(".workspace"),
        "the keeper takes its package fields from the svrngs workspace"
    );
}
