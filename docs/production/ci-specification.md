# CI Specification (GOV-05)

**Status:** decided under delegation, DEC-073 in `docs/decisions/decision-registry.md`. Tool pins follow DEC-068. The running implementation is `.github/workflows/ci.yml` (GOV-06).

## 1. Required checks

| Check (job) | Command | Pass condition | Owner |
|---|---|---|---|
| `fmt` | `cargo fmt --all -- --check` | no diff | Agent I |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | no warnings | Agent I |
| `test (ubuntu-latest)`, `test (windows-latest)` | `cargo test --workspace` | all tests pass | Agent I |
| `deny` | `cargo deny check` (action `EmbarkStudios/cargo-deny-action@v2`) | licences within the allow-list of `deny.toml`, no yanked crates, only crates.io | Agent I |
| `archcheck` | `cargo xtask archcheck` | crate graph equals the ADG Part B edges; no forbidden `use` or token; self-tests of seeded violations pass (`cargo test -p xtask`) | Integration Owner |

Later additions (not required yet): public-API snapshots of `cube-core` and `cube-editing-api`, contract-id check, registry lint.

## 2. Tool pins

* Toolchain: `rust-toolchain.toml`, channel `1.97.1`, components `rustfmt`, `clippy`; edition 2024; no async runtime in Core, Processing, Editing, backends.
* Hosts: `ubuntu-latest` for fmt, clippy, deny, archcheck; `ubuntu-latest` and `windows-latest` for tests.
* Licence allow-list: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib, MPL-2.0. The project's own licence is DEFERRED (DEC-070); workspace crates are `publish = false`.

## 3. Evidence rules for GOV-06

1. A workflow run on the skeleton shows every job green.
2. Seeded failures show red for each check: a format error (`fmt`), a clippy warning (`clippy`), a failing test (`test`), a forbidden dependency (`archcheck`, also covered by the xtask self-tests), a forbidden licence (`deny`).
3. Branch protection lists the checks above as required status checks. **Enabling it is a GitHub administration act**; agents have no GitHub API access in the current environment, so this evidence needs the repository owner (see the PS-0 status in `ps-0-exit-checklist.md`).

## 4. Branches

`master` is the release branch; `dev` is the integration branch; feature branches are `prod/ps<stage>-<topic>` or `docs/<topic>` from `dev` (DEC-003). CI runs on every push and pull request.
