// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>

//! gsm-core: the stable, rustc-independent heart of guardstate-mir.
//!
//! Contains the abstract CFG (`ir`), the guard-liveness typestate analysis
//! (`analysis`), and the detector/checker (`config`). This crate compiles on
//! stable Rust and is unit-tested without any nightly toolchain; the fragile
//! rustc-facing code lives entirely in `gsm-mir`.

pub mod analysis;
pub mod config;
pub mod ir;
