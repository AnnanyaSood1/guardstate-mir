// SPDX-License-Identifier: GPL-2.0
// Fixture: D-AWAIT negative (stretch). Guard dropped before the .await.
// Expected: no violation.
use std::sync::Mutex;

static M: Mutex<u32> = Mutex::new(0);

async fn yield_now() {}

pub async fn good() {
    {
        let _g = M.lock().unwrap();
    } // dropped before suspend
    yield_now().await; // not held -> ok
}
