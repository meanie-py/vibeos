# Concurrent Windows media-cache fills

Read-only media staging now copies different cache entries concurrently. Warm
hits can pin an existing image while an unrelated image is still copying. This
is the default in `backend.stage_readonly_file`, including smoke, gate, farm,
checkpoint, and the detached debugger's media staging.

## Locking and compatibility

- Hash the actual source on every call, as before.
- Check and pin existing entries under the original global `cache.lock`.
- On a miss, release the global lock, acquire a per-entry Linux flock, and
  recheck. Only one new launcher fills a particular entry at a time.
- Copy into a private directory while holding only the per-entry lock. Retain
  the existing 16 KiB writer and source identity checks.
- Acquire the global lock to atomically publish, pin, touch and prune. If an
  older launcher published first, use its immutable inode; never overwrite it.

The lock order is per-entry → global; no caller waits for a per-entry lock while
holding the global lock. Cleanup takes only the global lock. Linux lock files
remain permanent so a waiter cannot end up locking an unlinked inode.

Temporary directories use `.populate-v2-<linux-pid>-<random>`. Older cleanup only
recognizes `.fill-*` and therefore cannot delete a new fill while it runs outside
the global lock. New cleanup recognizes both formats and preserves a live
population owner even when `min_age=0`. Abandoned fills become eligible under the
normal age policy after their owner exits. A copy error removes its private
files without publishing or pinning a partial image.

## Verification

Tests exercise overlapping cold copies, a warm hit plus cleanup during a blocked
cold copy, same-entry deduplication, an older publisher winning the race, failed
copies, same-size/mtime source mutation, and a real child process dying mid-fill.
The latter also verifies automatic flock release, abandoned-file cleanup and a
successful subsequent fill. Existing writable-isolation and pin-aware retention
tests remain in place.

The controlled helper-level experiment and raw results are retained under
`/tmp/vibeos-media-cache-bench/`: `measure.py`, `baseline_backend.py`,
`candidate_backend.py`, and `summary.json`. It compares the prior committed
backend with the candidate using the same i386 ISO, amd64 ISO and ARM64 boot
image. It checks every output's SHA-256 and hard-link count and verifies that
inputs did not change. Three paired trials alternate implementation order,
using fresh Windows cache directories for cold trials and reusing them for warm
trials. Cold means an empty tool cache; operating-system page caches are not
flushed. A separate trial starts an unrelated cold copy before requesting a warm
hit. All benchmark-owned Windows directories are removed after use.

### Staging measurements

| Trial | Previous cold batch | Concurrent cold batch |
|---|---:|---:|
| 1 | 46.503s | 24.153s |
| 2 (new implementation first) | 40.994s | 20.784s |
| 3 | 41.399s | 17.823s |
| Median | **41.399s** | **20.784s** |

Warm three-image batch medians were 0.313s before and 0.343s after. These are
helper-level staging measurements on a shared host, not whole-boot or full-gate
speedups. Parent-side monotonic timers measure complete calls; output hashes and
hard-link counts independently prove that the calls staged the requested bytes.

A warm hit requested after an unrelated cold copy began took **16.999s before**
and **0.152s after**. This isolates the head-of-line blocking removed by the
change. All input hashes remained unchanged and all copied output hashes matched;
the benchmark's Windows directories were removed.

### Integration validation (2026-09-20)

Artifacts under `/tmp/vibeos-cli/`:

- `20260920-011436-jgaagd_x-gate-i386+amd64+arm64`: all three clean
  builds and desktop smokes passed; actual ISO/ARM64 image files verified.
  The gate process later exited 143 without a final summary; this is not a
  completed gate.
- `20260920-012146-hs78pf5g-gate-i386+amd64+arm64`: second desktop smoke
  on each architecture plus networking passed (execution-only gate).
- `20260920-012312-1ycsu90s-gate-i386+amd64+arm64`: third successful
  desktop smoke per arch, networking, conformance, source checks and difftest.
  ARM64 smoke recovered from `KNOWN_FLAKE` (BUGCHECK 0x1E, PC 0); networking
  and ARM64 conformance also required the existing classified-flake retries.
  Its final verdict remains `TOOLING_FAIL`, preserving the failed test result.

That tooling run exposed a replay progress probe resolving every `/proc` fd
through unrelated mounts. It now compares the kernel's symlink target directly,
with a regression check forbidding fd-target resolution. The other failure,
a host-lock announcement test, passed in isolation and in the full rerun.
After the probe fix, all **57 tooling checks passed**, with no cached checks
reused; the focused replay/cache suite also passed **100 tests**. Screenshots
from the first smoke round were inspected. These results establish component
coverage, not a clean-tree landing stamp; the merge queue must run its own gate.
