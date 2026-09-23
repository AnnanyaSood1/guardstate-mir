// SPDX-License-Identifier: GPL-2.0
// Fixture: D-BLOCK negative. Guard dropped before the blocking call.
// Expected: no violation (drop-point precision).
use std::sync::Mutex;
use std::time::Duration;

static M: Mutex<u32> = Mutex::new(0);

pub fn good() {
    {
        let _g = M.lock().unwrap();
    } // guard dropped here
    std::thread::sleep(Duration::from_millis(1)); // not held -> ok
}
