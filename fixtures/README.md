# Fixtures

Real Rust source used by the MIR integration tests (`gsm-dylint`, nightly).
Each fixture isolates one behaviour and documents its expected verdict.

Naming: `fNN_shape.rs`. Positive fixtures must produce a violation; negative
fixtures (dropped-before, non-forbidden) must not. Async fixtures (`await_*`)
exercise the D-AWAIT stretch detector and ship as known-limitation cases until
coroutine saved-local analysis lands (see ../DESIGN.md §7.3, §13).

Target: at least 10 fixtures — the seven `guardstate` shapes ported to real Rust
plus at least three async shapes. The four here are the seed set.
