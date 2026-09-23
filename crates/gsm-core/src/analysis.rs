// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>

//! Guard-liveness typestate analysis over the abstract CFG.
//!
//! Lattice: the abstract state at a program point is the set of live guard
//! locals; the ordering is `Clear ⊑ GuardHeld` (a point is "held" iff the set is
//! non-empty). Join is set union at control-flow merges (may-held), which is the
//! sound direction for a *detector*: a guard live on any path is treated as live.
//!
//! The analysis is a forward dataflow to a least fixpoint. It emits a
//! **checkpoint** record at every forbidden `Call` and every `Suspend`, carrying
//! the set of guards live at that point. The checker (`check.rs`) turns
//! checkpoints into violations for a given detector.

use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet};

/// The two kinds of program point at which the restriction is tested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointKind {
    /// A call to a forbidden operation.
    Call { def_path: String },
    /// An await / coroutine suspend point.
    Suspend,
}

/// A recorded checkpoint and the guards live there.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub func: String,
    pub bb: BasicBlock,
    pub kind: CheckpointKind,
    /// Guards live at the checkpoint, with their class, deterministically ordered.
    pub held: Vec<(Local, GuardClass)>,
}

/// Collect the class of every guard local mentioned in the function.
fn guard_classes(f: &Function) -> BTreeMap<Local, GuardClass> {
    let mut m = BTreeMap::new();
    for id in &f.order {
        let b = &f.blocks[id];
        for s in &b.stmts {
            if let Stmt::Acquire { local, class } = s {
                m.insert(*local, *class);
            }
        }
        if let Term::Branch { succs } = &b.term {
            for s in succs {
                if let Some((local, class)) = s.activate {
                    m.insert(local, class);
                }
            }
        }
    }
    m
}

/// Apply a block's statements to the working set.
fn apply_stmts(b: &Block, set: &mut BTreeSet<Local>) {
    for s in &b.stmts {
        match s {
            Stmt::Acquire { local, .. } => {
                set.insert(*local);
            }
            Stmt::Kill { local } => {
                set.remove(local);
            }
        }
    }
}

/// Compute the outgoing set along each successor edge from a block, given the
/// working set after the block's statements have been applied.
fn edge_outputs(term: &Term, after_stmts: &BTreeSet<Local>) -> Vec<(BasicBlock, BTreeSet<Local>)> {
    match term {
        Term::Call { target, .. } => vec![(*target, after_stmts.clone())],
        Term::Suspend { target } => vec![(*target, after_stmts.clone())],
        Term::Drop { local, target } => {
            let mut s = after_stmts.clone();
            s.remove(local);
            vec![(*target, s)]
        }
        Term::Branch { succs } => succs
            .iter()
            .map(|succ| {
                let mut s = after_stmts.clone();
                if let Some((local, _)) = succ.activate {
                    s.insert(local);
                }
                (succ.target, s)
            })
            .collect(),
        Term::Return => vec![],
    }
}

/// Forward fixpoint: returns the in-state (guards live on entry) of each block.
fn fixpoint(f: &Function) -> BTreeMap<BasicBlock, BTreeSet<Local>> {
    let mut in_state: BTreeMap<BasicBlock, BTreeSet<Local>> =
        f.order.iter().map(|id| (*id, BTreeSet::new())).collect();

    let mut changed = true;
    while changed {
        changed = false;
        let mut next: BTreeMap<BasicBlock, BTreeSet<Local>> =
            f.order.iter().map(|id| (*id, BTreeSet::new())).collect();

        for id in &f.order {
            let b = &f.blocks[id];
            let mut cur = in_state[id].clone();
            apply_stmts(b, &mut cur);
            for (target, set) in edge_outputs(&b.term, &cur) {
                if let Some(dst) = next.get_mut(&target) {
                    dst.extend(set);
                }
            }
        }

        for id in &f.order {
            // Entry has no predecessors: always empty on entry.
            let merged = if *id == f.entry { BTreeSet::new() } else { next[id].clone() };
            if merged != in_state[id] {
                in_state.insert(*id, merged);
                changed = true;
            }
        }
    }
    in_state
}

/// Analyze one function and return its checkpoints (deterministically ordered).
pub fn analyze(f: &Function) -> Vec<Checkpoint> {
    let classes = guard_classes(f);
    let in_state = fixpoint(f);
    let mut out = Vec::new();

    for id in &f.order {
        let b = &f.blocks[id];
        let mut cur = in_state[id].clone();
        apply_stmts(b, &mut cur);
        let held: Vec<(Local, GuardClass)> = cur
            .iter()
            .map(|l| (*l, *classes.get(l).unwrap_or(&GuardClass::Sleepable)))
            .collect();
        match &b.term {
            Term::Call { callee, .. } if callee.is_forbidden => out.push(Checkpoint {
                func: f.name.clone(),
                bb: *id,
                kind: CheckpointKind::Call { def_path: callee.def_path.clone() },
                held,
            }),
            Term::Suspend { .. } => out.push(Checkpoint {
                func: f.name.clone(),
                bb: *id,
                kind: CheckpointKind::Suspend,
                held,
            }),
            _ => {}
        }
    }
    out.sort_by(|a, b| (a.func.as_str(), a.bb.0).cmp(&(b.func.as_str(), b.bb.0)));
    out
}
