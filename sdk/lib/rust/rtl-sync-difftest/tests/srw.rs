//! Mixed shared/exclusive traffic on one shipping SRW lock. Checks:
//!   - exclusion: a writer never overlaps a reader or another writer, and a counter only the
//!     writers touch, without atomics, ends at exactly the number of exclusive sections;
//!   - liveness: a watchdog aborts if a round stops making progress (a lost wake hangs forever);
//!   - coverage: in the user-mode build the waits really blocked on the keyed event.
//! Each round runs once on every CPU and once with every thread pinned to CPU 0, the arm64 smoke
//! VM's shape, where a waiter can only make progress by giving the CPU up.

use rtl_sync_difftest::{srw, KEYED_RELEASES, KEYED_WAITS, RTL_SRWLOCK};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::time::{Duration, Instant};

static ROUNDS: AtomicU64 = AtomicU64::new(0);

struct Shared {
    lock: UnsafeCell<RTL_SRWLOCK>,
    readers: AtomicI64,
    writer: AtomicBool,
    plain: UnsafeCell<u64>,
    exclusive_sections: AtomicU64,
    progress: AtomicU64,
}
unsafe impl Sync for Shared {}

fn pin_to_cpu0() {
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(0, &mut set);
        assert_eq!(
            libc::sched_setaffinity(0, std::mem::size_of_val(&set), &set),
            0
        );
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Hold a section: usually a few spins, sometimes a yield, rarely a sleep (which is what makes
/// the other threads exhaust their spin budget and block).
fn hold(rng: &mut Rng) {
    match rng.next() % 64 {
        0 => std::thread::sleep(Duration::from_micros(300)),
        1..=4 => std::thread::yield_now(),
        _ => {
            for _ in 0..(rng.next() % 32) {
                std::hint::spin_loop();
            }
        }
    }
}

fn round(name: &str, threads: usize, iters: u64, pinned: bool) {
    let s: &'static Shared = Box::leak(Box::new(Shared {
        lock: UnsafeCell::new(RTL_SRWLOCK {
            Ptr: std::ptr::null_mut(),
        }),
        readers: AtomicI64::new(0),
        writer: AtomicBool::new(false),
        plain: UnsafeCell::new(0),
        exclusive_sections: AtomicU64::new(0),
        progress: AtomicU64::new(0),
    }));
    let waits0 = KEYED_WAITS.load(Ordering::Relaxed);
    let t0 = Instant::now();
    let done: &'static AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));
    let label = name.to_string();
    std::thread::spawn(move || {
        let mut last = 0;
        let trace = std::env::var_os("SRW_TRACE").is_some();
        loop {
            for _ in 0..20 {
                std::thread::sleep(Duration::from_secs(1));
                if trace {
                    eprintln!(
                        "srw {label}: {} sections, lock word {:#x}",
                        s.progress.load(Ordering::Relaxed),
                        unsafe { (*s.lock.get()).Ptr as usize }
                    );
                }
            }
            if done.load(Ordering::Relaxed) {
                return;
            }
            let now = s.progress.load(Ordering::Relaxed);
            if now == last {
                eprintln!(
                    "srw {label}: no progress in 20 s at {now} sections (lost wake?); lock word \
                     {:#x}",
                    unsafe { (*s.lock.get()).Ptr as usize }
                );
                std::process::abort();
            }
            last = now;
        }
    });
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            std::thread::spawn(move || {
                if pinned {
                    pin_to_cpu0();
                }
                let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (t as u64 + 1) * 0x1234_5678_9abc);
                let l = s.lock.get();
                let mix = std::env::var("SRW_MIX").unwrap_or_default();
                for _ in 0..iters {
                    let mut r = rng.next() % 16;
                    // SRW_MIX narrows a failure: x = exclusive only, xs = no try-acquires
                    if mix == "x" {
                        r %= 5;
                    } else if mix == "xs" && (r == 5 || r == 6) {
                        r = 7;
                    }
                    unsafe {
                        if r < 5 || (r == 5 && srw::RtlTryAcquireSRWLockExclusive(l) != 0) {
                            if r < 5 {
                                srw::RtlAcquireSRWLockExclusive(l);
                            }
                            assert!(!s.writer.swap(true, Ordering::SeqCst), "two writers");
                            assert_eq!(s.readers.load(Ordering::SeqCst), 0, "writer with readers");
                            *s.plain.get() += 1;
                            hold(&mut rng);
                            s.exclusive_sections.fetch_add(1, Ordering::Relaxed);
                            s.writer.store(false, Ordering::SeqCst);
                            srw::RtlReleaseSRWLockExclusive(l);
                        } else if r == 5 {
                            // try-exclusive failed: nothing held
                        } else if r == 6 && srw::RtlTryAcquireSRWLockShared(l) == 0 {
                            // try-shared failed: nothing held
                        } else {
                            if r != 6 {
                                srw::RtlAcquireSRWLockShared(l);
                            }
                            s.readers.fetch_add(1, Ordering::SeqCst);
                            assert!(!s.writer.load(Ordering::SeqCst), "reader with a writer");
                            hold(&mut rng);
                            s.readers.fetch_sub(1, Ordering::SeqCst);
                            srw::RtlReleaseSRWLockShared(l);
                        }
                    }
                    s.progress.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("a stress thread panicked");
    }
    done.store(true, Ordering::Relaxed);
    let plain = unsafe { *s.plain.get() };
    assert_eq!(
        plain,
        s.exclusive_sections.load(Ordering::Relaxed),
        "{name}: lost update"
    );
    assert!(
        unsafe { (*s.lock.get()).Ptr.is_null() },
        "{name}: lock word left non-zero"
    );
    let waits = KEYED_WAITS.load(Ordering::Relaxed) - waits0;
    println!(
        "srw {name:>22}: {threads} threads x {iters}, {plain} exclusive sections, {waits} keyed \
         waits, {:.2} s",
        t0.elapsed().as_secs_f64()
    );
    if cfg!(feature = "kernel") {
        assert_eq!(waits, 0, "{name}: the kernel copy must never block");
    }
    ROUNDS.fetch_add(1, Ordering::Relaxed);
}

fn main() {
    if cfg!(feature = "kernel") {
        // spin-only: more spinners than CPUs, or one CPU, measures the host's timeslice, not the
        // lock, so the kernel copy gets the rounds where every thread can run
        round("all CPUs, 4 threads", 4, 40_000, false);
        round("all CPUs, 8 threads", 8, 20_000, false);
    } else {
        round("all CPUs, 8 threads", 8, 20_000, false);
        round("all CPUs, 32 threads", 32, 4_000, false);
        round("CPU 0, 4 threads", 4, 10_000, true);
        round("CPU 0, 16 threads", 16, 2_000, true);
        // per round a quiet host can go without a block; over all four it cannot
        assert!(
            KEYED_WAITS.load(Ordering::Relaxed) > 0,
            "no waiter ever blocked, so the sleeping path went untested"
        );
    }
    // the line `vibeos-cli difftest` counts: one case per round
    println!(
        "difftest result: ok. {} passed; 0 failed ({} keyed waits, {} releases)",
        ROUNDS.load(Ordering::Relaxed),
        KEYED_WAITS.load(Ordering::Relaxed),
        KEYED_RELEASES.load(Ordering::Relaxed)
    );
}
