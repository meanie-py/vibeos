# Native build infrastructure rollout

The capacity policy is unchanged: version 1, six legacy leases, three builds, six Ninja
jobs per build, one Cargo job. No cache was flushed and no host-policy cutover was applied.
The policy file SHA-256 before and after the cold run was
`52aa598ebacd7102400d8e45a5a7deb21195d98561c81f4f1b41ff2e62d5f06f`.

## Cold outputs and media

Revision `86a4fe6aab1fd9b9137af3d434678dbe4ede7a98`, clean tree, 2026-11-02.
Pinned tools: rustc/cargo 1.92.0; LLVM-MinGW clang 19.1.7 (20250114 UCRT package).

```sh
./vibeos-cli build-bench --arches i386 amd64 arm64 --clean \
  --output .cache/build-infra/phase6-cold.json
```

All child verdicts succeeded, including AMD64 guest restaging, early/deferred source
checks and architecture checks. Total wall time was 3218.81 s; child build times were
2201.80 s i386, 2576.02 s amd64 and 2162.82 s arm64. AMD64 restaging added 442.91 s.
This run includes native-tool provisioning and artifact-policy cache refill; it is a
cold-output observation with shared compiler caches, not a controlled speed comparison.

The 200 ms sampler collected 15,268 rows. Its maximum summed child-tree RSS was 16.621 GiB;
minimum host MemAvailable was 27.046 GiB and the kernel OOM-kill delta was zero. A separate
sampler used procps `ps -eo pid,ppid,rss,args` to select worker descendants, converted RSS
from KiB to bytes, and read `/proc/meminfo` directly once per second. It began after the
trial started and collected 2963 rows over 3038.45 s: maximum summed RSS 16.127 GiB and
minimum host availability 27.227 GiB during that overlapping window. Shared pages can be
counted in several processes; samples miss short peaks. Host availability includes other
lanes. `/proc/vmstat` read directly after the run also reported `oom_kill 0` since boot.

Durable local evidence:

- `.cache/build-infra/phase6-cold.json` and `phase6-cold-run01/`
- `.cache/build-infra/phase6-cold-run01/samples.jsonl`
- `.cache/build-infra/phase6-cold-independent.jsonl`

All three desktop smoke markers passed, and screenshots were inspected:

| Architecture | Run | Verdict |
| --- | --- | --- |
| i386 | `/tmp/vibeos-cli/20261102-212310-69szvff8-smoke-i386` | OK, boot reached |
| amd64 | `/tmp/vibeos-cli/20261102-212310-d2utl3t5-smoke-amd64` | WARN: latest build is the required warm restage; boot reached, 16 network tests passed |
| arm64 | `/tmp/vibeos-cli/20261102-212310-lq9w4b6d-smoke-arm64` | OK, boot reached |

## Invalidation and cross-root reuse

`tools/test_artifact_runtime_real.py` runs real isolated compilers against two fresh
synthetic source roots for each architecture. Each ladder performs seven requests:
initial build, unchanged second root, warm first root, direct source edit, dependency
edit, embedded-input edit, then the unchanged second root again. Exactly four producers
and three cache hits are required; edited outputs must differ and the unchanged root
must retain its original archive. All three architecture ladders passed. No paid Rust
translation was edited or regenerated.

The cache/job/environment plans also match 114 frozen Python cases from revision
`781252da0f4d2e8513efa428b594d9d51c486cae`.

## Hardening and remaining measurements

Fault injection after the cold checkpoint exposed two edge cases: selecting a cache
executable whose execute bit belonged to another permission class, and a SIGKILLed
worker stranding a grandchild. Both are addressed by caller-permission checks and the
private-session controller/runner boundary. Tests cover raw signal receipts, whole-group
cleanup, stubborn children, untouched detached services and private receipt FD closure.
The cold observations above identify their exact earlier revision; they do not claim to
measure later hardening commits. Warm-output/refill and steady-warm trials on the hardened
checkpoint are still owed before this rollout record is complete.

Python argument routing, Run/Stage presentation, policy-lock lifetimes and drained fleet
activation remain supported control-plane adapters until the CLI/QEMU phases. No new
scheduler or capacity tuning is part of this migration.
