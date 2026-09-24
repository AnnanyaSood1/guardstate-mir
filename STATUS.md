# Status

*Last updated: 2026-09-24*

Single source of truth for what works, what doesn't, and what's next.
Updated weekly on Fridays.

## What works today

- **`gsm-core`** (stable Rust) — abstract CFG, guard-liveness typestate
  lattice, checker rule. Unit-tested in isolation.
- **`gsm-text`** (stable Rust) — text-format `.mir` frontend and CLI. Lowers
  into `gsm-core` and runs the analysis.
- **9 golden fixtures** — seven ported from `guardstate` plus two async-shape
  suspend checkpoints. All passing. Exercise the lattice and the checker, not
  MIR ingestion.
- **CI** — builds and tests the stable workspace on every push.

## What does not work yet

- **Real `rustc` MIR is not read by anything in this repository.** `gsm-mir`
  is a scaffold: modules and types are declared, entry points compile against
  the pinned nightly, but the lowering functions carry `todo!()` bodies. The
  Tier-1 `gsm-dylint` lint depends on `gsm-mir` and is in the same state.
- No real `.rs` fixtures are analyzed end-to-end.
- No coroutine / async MIR analysis (D-AWAIT); the two `suspend` golden
  fixtures exercise the checkpoint rule at the lattice level only.

## In progress this week

Tracked as GitHub issues under the `P1a` label:

- **#1** Verify the MIR-query choice on the pinned nightly.
- **#2** MIR-walking hello-world: read one `.rs` fixture, print resolved
  callee def-paths and destination types.

Once #1 and #2 land, the next tranche (#3–#5) implements the guard-by-type
classifier and ports the lattice as a `rustc_mir_dataflow::Analysis`.

## Roadmap phases

- **P0** — workspace, stable core, text tests. ✅ complete.
- **P1a** — `gsm-mir` reads real MIR; D-BLOCK detector on real `.rs`
  fixtures via `gsm-dylint`. **In progress.**
- **P1b** — D-AWAIT via coroutine saved-local analysis. *Attempted;
  outcome reported honestly whichever way it goes.*
- **P2a** — `gsm-driver` (rustc wrapper) for non-cargo builds.
- **P2b** — CLSC configuration: RfL guard backends + may-sleep primitives.

## How to read this repository right now

- If you want to see whether the **lattice logic** works: run
  `./run_tests.sh` and read `crates/gsm-core/src/analysis.rs`.
- If you want to see whether **real MIR is ingested**: not yet — see
  `crates/gsm-mir/` for the scaffolding and the linked issues for the
  active work.
- If you want to see the **design**: `DESIGN.md`.
