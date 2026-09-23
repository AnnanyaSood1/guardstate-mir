// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>
//
// gsm-mir — NIGHTLY, rustc_private. Lowers real rustc MIR into gsm-core IR.
//
// ============================ STATUS: SCAFFOLD ============================
// This crate compiles ONLY on the pinned nightly (see ../../rust-toolchain.toml)
// with the `rustc-dev` component, and is EXCLUDED from the stable workspace.
// The rustc_private API is unstable: the query/method names below are the design
// targets and MUST be verified against the pinned nightly (see DESIGN.md §7,§12).
// Expect an iterative compile-fix loop — this is the fragile crate by design.
// =========================================================================
#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use gsm_core::ir::{Function as GsmFn, GuardClass};

/// Knowledge base: which ADT def-paths are guards, and their class + return shape.
/// (Design §9: entries need return-shape so acquisition is recognized without
/// walking backward from the caller.)
#[derive(Clone, Copy)]
pub enum ReturnShape { Bare, Result, Option }

pub struct GuardEntry {
    pub def_path: &'static str,
    pub class: GuardClass,
    pub shape: ReturnShape,
}

/// Default std/RfL guard knowledge base (extend as needed).
pub fn default_guards() -> Vec<GuardEntry> {
    vec![
        GuardEntry { def_path: "std::sync::MutexGuard",        class: GuardClass::Sleepable,        shape: ReturnShape::Result },
        GuardEntry { def_path: "std::sync::RwLockReadGuard",   class: GuardClass::Sleepable,        shape: ReturnShape::Result },
        GuardEntry { def_path: "std::sync::RwLockWriteGuard",  class: GuardClass::Sleepable,        shape: ReturnShape::Result },
        GuardEntry { def_path: "parking_lot::MutexGuard",      class: GuardClass::Sleepable,        shape: ReturnShape::Bare },
        // CLSC configuration (P2b): RfL Guard parameterized by a preempt-disabling backend.
        GuardEntry { def_path: "kernel::sync::lock::Guard",    class: GuardClass::PreemptDisabling, shape: ReturnShape::Bare },
    ]
}

// ---------------------------------------------------------------------------
// The lowering entry point. Given a TyCtxt and a LocalDefId, build a gsm-core
// Function. Sketched against the design; each `rustc` call is annotated with
// what to verify. Types are left as `todo!()` bodies so the intent is explicit
// and the compile-fix loop has clear anchors.
// ---------------------------------------------------------------------------
//
// pub fn lower_body<'tcx>(
//     tcx: rustc_middle::ty::TyCtxt<'tcx>,
//     def_id: rustc_span::def_id::LocalDefId,
//     kb: &[GuardEntry],
//     forbidden: &std::collections::BTreeSet<String>,
// ) -> Option<GsmFn> {
//     // §7.1  Obtain drop-elaborated MIR. VERIFY on day 1 which query is correct
//     //       and stealable on the pinned nightly:
//     //         let body = tcx.mir_drops_elaborated_and_const_checked(def_id).borrow();
//     //       or, to also suppress inlining, compile at -Zmir-opt-level=0 and use
//     //         let body = tcx.optimized_mir(def_id.to_def_id());
//     //       Drop elaboration runs regardless of opt level (it is mandatory);
//     //       opt-level=0 only suppresses inlining/const-prop.
//     //
//     //   for (bb, data) in body.basic_blocks.iter_enumerated() {
//     //       // statements: StorageDead(local) -> Stmt::Kill; guard-producing
//     //       //   calls (below) -> Stmt::Acquire in the successor block.
//     //       // terminator:
//     //       //   TerminatorKind::Call { func, destination, target, .. }:
//     //       //     - resolve func Operand -> FnDef(def_id, args): tcx.def_path_str(def_id)
//     //       //     - is_foreign: tcx.is_foreign_item(def_id)
//     //       //     - guard-producing? inspect destination local's Ty (§7.2):
//     //       //         Ty -> TyKind::Adt(adt_def, substs); adt_def.did() -> def-path;
//     //       //         match kb; class from the backend type parameter.
//     //       //     - forbidden? def_path in `forbidden`.
//     //       //   TerminatorKind::Drop { place, target, .. } -> Term::Drop
//     //       //   TerminatorKind::SwitchInt { targets, .. } -> Term::Branch
//     //       //     (recognize try_lock: a guard-producing call returning
//     //       //      Result/Option followed by this SwitchInt -> activate the
//     //       //      guard on the success edge only; §7.4)
//     //       //   TerminatorKind::Goto { target } -> Term::Branch [one succ]
//     //       //   Coroutine bodies: TerminatorKind::Yield -> Term::Suspend, BUT
//     //       //     see §7.3 — "guard live across await" needs the coroutine's
//     //       //     SAVED-LOCAL set at the suspend point, not a plain-CFG query.
//     //   }
//     todo!("verify rustc_private API on the pinned nightly, then implement per DESIGN.md §7")
// }
