// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn the_release_s_name_is_install_sh_s_for_each_platform() {
    assert_eq!(
        platform_of("macos", "aarch64").as_deref(),
        Some("darwin-aarch64")
    );
    assert_eq!(
        platform_of("macos", "x86_64").as_deref(),
        Some("darwin-x86_64")
    );
    assert_eq!(
        platform_of("linux", "aarch64").as_deref(),
        Some("linux-aarch64")
    );
    assert_eq!(
        platform_of("linux", "x86_64").as_deref(),
        Some("linux-x86_64")
    );
    assert_eq!(platform_of("windows", "x86_64"), None);
    assert_eq!(platform_of("linux", "riscv64"), None);
}

#[test]
fn the_sums_line_for_this_computer_is_read_and_any_other_is_not() {
    let sums = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  svrngs-darwin-aarch64.tar.gz\n\
                bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb *give-darwin-aarch64.tar.gz\n";
    assert_eq!(
        wanted(sums, "svrngs-darwin-aarch64.tar.gz").unwrap(),
        "a".repeat(64)
    );
    assert!(matches!(
        wanted(sums, "svrngs-linux-x86_64.tar.gz"),
        Err(UpdateError::NotListed)
    ));
    assert!(matches!(
        wanted(
            "not-a-sum  svrngs-darwin-aarch64.tar.gz\n",
            "svrngs-darwin-aarch64.tar.gz"
        ),
        Err(UpdateError::CouldNotJudge(_))
    ));
}
