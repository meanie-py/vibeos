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

## Hardened warm-output trials

Fault injection after the cold checkpoint exposed two edge cases: selecting a cache
executable whose execute bit belonged to another permission class, and a SIGKILLed
worker stranding a grandchild. Both are addressed by caller-permission checks and the
private-session controller/runner boundary. Tests cover raw signal receipts, whole-group
cleanup, stubborn children, untouched detached services and private receipt FD closure.
The cold observations above identify their exact earlier revision; they do not claim to
measure later hardening commits.

Revision `08f120e4c90cad8015a7586b073386a77a323c66`, clean tree throughout,
2026-11-02 local time (2026-11-03 UTC), same pinned tools and unchanged capacity:

```sh
/usr/bin/time -v ./vibeos-cli build-bench --arches i386 amd64 arm64 --repeat 2 \
  --output .cache/build-infra/phase6-warm.json
```

Both repeats passed all builds, AMD64 guest restaging, early/deferred source checks
and architecture checks. The first repeat refilled changed artifact-policy entries;
the second reused the resulting store. Source and policy snapshots matched before and
after, and the live policy file retained the SHA-256 above.

| Measurement | Refill repeat | Reuse repeat |
| --- | ---: | ---: |
| Total wall time | 2699.99 s | 1222.79 s |
| i386 build | 1721.00 s | 106.80 s |
| amd64 build | 2098.44 s | 637.44 s |
| arm64 build | 1719.11 s | 86.59 s |
| AMD64 guest restage | 376.65 s | 496.62 s |
| Sampled maximum summed child-tree RSS | 18.658 GiB | 5.960 GiB |
| Minimum host MemAvailable | 17.467 GiB | 32.554 GiB |
| Kernel OOM-kill delta | 0 | 0 |
| Samples at nominal 200 ms | 12,653 | 5,942 |

The artifact store recorded 314 i386, 341 amd64 and 342 arm64 producer calls during
refill; its AMD64 restage recorded none. The reuse repeat recorded no artifact-store
producer calls: i386 and arm64 made no artifact requests, while each AMD64 build/restage
recorded 886 hits and seven explicitly ineligible requests. Those ineligible requests
retain the existing build path; zero store producers does not prove zero compilation
outside the store. Reuse also includes wrapper, link and staging work. Reducing
that AMD64 churn is a measured follow-up, not a claimed improvement in this migration.
These repeats have different cache histories; their timings are not an old-versus-new
implementation comparison. Other lanes were active; source and tooling gates also ran
alongside the refill repeat.

The independent procps/direct-meminfo sampler covered 3570 rows over 3622.86 s,
starting after the benchmark began: maximum summed tree RSS 18.385 GiB and minimum
host availability 17.557 GiB. The shared-page and sampling caveats above still apply.
GNU time separately reported maximum RSS 5,819,504 KiB; this per-process high-water
measurement is not the simultaneous sum of the descendant tree. Its elapsed time
was 1:05:25, exit zero.

Evidence: `.cache/build-infra/phase6-warm.json`, `phase6-warm-run{01,02}/`,
`phase6-warm-independent.jsonl` and `phase6-warm-time.log`. These runs validate the
hardened build/check path; the desktop smokes above belong to the earlier cold checkpoint.
The integrator still owns the final merged-tree cold gate and three desktop smokes.

Python argument routing, Run/Stage presentation, policy-lock lifetimes and drained fleet
activation remain supported control-plane adapters until the CLI/QEMU phases. No new
scheduler or capacity tuning is part of this migration.

## Warm invalidation follow-up

Revision `df08aa31d3d37abb46105b4c844513661c09c956`, 2026-11-03 local time
(2026-11-04 UTC). The tree stayed clean at that revision throughout both repeats:

```sh
./vibeos-cli build-bench --arches i386 amd64 arm64 --repeat 2 \
  --output .cache/build-infra/phase7-fixed-warm.json
```

Both repeats passed all builds, guest restaging and source/architecture checks.
The first repeat took 1916.14 s and included native-policy cache refill; old store
objects were retained. Neither repeat recorded an OOM kill. Capacity values and
the live policy SHA-256 above were unchanged, and no cache was flushed.

The table compares repeat 2 against the earlier repeat 2 at `08f120e4c90`:

| Observed measurement | Earlier reuse | Current reuse |
| --- | ---: | ---: |
| Complete three-architecture suite | 1222.79 s | 311.41 s |
| AMD64 initial build | 637.44 s | 138.29 s |
| AMD64 guest restage | 496.62 s | 82.95 s |
| Restage main Ninja phase | 406.6 s | 19.8 s |
| Logged Cargo-link edges per AMD64 build/restage | 368 | 5 |
| Artifact requests per AMD64 build/restage | 893 | 35 |
| Store hits per AMD64 build/restage | 886 | 31 |
| Explicitly ineligible requests per AMD64 build/restage | 7 | 4 |
| Store producer calls in reuse repeat | 0 | 0 |

Times come from the benchmark's owned child receipts and each child's `full.log`;
Cargo-link counts come from the logged Ninja progress lines, not its planned-work
denominator, which shrinks when restat prunes unchanged consumers. Store counts
come from supervisor activity receipts. Neither i386 nor arm64 made artifact
requests in either reuse repeat. Ineligible requests retain their existing build
path; zero store producers does not prove zero compilation outside the store.
These are observations on a shared host at different revisions. Other lanes were
active, and gates ran alongside the first repeat; no isolated wall-time speedup is
claimed. The CMake/Ninja/Cargo fixture supplies the controlled invalidation proof:
identical reconfiguration runs no link edge, while Boolean and genuine compile
command changes rebuild readers and update executable behavior.

`check` and all 118 `check-tooling` checks passed on the measured runtime revision.
The 21 focused dependency/context tests, 13 provenance controls and native depfile
snapshot test passed, including refusal to boot changed commands hidden behind an
old snapshot. Evidence is under `.cache/build-infra/phase7-fixed-*`; the earlier
`phase7-warm.json` is an interrupted investigation probe, not a successful trial.

All three subsequent desktop smoke markers passed and screenshots were inspected.
Each summary is WARN solely because these were warm builds:

- i386: `/tmp/vibeos-cli/20261103-200443-i13h8gnx-smoke-i386`, boot reached.
- amd64: `/tmp/vibeos-cli/20261103-200443-7br23k8c-smoke-amd64`, boot reached;
  16 network tests passed in 55.18 s.
- arm64: `/tmp/vibeos-cli/20261103-200443-1brvojlt-smoke-arm64`, boot reached.

The integrator retains the final merged-tree cold gate and three desktop smokes.
The five remaining AMD64 Cargo-link edges are tracked in the
[build invalidation notes](build-invalidation.md#cmake-option-projections-for-cargo-links).
