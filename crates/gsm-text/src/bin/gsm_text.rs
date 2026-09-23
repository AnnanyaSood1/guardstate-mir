// SPDX-License-Identifier: GPL-2.0
// Author: Annanya Sood <annanyas0142@gmail.com>

//! gsm-text CLI: parse a MIR-shaped .mir file, run the guard-liveness analysis,
//! and report checkpoints and violations for a chosen detector.
//!
//! Usage:
//!   gsm-text <input.mir> [--forbidden <list.txt>] [--detector block-in-atomic|any-guard]
//!
//! The --forbidden file is the detector knowledge base: one callee symbol per
//! line (`#` comments allowed). It plays the role the C-side CanSleep summary
//! plays in the full CLSC design.

use gsm_core::analysis::{analyze, CheckpointKind};
use gsm_core::config::{check, Detector};
use std::collections::BTreeSet;
use std::process::exit;

fn load_set(path: &str) -> BTreeSet<String> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| { eprintln!("error: cannot read {}: {}", path, e); exit(2); });
    text.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <input.mir> [--forbidden <list.txt>] [--detector block-in-atomic|any-guard]", args[0]);
        exit(2);
    }
    let input = &args[1];
    let mut forbidden = BTreeSet::new();
    let mut detector = Detector::block_in_atomic();
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--forbidden" => {
                forbidden = load_set(args.get(i + 1).unwrap_or_else(|| { eprintln!("--forbidden needs a path"); exit(2); }));
                i += 2;
            }
            "--detector" => {
                detector = match args.get(i + 1).map(|s| s.as_str()) {
                    Some("block-in-atomic") => Detector::block_in_atomic(),
                    Some("any-guard") => Detector::any_guard(),
                    other => { eprintln!("unknown detector {:?}", other); exit(2); }
                };
                i += 2;
            }
            other => { eprintln!("unknown argument {}", other); exit(2); }
        }
    }

    let src = std::fs::read_to_string(input)
        .unwrap_or_else(|e| { eprintln!("error: cannot read {}: {}", input, e); exit(2); });
    let funcs = gsm_text::parse_module(&src, &forbidden)
        .unwrap_or_else(|e| { eprintln!("parse error: {}", e); exit(2); });

    let mut checkpoints = Vec::new();
    for f in &funcs {
        checkpoints.extend(analyze(f));
    }
    checkpoints.sort_by(|a, b| (a.func.as_str(), a.bb.0).cmp(&(b.func.as_str(), b.bb.0)));

    println!("== checkpoints ==");
    for c in &checkpoints {
        let what = match &c.kind {
            CheckpointKind::Call { def_path } => format!("call {}", def_path),
            CheckpointKind::Suspend => "suspend".to_string(),
        };
        let held = if c.held.is_empty() {
            "none".to_string()
        } else {
            c.held.iter().map(|(l, k)| format!("{:?}(_{})", k, l.0)).collect::<Vec<_>>().join(", ")
        };
        println!("{}::bb{}  {:<28} [held: {}]", c.func, c.bb.0, what, held);
    }

    let violations = check(&checkpoints, &detector);
    println!("\n== violations (detector: {}) ==", detector.name);
    if violations.is_empty() {
        println!("(none)");
    } else {
        for v in &violations {
            let what = match &v.kind {
                CheckpointKind::Call { def_path } => format!("call {}", def_path),
                CheckpointKind::Suspend => "suspend".to_string(),
            };
            let classes = v.held.iter().map(|k| format!("{:?}", k)).collect::<Vec<_>>().join("+");
            println!("VIOLATION {}::bb{} {} while holding {}", v.func, v.bb.0, what, classes);
        }
    }
    println!("\nsummary: {} checkpoint(s), {} violation(s)", checkpoints.len(), violations.len());
}
