// SPDX-License-Identifier: GPL-2.0
// Fixture: D-AWAIT positive (stretch). A guard held across an .await.
// Expected (if D-AWAIT is implemented): violation at the suspend point.
// NOTE: requires coroutine saved-local analysis (DESIGN.md §7.3).
use std::sync::Mutex;

static M: Mutex<u32> = Mutex::new(0);

async fn yield_now() {}

pub async fn bad() {
    let _g = M.lock().unwrap();  // guard saved across suspend
    yield_now().await;           // <-- VIOLATION (await while holding lock)
}
