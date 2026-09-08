# surgeist-shape Repository Guide

Use `$pisct:coordination` for standalone delivery and the smallest focused
`$pisct:<skill>` for focused work. Use `$pisct:plane-coordination` only when the
user explicitly selects plane coordination. This guide supplies repository facts;
it does not grant mutation, installation, commit, publication, or cross-repository
authority. Higher-priority user and system instructions still apply.

## Authority Split

This file is the repository's committed discovery entry point. It owns the
mapping from mutable repository facts to their authoritative sources, the
product boundary, the configured command inventory, and the standalone workflow
selection. PISCT skills supply reusable coordination and engineering guidance.

Use the sources below for current repository facts and the task-appropriate PISCT
skill for workflow. Do not copy skill policy into repository documentation or
infer additional authority from a review result.

## Repository Identity And Ownership

`surgeist-shape` is an independent leaf repository. It owns its manifest,
resolved geometry and shape implementation, public front door, focused tests,
documentation, commits, and published `main` candidate. Repository placement in a
parent workspace, project, task, branch, or worktree does not transfer ownership.

Root `surgeist` owns the facade and public composition, cross-crate adapters,
integration tests and tools, this leaf's gitlink, and the API generator and its
artifacts. Scope changes at that boundary explicitly with the root owner.

## Discover The Current Structure

Read these sources for current facts instead of relying on a cached inventory.

| Fact | Authoritative source |
| --- | --- |
| Package identity, edition, dependencies, features, and targets | `Cargo.toml` |
| Public front door and exported surface | `src/lib.rs` and its reexports |
| Adoption and documentation navigation | `README.md` and the four guides under `docs/` |
| Behavior and domain invariants | Implementation under `src/` and focused assertions in `src/tests.rs` |
| Focused verification | `src/tests.rs`, registered by `src/lib.rs` |
| Local verification commands | The command inventory below and Cargo targets in `Cargo.toml` |
| Project licensing | `LICENSE` and the license declaration in `Cargo.toml` |
| Dependency attribution and included license material | `NOTICE.md` and its linked files under `licenses/`; compare with exact upstream versions when dependencies change |
| Integration MSRV, authoritative URL, and compatible pin | Root `surgeist`'s `Cargo.toml`, `.gitmodules`, and committed gitlink when root integration is in scope |

The leaf manifest declares no feature switches or `rust-version`. `Cargo.lock`
is ignored by `.gitignore`; a local resolution is not a committed dependency pin
for consumers. Inspect any newly added CI, task runner, or toolchain configuration
when it becomes relevant rather than assuming this inventory is exhaustive.

When sources disagree, report exact paths and revisions. Do not guess, silently
rewrite another authority, or widen the task to reconcile them.

## Product Boundary

`surgeist-shape` owns resolved geometry, primitive path data, shape normalization,
bounds, containment, path conversion, geometry keys, stroke geometry, and dash
geometry. Constructors and validated transitions own their domain invariants.

Style resolution, layout, GPU and render resources, widgets, and application
behavior belong outside this leaf. Surgeist-to-Surgeist lowering and adapters
belong to root; sibling internals are not this repository's public surface.

For work involving another repository, resolve its ownership from current
committed policy and source. Inspecting another repository never grants write
authority there.

## Generated Artifacts

Source in this repository is authoritative. Root `surgeist` owns the only API
generator and all generated audit artifacts; this leaf carries no copies.
Resolve refresh and verification commands in root when an authorized task affects
those artifacts. Do not hand-edit generated output or recreate a leaf generator.

## Command Inventory

These commands describe local verification capability. The assigned scope and
PISCT skill guidance select the checks; use `$pisct:process` for authorized command
execution with already-present tooling.

```sh
cargo check -p surgeist-shape
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
```

The Clippy command forbids unsafe code and denies warnings. Cargo check, test,
and Clippy support `--offline --locked` when dependencies and the local lockfile
are already available. Command availability does not authorize software
acquisition.

Discovery is complete when ownership, product boundary, public entry points,
dependency facts, generated-artifact policy, verification sources, and the
applicable command inventory are established from current source.
