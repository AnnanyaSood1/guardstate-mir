// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>

//! Parser for the line-oriented, MIR-shaped text format, lowering into
//! `gsm-core::ir`. This frontend exists to unit-test the core lattice on stable
//! Rust without any nightly toolchain; it is the text analogue of what `gsm-mir`
//! does with real rustc MIR.
//!
//! Grammar (`#` starts a comment):
//!
//!   fn NAME
//!   bbN:
//!     LOCAL = acquire KIND         -- unconditional guard acquire (statement)
//!     drop LOCAL                   -- guard drop / StorageDead (statement)
//!     storage_dead LOCAL           -- same as drop
//!     call SYMBOL -> bbN           -- call terminator (checkpoint iff forbidden)
//!     suspend -> bbN               -- await / coroutine suspend (always checkpoint)
//!     drop_call LOCAL -> bbN       -- core::mem::drop(LOCAL): release then continue
//!     forget_call LOCAL -> bbN     -- core::mem::forget(LOCAL): does NOT release
//!     try LOCAL = KIND -> succ fail -- conditional acquire; guard live on success
//!     goto bbN
//!     ret
//!
//! KIND classifies the guard: spin_lock family -> PreemptDisabling; else Sleepable.

use gsm_core::ir::*;
use std::collections::{BTreeMap, BTreeSet};

fn class_of(kind: &str) -> GuardClass {
    match kind {
        "spin_lock" | "raw_spin_lock" | "spin_lock_irqsave" | "spin_lock_bh" | "local_irq_save"
        | "preempt_disable" => GuardClass::PreemptDisabling,
        _ => GuardClass::Sleepable,
    }
}

/// Intern block/local names to indices, preserving first-seen order for blocks.
#[derive(Default)]
struct Interner {
    locals: BTreeMap<String, u32>,
    blocks: BTreeMap<String, u32>,
    next_local: u32,
    next_block: u32,
}
impl Interner {
    fn local(&mut self, name: &str) -> Local {
        if let Some(i) = self.locals.get(name) {
            return Local(*i);
        }
        let i = self.next_local;
        self.next_local += 1;
        self.locals.insert(name.to_string(), i);
        Local(i)
    }
    fn block(&mut self, name: &str) -> BasicBlock {
        if let Some(i) = self.blocks.get(name) {
            return BasicBlock(*i);
        }
        let i = self.next_block;
        self.next_block += 1;
        self.blocks.insert(name.to_string(), i);
        BasicBlock(i)
    }
}

/// Parse a module of functions. `forbidden` is the detector knowledge base: any
/// call symbol in this set is marked `is_forbidden` on its `Callee`.
#[allow(unused_assignments)]
pub fn parse_module(src: &str, forbidden: &BTreeSet<String>) -> Result<Vec<Function>, String> {
    let mut funcs = Vec::new();
    let mut name: Option<String> = None;
    let mut intern = Interner::default();
    let mut blocks: BTreeMap<BasicBlock, Block> = BTreeMap::new();
    let mut order: Vec<BasicBlock> = Vec::new();
    let mut entry: Option<BasicBlock> = None;
    let mut cur: Option<(BasicBlock, Vec<Stmt>, Option<Term>)> = None;

    macro_rules! flush_block {
        () => {
            if let Some((id, stmts, term)) = cur.take() {
                blocks.insert(id, Block { id, stmts, term: term.unwrap_or(Term::Return) });
            }
        };
    }
    macro_rules! flush_fn {
        () => {
            if let Some(n) = name.take() {
                flush_block!();
                funcs.push(Function {
                    name: n,
                    entry: entry.take().unwrap_or(BasicBlock(0)),
                    blocks: std::mem::take(&mut blocks),
                    order: std::mem::take(&mut order),
                });
                intern = Interner::default();
            }
        };
    }

    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        let err = |m: &str| format!("line {}: {} (`{}`)", lineno + 1, m, line);

        if t[0] == "fn" {
            flush_fn!();
            name = Some(t.get(1).ok_or_else(|| err("missing function name"))?.to_string());
            continue;
        }
        if name.is_none() {
            return Err(err("statement outside a function"));
        }
        if line.ends_with(':') {
            flush_block!();
            let id = intern.block(line.trim_end_matches(':'));
            if entry.is_none() {
                entry = Some(id);
            }
            order.push(id);
            cur = Some((id, Vec::new(), None));
            continue;
        }
        let blk = cur.as_mut().ok_or_else(|| err("statement outside a block"))?;

        // LOCAL = acquire KIND
        if t.len() >= 4 && t[1] == "=" && t[2] == "acquire" {
            let local = intern.local(t[0]);
            blk.1.push(Stmt::Acquire { local, class: class_of(t[3]) });
            continue;
        }

        match t[0] {
            "drop" | "storage_dead" => {
                let local = intern.local(t[1]);
                blk.1.push(Stmt::Kill { local });
            }
            "goto" => {
                let target = intern.block(t[1]);
                blk.2 = Some(Term::Branch { succs: vec![Successor::plain(target)] });
            }
            "ret" => blk.2 = Some(Term::Return),
            "suspend" => {
                let arrow = t.iter().position(|x| *x == "->").ok_or_else(|| err("suspend needs ->"))?;
                let target = intern.block(t[arrow + 1]);
                blk.2 = Some(Term::Suspend { target });
            }
            "call" => {
                let arrow = t.iter().position(|x| *x == "->").ok_or_else(|| err("call needs ->"))?;
                let sym = t[1..arrow].join(" ");
                let is_forbidden = forbidden.contains(&sym);
                let target = intern.block(t[arrow + 1]);
                blk.2 = Some(Term::Call {
                    callee: Callee { def_path: sym, is_foreign: true, is_forbidden },
                    target,
                });
            }
            "drop_call" => {
                let arrow = t.iter().position(|x| *x == "->").ok_or_else(|| err("drop_call needs ->"))?;
                let local = intern.local(t[1]);
                let target = intern.block(t[arrow + 1]);
                blk.2 = Some(Term::Drop { local, target });
            }
            "forget_call" => {
                // mem::forget: a call that does NOT release the guard (conservative).
                let arrow = t.iter().position(|x| *x == "->").ok_or_else(|| err("forget_call needs ->"))?;
                let target = intern.block(t[arrow + 1]);
                blk.2 = Some(Term::Call {
                    callee: Callee {
                        def_path: "core::mem::forget".into(),
                        is_foreign: false,
                        is_forbidden: false,
                    },
                    target,
                });
            }
            "try" => {
                // try LOCAL = KIND -> succ fail  (guard live only on success edge)
                if t.get(2) != Some(&"=") {
                    return Err(err("try syntax: try LOCAL = KIND -> succ fail"));
                }
                let arrow = t.iter().position(|x| *x == "->").ok_or_else(|| err("try needs ->"))?;
                let local = intern.local(t[1]);
                let class = class_of(t[3]);
                let succ = intern.block(t[arrow + 1]);
                let fail = intern.block(*t.get(arrow + 2).ok_or_else(|| err("try needs fail target"))?);
                blk.2 = Some(Term::Branch {
                    succs: vec![
                        Successor { target: succ, activate: Some((local, class)) },
                        Successor { target: fail, activate: None },
                    ],
                });
            }
            _ => return Err(err("unrecognised statement")),
        }
    }
    flush_fn!();
    Ok(funcs)
}
