// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>
//
// gsm-dylint — NIGHTLY Tier-1 frontend (dylint lint).  STATUS: SCAFFOLD.
//
// Runs on ordinary Cargo crates via `cargo dylint`. A LateLintPass gets access
// to `cx.tcx`, from which we drive gsm-mir's lowering and gsm-core's analysis.
// Builds only on the pinned nightly; excluded from the stable workspace.
//
// Skeleton (fill in against the dylint template on the pinned nightly):
//
//   dylint_linting::declare_late_lint! {
//       pub GUARDSTATE_MIR, Warn,
//       "guard held at a forbidden checkpoint (block-in-atomic / await-holding-guard)"
//   }
//
//   impl<'tcx> LateLintPass<'tcx> for GuardstateMir {
//       fn check_crate(&mut self, cx: &LateContext<'tcx>) {
//           let kb = gsm_mir::default_guards();
//           let forbidden = /* load knowledge base */ Default::default();
//           for def_id in cx.tcx.hir().body_owners() {
//               if let Some(func) = gsm_mir::lower_body(cx.tcx, def_id, &kb, &forbidden) {
//                   for v in gsm_core::config::check(
//                       &gsm_core::analysis::analyze(&func),
//                       &gsm_core::config::Detector::block_in_atomic(),
//                   ) {
//                       // cx.span_lint(GUARDSTATE_MIR, span_of(&v), "...");
//                   }
//               }
//           }
//       }
//   }
