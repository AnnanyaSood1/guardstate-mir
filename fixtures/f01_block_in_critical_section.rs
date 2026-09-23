// SPDX-License-Identifier: GPL-2.0
// Fixture: D-BLOCK positive. A blocking call while a std Mutex guard is live.
// Expected: guardstate-mir reports a violation at the thread::sleep call.
use std::sync::Mutex;
use std::time::Duration;

static M: Mutex<u32> = Mutex::new(0);

pub fn bad() {
    let _g = M.lock().unwrap();          // guard live
    std::thread::sleep(Duration::from_millis(1)); // forbidden while held  <-- VIOLATION
}
