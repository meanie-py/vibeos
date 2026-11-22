//! A lock the kernel takes as an in-stack QUEUED spinlock must never be taken or
//! released with the plain spinlock calls. `KTHREAD.ApcQueueLock` and
//! `KPROCESS.ProcessLock` are queued everywhere the kernel touches them
//! (`KiAcquireApcLock*`, `KiAcquireProcessLock*`); a plain `KeAcquireSpinLock` on
//! the same word mixes two protocols. On one CPU both only raise IRQL, so nothing
//! shows; on several, the plain release zeroes a queued waiter's tail and that
//! waiter's own release then waits forever for a successor. iomgr's stranded-
//! completion check did this and wedged the amd64 Chrome payload under -smp 4 in
//! 3 of 3 boots (one CPU spinning in KiReleaseApcLockFromSynchLevel, the rest idle).
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct QueuedLockPlainAcquire;

const PLAIN: &str = r"\b(?:SpinLockGuard|DpcSpinLockGuard)::acquire\s*\(|\b(?:KeAcquireSpinLock|KeAcquireSpinLockAtDpcLevel|KeAcquireSpinLockRaiseToDpc|KeTryToAcquireSpinLockAtDpcLevel|KeReleaseSpinLock|KeReleaseSpinLockFromDpcLevel|KxAcquireSpinLock|KxReleaseSpinLock)\s*\(";
const QUEUED: &str = r"\b(ApcQueueLock|ProcessLock)\b";

#[cfg(test)]
use crate::lexical::strip_noise as blank_noise;
#[cfg(test)]
fn scan(text: &str) -> Vec<(usize, String)> {
    scan_code(&blank_noise(text))
}

/// Each plain call's argument list (to its balancing `)`) is checked for a
/// queued-lock field name. Returns (1-based line, lock name).
fn scan_code(code: &str) -> Vec<(usize, String)> {
    static PLAIN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(PLAIN).unwrap());
    static QUEUED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(QUEUED).unwrap());
    let bytes = code.as_bytes();
    let mut out = vec![];
    for m in PLAIN_RE.find_iter(code) {
        // A definition (`fn KeAcquireSpinLock(`) or an extern declaration is not a call.
        let before = code[..m.start()].trim_end();
        if before.ends_with("fn") {
            continue;
        }
        let mut depth = 1i32;
        let mut end = m.end();
        while end < bytes.len() && depth > 0 {
            match bytes[end] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        if let Some(q) = QUEUED_RE.captures(&code[m.end()..end]) {
            let line = code[..m.start()].matches('\n').count() + 1;
            out.push((line, q[1].to_string()));
        }
    }
    out
}

impl Rule for QueuedLockPlainAcquire {
    fn name(&self) -> &'static str {
        "scan_queued_lock_plain_acquire"
    }
    fn needs_sdk_rust(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        Ok(c.sdk_rust_files()
            .flat_map(|f| {
                scan_code(&f.rust_noise()).into_iter().map(move |(n, lock)| {
                    format!(
                        "{}:{}: QUEUED_LOCK_PLAIN_ACQUIRE `{lock}` is an in-stack queued lock; a plain spinlock call on it breaks every queued holder on SMP. Use the KiAcquire*/KiRelease* queued helpers (e.g. ntkrnl_sys::inline::ke::KiAcquireApcLockRaiseToDpc / KiReleaseApcLock).",
                        f.path.display(),
                        n
                    )
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guard_on_apc_queue_lock_fires() {
        let src = "    let _lock = ntkrnl_sys::guards::SpinLockGuard::acquire((&raw mut (*tcb).ApcQueueLock).cast());\n";
        assert_eq!(scan(src), vec![(1, "ApcQueueLock".to_string())]);
    }
    #[test]
    fn multiline_ke_acquire_on_process_lock_fires() {
        let src = "fn f() {\n    KeAcquireSpinLock(\n        &mut (*process).ProcessLock,\n        &mut irql,\n    );\n}\n";
        assert_eq!(scan(src), vec![(2, "ProcessLock".to_string())]);
    }
    #[test]
    fn queued_helpers_do_not_fire() {
        let src = "    KiAcquireApcLockRaiseToDpc(tcb, &mut handle);\n    KeAcquireInStackQueuedSpinLock(addr_of_mut!((*t).ApcQueueLock) as *mut _, h);\n";
        assert!(scan(src).is_empty());
    }
    #[test]
    fn plain_lock_on_other_fields_does_not_fire() {
        let src = "    let _g = SpinLockGuard::acquire(&mut (*ext).QueueLock);\n    KeReleaseSpinLock(&mut DeviceLock, irql);\n";
        assert!(scan(src).is_empty());
    }
    #[test]
    fn comments_and_definitions_do_not_fire() {
        let src = "// SpinLockGuard::acquire(&mut t.ApcQueueLock) was the bug\npub unsafe extern \"system\" fn KeAcquireSpinLock(SpinLock: PKSPIN_LOCK, ApcQueueLock: u8) {}\n";
        assert!(scan(src).is_empty());
    }
}
