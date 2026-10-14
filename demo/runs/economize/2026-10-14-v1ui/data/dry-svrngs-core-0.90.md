# DRY report — `prod-8e97b27`

Scope: `crates/svrngs-core` — 43 file(s) under `crates/svrngs-core/`, matched on whole path components.

1900 function/method symbols · 466 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**4** exact-clone groups · **12** near-clone clusters · ~**223** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **2 copies × 3 lines** (`7e4a51e8197d`)
  - `signed_by_link` — crates/svrngs-core/src/joining.rs:56
  - `signed_by_link` — crates/svrngs-core/src/joining.rs:148
- **2 copies × 3 lines** (`f6a912a9fe34`)
  - `lines` — crates/svrngs-core/src/command.rs:87
  - `lines` — crates/svrngs-core/src/command.rs:105
- **2 copies × 3 lines** (`a588a00c526a`)
  - `as_str` — crates/svrngs-core/src/link.rs:186
  - `as_str` — crates/svrngs-core/src/synced.rs:67
- **2 copies × 3 lines** (`7d516fc1b939`)
  - `is_empty` — crates/svrngs-core/src/act_set.rs:40
  - `is_empty` — crates/svrngs-core/src/command.rs:109

## Near clones (similar, not identical)

- **2 symbols** · ~32 lines · cosine 0.921–0.921
  - `a_hand_on_moves_the_keeper_and_only_the_keepers_counts` — crates/svrngs-core/src/removal_tests.rs:268–298 (31 lines)
  - `a_removed_key_cannot_remove_ask_or_hand_on` — crates/svrngs-core/src/removal_tests.rs:298–329 (32 lines)
- **2 symbols** · ~26 lines · cosine 0.917–0.917
  - `a_stranger_under_the_keepers_name_does_not_stand` — crates/svrngs-core/src/register_tests.rs:426–450 (25 lines)
  - `a_hand_on_to_a_name_never_reaches_a_strangers_computer` — crates/svrngs-core/src/register_tests.rs:452–477 (26 lines)
- **2 symbols** · ~26 lines · cosine 0.904–0.904
  - `a_hand_on_survives_its_givers_removal` — crates/svrngs-core/src/removal_tests.rs:419–444 (26 lines)
  - `a_hand_on_that_made_no_keeper_is_cut_as_before` — crates/svrngs-core/src/removal_tests.rs:446–471 (26 lines)
- **2 symbols** · ~25 lines · cosine 0.901–0.901
  - `redeeming_again_writes_nothing` — crates/svrngs-core/src/register_tests.rs:105–121 (17 lines)
  - `the_decider_refuses_what_the_invite_does_not_allow` — crates/svrngs-core/src/register_tests.rs:118–142 (25 lines)
- **2 symbols** · ~18 lines · cosine 0.921–0.921
  - `a_lapsed_invite_is_refused_by_the_register_and_nothing_is_written` — crates/svrngs-core/src/register_tests.rs:139–156 (18 lines)
  - `a_signer_without_standing_writes_nothing` — crates/svrngs-core/src/register_tests.rs:153–167 (15 lines)
- **2 symbols** · ~18 lines · cosine 0.918–0.918
  - `admit_act` — crates/svrngs-core/src/membership_tests.rs:12–29 (18 lines)
  - `admit_of` — crates/svrngs-core/src/removal_tests.rs:17–29 (13 lines)
- **3 symbols** · ~9 lines · cosine 0.922–0.940
  - `decide` — crates/svrngs-core/src/record.rs:53–61 (9 lines)
  - `decide` — crates/svrngs-core/src/tests_support.rs:289–297 (9 lines)
  - `decide` — crates/svrngs-core/src/tests_support.rs:305–313 (9 lines)
- **2 symbols** · ~12 lines · cosine 0.929–0.929
  - `a_redemption_by_someone_who_did_not_sign_the_invite_is_refused` — crates/svrngs-core/src/membership_invite_tests.rs:134–145 (12 lines)
  - `a_redemption_co_signed_by_another_key_is_refused` — crates/svrngs-core/src/membership_invite_tests.rs:144–155 (12 lines)
- **2 symbols** · ~9 lines · cosine 0.934–0.934
  - `name` — crates/svrngs-core/src/lib.rs:871–879 (9 lines)
  - `verify` — crates/svrngs-core/src/lib.rs:874–882 (9 lines)
- **2 symbols** · ~9 lines · cosine 0.924–0.924
  - pub `from_hash` — crates/svrngs-core/src/op.rs:82–90 (9 lines)
  - pub `as_bytes` — crates/svrngs-core/src/op.rs:85–93 (9 lines)
- **2 symbols** · ~9 lines · cosine 0.921–0.921
  - `actor` — crates/svrngs-core/src/tests_support.rs:149–157 (9 lines)
  - `sign` — crates/svrngs-core/src/tests_support.rs:152–160 (9 lines)
- **2 symbols** · ~9 lines · cosine 0.912–0.912
  - `actor` — crates/svrngs-core/src/lib.rs:827–835 (9 lines)
  - `sign` — crates/svrngs-core/src/lib.rs:830–838 (9 lines)


