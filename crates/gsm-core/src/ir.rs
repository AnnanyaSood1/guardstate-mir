// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>

//! The abstract control-flow graph that every frontend lowers into.
//!
//! This is the stable, rustc-independent lowering target. Real MIR (via
//! `gsm-mir`) and the text frontend (`gsm-text`) both produce this IR, and the
//! analysis and checker consume it. Keeping it small and generic is what lets
//! the fragile rustc-facing code live in exactly one crate.
//!
//! Design note (improvement over `guardstate`): there is no bespoke
//! `TryAcquire` node. Unconditional acquisition is a `Stmt::Acquire`; conditional
//! acquisition (`try_lock`) is expressed by an *edge activation* on a general
//! `Branch` — the lowering recognizes the pattern and marks which successor edge
//! makes the guard live. Cleverness stays in the lowering; the core stays generic.

use std::collections::BTreeMap;

/// A MIR-style local slot (interned to an index by each frontend).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local(pub u32);

/// A basic-block identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasicBlock(pub u32);

/// How a guard relates to the restricted context a detector cares about.
///
/// Extensible: a detector selects which classes "count" (see `gsm-core::config`).
/// For the sleep-in-atomic configuration, only `PreemptDisabling` counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuardClass {
    /// spinlock family, raw spinlocks, IRQ/preempt-disabling guards.
    PreemptDisabling,
    /// mutex / rwsem style: sleepable, does not forbid blocking.
    Sleepable,
}

/// A resolved call target. Frontends fill this in so the analysis never sees a
/// rustc `Ty` or `DefId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Callee {
    pub def_path: String,
    pub is_foreign: bool,
    /// True when this callee is a forbidden operation for the active detector.
    pub is_forbidden: bool,
}

/// A control-flow successor edge, optionally *activating* a guard on that edge.
/// A `try_lock`'s success edge carries `activate = Some((guard, class))`; its
/// failure edge carries `None`. A plain `goto`/`switch` edge carries `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Successor {
    pub target: BasicBlock,
    pub activate: Option<(Local, GuardClass)>,
}

impl Successor {
    pub fn plain(target: BasicBlock) -> Successor {
        Successor { target, activate: None }
    }
}

/// Straight-line statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// Unconditional guard acquisition: after this point `local` holds a guard.
    Acquire { local: Local, class: GuardClass },
    /// `StorageDead(local)` or a drop-as-statement: the guard is released.
    Kill { local: Local },
}

/// Block terminators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// A call. A **checkpoint** iff `callee.is_forbidden`.
    Call { callee: Callee, target: BasicBlock },
    /// An `.await` / coroutine suspend point. **Always a checkpoint.**
    Suspend { target: BasicBlock },
    /// A drop-as-terminator: releases `local`, then continues.
    Drop { local: Local, target: BasicBlock },
    /// Generalized control flow: `goto` (one succ), `switch` (N succs), and the
    /// recognized `try_lock` (success succ activates the guard).
    Branch { succs: Vec<Successor> },
    /// Return / no successor.
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: BasicBlock,
    pub stmts: Vec<Stmt>,
    pub term: Term,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub entry: BasicBlock,
    pub blocks: BTreeMap<BasicBlock, Block>,
    /// Source order of blocks, for deterministic iteration.
    pub order: Vec<BasicBlock>,
}

impl Function {
    pub fn successors(&self, term: &Term) -> Vec<Successor> {
        match term {
            Term::Call { target, .. } => vec![Successor::plain(*target)],
            Term::Suspend { target } => vec![Successor::plain(*target)],
            Term::Drop { target, .. } => vec![Successor::plain(*target)],
            Term::Branch { succs } => succs.clone(),
            Term::Return => vec![],
        }
    }
}
