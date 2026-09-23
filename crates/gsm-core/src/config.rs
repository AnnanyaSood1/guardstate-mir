// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>

//! Detector configuration and the checkpoint rule.
//!
//! A detector is defined by which guard classes "count" (i.e. whose liveness at
//! a checkpoint constitutes a violation). Which callees are *forbidden* is baked
//! into each `Callee` by the frontend using the detector's knowledge base, so the
//! checker here only has to apply the class filter.

use crate::analysis::{Checkpoint, CheckpointKind};
use crate::ir::{BasicBlock, GuardClass};
use std::collections::BTreeSet;

/// A detector specification.
#[derive(Debug, Clone)]
pub struct Detector {
    pub name: String,
    /// Guard classes whose liveness at a checkpoint is a violation.
    pub counted: BTreeSet<GuardClass>,
}

impl Detector {
    /// The sleep-in-atomic configuration: only preemption-disabling guards count.
    pub fn block_in_atomic() -> Detector {
        let mut counted = BTreeSet::new();
        counted.insert(GuardClass::PreemptDisabling);
        Detector { name: "block-in-atomic".into(), counted }
    }

    /// A permissive configuration counting any lock guard (e.g. await-holding-lock).
    pub fn any_guard() -> Detector {
        let mut counted = BTreeSet::new();
        counted.insert(GuardClass::PreemptDisabling);
        counted.insert(GuardClass::Sleepable);
        Detector { name: "await-holding-guard".into(), counted }
    }
}

/// A reported violation.
#[derive(Debug, Clone)]
pub struct Violation {
    pub func: String,
    pub bb: BasicBlock,
    pub kind: CheckpointKind,
    /// The counted guard classes found live at the checkpoint.
    pub held: Vec<GuardClass>,
}

/// Apply a detector to a set of checkpoints, producing violations.
pub fn check(checkpoints: &[Checkpoint], det: &Detector) -> Vec<Violation> {
    let mut v: Vec<Violation> = checkpoints
        .iter()
        .filter_map(|c| {
            let held: Vec<GuardClass> = c
                .held
                .iter()
                .map(|(_, class)| *class)
                .filter(|class| det.counted.contains(class))
                .collect();
            if held.is_empty() {
                None
            } else {
                Some(Violation { func: c.func.clone(), bb: c.bb, kind: c.kind.clone(), held })
            }
        })
        .collect();
    v.sort_by(|a, b| (a.func.as_str(), a.bb.0).cmp(&(b.func.as_str(), b.bb.0)));
    v
}
