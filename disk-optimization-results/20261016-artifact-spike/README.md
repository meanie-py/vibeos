# Immutable-body implementation decision

Measured 2026-10-16 UTC on `shared-rust-bodies`, baseline `eaa4c573a9a`, with the
discovery/worker tooling under development. Full reports and compiler logs live in
`/home/curi/vibeos-rust-staging/artifact-{inventory,spike,executor}/`; committed
summaries retain revisions, keys, methods and evidence paths. No store hits or
archive deletion were enabled for the initial inventory measurements. Subsequent
implementation evidence is distinguished below.

## Decision: proceed with an opt-in body-store prototype

The material first consumers are CRT startup, CRT entry bridges, DLL entry shims
and console utilities. The full kernel remains a useful large-closure correctness
case, but it is not the dominant repeated archive in one lane.

The four source-eligible body packages have 1,074 consumer snapshots occupying
5,758,980,096 allocated bytes. SHA-256 finds 21 distinct contents needing
112,635,904 bytes. Replacing consumer copies with immutable direct reads therefore
offers **5.26 GiB per lane before source/scratch/reference overhead**, or **1.45 GiB
on amd64**. These are migration estimates, not reclaimed bytes. No shared dependency
intermediate or primary-archive deletion receives any credit. Copying store hits
back into consumers would retain all existing copy bytes and add the store blobs.

All source-supported packages were checked with actual request features/defaults.
Native DLL import metadata was admitted only after pinned compiler negative controls;
bundled static native inputs, build scripts, proc macros, target-specific dependency
tables and other unmodeled constructs remain ineligible. `cnt-rust-lib` is excluded.

The full survey found 57.599 GiB across the workspace target and three build roots.
Independent `du -sc --block-size=1` exactly matched 61,846,265,856 bytes. Outer Cargo
coordinators account for about 16.0 GiB and remain private. See `inventory.json` and
`economics-reviewed-native.json`. Duplicate-byte totals do not establish reachability
or authorize pruning another lane.

## Resolution and reuse

Across sixteen current worktrees, all seven material request variants are eligible.
Six variants each have one projected key; conutils has three. Under one proposed
common execution recipe, this is **91.96% request- and archive-byte-weighted reuse**
after the first producer of each key, versus 31.25% with whole-manifest keys.
Compile-cost weighting is 83.15% on the two variants actually timed; the other five
are explicitly unmeasured and excluded from that weighting. These are candidate
matches, not observed hits or claims about each lane's existing compiler settings.
`projection-rates.json` records the normalization and denominators.

Historical replay across the parent/child states of `890939fad27` and `263a1bb5994`
kept beep and kernel projected keys unchanged across unrelated workspace additions;
whole-manifest comparison produced three distinct keys. This replay uses tracked
sources and a proposed current toolchain, not reconstructed historical builds.

## Isolated build measurements

All runs use pinned Rust 1.92, one admitted compiler slot, Cargo jobs=1, empty targets,
no sccache/incremental cache, canonical paths, read-only source/tool mounts, a minimal
chroot and private network namespace. The native import recipe includes hashed GNU
dlltool, its assembler and runtime libraries. Pinned tool/runtime files must not be
modified by a host writer during a job; postchecks verify their content again.

Those initial timings used GNU dlltool. Reproduction testing subsequently found
that it embeds rustc's random temporary-directory names in COFF import symbols;
deterministic archive headers do not remove those names. The implementation now
uses the bundled, hashed LLVM dlltool and its shared library via an explicit rustc
flag. It compares the original archive bytes without normalization.

| Body | Prepare | Cold compile | Target allocation | Two source copies |
|---|---:|---:|---:|---:|
| CRT startup (`crtexe`) | 1.68 s | 0.47 s | 4.41 MiB | 0.47 MiB |
| Console utilities | 3.03 s | 2.61 s | 26.62 MiB | 10.43 MiB |
| Beep | 6.89 s | 9.26 s | 101.88 MiB | 37.98 MiB |
| Kernel (34 packages) | 12.61 s | 32.04 s | 192.93 MiB | 74.23 MiB |

Preparation includes whole-toolchain fingerprinting and repeated discovery; it is
not a hit-time budget. A supervised build must amortize toolchain provisioning.
Worker source/scratch can be removed after publication, retaining provenance and
the blob. Current prototype directories were retained as measurement evidence.

These are single runs on a busy host, not a controlled whole-OS speed comparison.
For four identical cold requests and one available compiler slot, four private
targets require approximately `4*C` compilation work; a single producer requires
`C`, plus lookup/setup and waiting. For beep this is 37.04 versus 9.26 compiler
seconds; for the kernel 128.15 versus 32.04. Existing warm shards can be faster than
this cold reference. This is a contention model, not an observed deduplicated run;
actual four-reader, four-distinct-miss and dependency-edit experiments remain owed.

The later tiny-fixture concurrency experiment below supplies observed reader and
distinct-miss results; it does not replace the material-body or dependency-edit
measurements.

## Fixed prototype acceptance targets

Set before implementing store primitives; do not lower these to fit results:

- Net retained reduction: at least **1.2 GiB on amd64**, and **4.5 GiB across all
  three architectures** before broad rollout, on the measured material consumers.
  These allow approximately 0.25/0.76 GiB overhead beneath the verified archive-only
  opportunities and give no credit for deleting shared intermediates.
- Projected request- and byte-weighted reuse: **at least 80%** on this eligible
  material sample, retaining the unrelated-workspace-member positive control.
- With a supervisor-provisioned toolchain: median small-body hit **at most 0.5 s**,
  large-closure hit **at most 5 s**; one-time provisioning **at most 2 s** on an idle
  host. Unchanged full-build preflight adds **at most 2 s**.
- Four same-key cold readers: one producer, all correct, wall time no more than
  **1.25 times the single cold producer plus 5 s**, under the same broker capacity.
  Four distinct misses must stay within **25% plus 10 s** of the equivalent admitted
  private-build reference. Measure the shared-dependency-edit case separately.
- Four admitted worker jobs: additional peak allocation **at most 1.5 GiB** for
  this sample; completed private worker targets/source copies are not retained.

The disk opportunity, key-stability controls and measured worker sizes justify the
prototype. This decision does not authorize production activation: independent byte
reproduction, real same-key contention, leases/GC, dependency invalidation, direct
linking/PE equivalence and the architecture gates remain required.

## Implementation evidence

`kernel-reproduction.json` records two independent isolated kernel compilations
using LLVM dlltool. Both produced the same SHA-256 archive. The GNU recipe's
conflicting outputs remain quarantined in the durable pilot evidence directory;
they were not accepted as equivalent. A separate raw-DLL fixture verifies both
function and data imports by linking a PE from each reproduced archive.

Three subsequent kernel hit requests took 13.13, 4.54 and 4.56 seconds, with zero
compiler calls (`kernel-hits.json`). The median meets the five-second request target,
but the first sample shows load sensitivity. These exclude one-time supervisor
provisioning and final toolchain checks; unchanged whole-build overhead still needs
its own measurement. Shared host load was uncontrolled.

`tools/test_artifact_runtime_real.py` runs four actual compiler consumers across
two independently located source trees: one producer, three hits, one read-only
archive with link count one, and no retained worker targets. GC preserves that
archive after both supervisors exit because durable consumer references remain.

`tools/test_artifact_ninja.py` exercises the shipping consumer, input watches and
depfile retargeting through real Cargo, CMake and Ninja. Nested Rust edits, included
asset edits and file additions change the linked result. Both negative controls
(package-root-only watches and suppressed body watches) leave stale output, proving
that the positive test depends on the required invalidation edges.

The final `./vibeos-cli check-tooling` run passed all 150 checks with matching
inputs, including the new store, runtime, repair, configuration and integration
tests. Earlier formatting failures were fixed; a subsequent all-tests-passing run
correctly refused receipts because source edits overlapped it. Neither earlier
run is counted as the final tooling pass.

`tiny-concurrency.json` records a real isolated-compiler experiment: one cold
request took 2.06 s; four same-key consumers in separate source roots took 2.30 s
with one producer and three hits. Four distinct misses took 2.48 s, versus 1.93 s
for four private workers, and both reached four admitted workers. Four serial
warm hits performed no compilation and had a 0.258 s median. Supervisor startup
and postcheck added approximately 0.90 s and 0.87 s separately. Independent
`sha256sum` checked all eleven produced archives against the recorded digests.
These are tiny-fixture timings under recorded shared-host load, not OS-body
latency or exact combined transient disk peaks. The report predates the fix that
coalesces concurrent verification comparisons; ordinary hit/miss behavior is
unchanged by that fix.

The cold OS verification run exposed duplicate comparison work among waiters.
`test_concurrent_cold_verification_compares_once_per_supervisor` now exercises
four concurrent requests against the real store: exactly two compilations total
(initial publication and independent comparison), one blob, all verified readers.

The initial cold amd64 OS build completed successfully at
`/tmp/vibeos-cli/20261015-215914-a282r9fi-build-amd64`. Its supervisor recorded
716 successful requests, zero failed requests, ten actions and matching independent
comparisons. This run includes a second Ninja pass after the helper changed and
the extra comparison builds that exposed the race above; its 23 builder calls
must not be presented as the final coalescing behavior. All twelve saved PE
baselines passed `tools/pe_equiv.py --code`, covering both kernels, win32k, ntdll,
kernel32, both HALs, beep, null, condrv, cmd and chcp.

Real-repository hit samples during that build were slower than the tiny fixture:
small CRT-body medians were approximately 0.6–0.7 s and conutils approximately
1.6 s. Initial supervisor overhead outside the outer build was 2.98 s, including
1.20 s provisioning and 1.37 s postcheck. These busy-host observations **do not
meet** the 0.5 s small-hit and 2 s unchanged-build overhead targets. Keep the
feature opt-in; amortizing repeated resolution and validation while preserving
the invalidation controls is required before default activation. Material-body
four-lane/dependency-edit timing, exact transient disk peaks, other architectures
and automatic reference retirement outside store-enabled clean remain owed.

After the cold build, all 358 baseline amd64 material-consumer snapshot paths
were absent, and scanning current body request markers found zero replacement
copies for those four packages. Their baseline allocation was 1,588,891,648 B.
Independent `du` charged the entire pilot store (67,723,264 B) and cold supervisor
receipts (5,447,680 B). A deliberately conservative metadata charge adds every
directory block (22,036,480 B), depfile, fingerprint, JSON/text file and build-script
output/stderr (65,482,752 B) in the selected outer Cargo roots, even though most
predated the store. The resulting **1.330 GiB archive-copy reduction** exceeds
the 1.2 GiB amd64 target. This is attributable archive-copy savings, not whole-disk
free-space change; no intermediate or primary archive deletion receives credit.
The sequential samples and method are recorded in
`/home/curi/vibeos-rust-staging/artifact-pilot/retention-after-cold.json`.

The initial full cold gate
`/tmp/vibeos-cli/20261015-215914-x7o38yqy-gate-i386+amd64+arm64` did **not** pass:
it retained the earlier formatting failure and amd64 panicked after reaching the
desktop, during the WoW64 APC ladder. The crashed VM and then its still-draining
smoke process were explicitly stopped; the aggregate records `SMOKE_NO_SUMMARY`,
while the preserved boot serial records the actual `ke/apc.rs:138` bounds panic.
Network tests (16/16), all conformance sweeps, differential tests, ARM64/i386
smokes, amd64 BIOS smoke and strict scans passed in that run.

Investigation found an independent pre-existing timer ABI defect, also visible in
the archived October 11 kernel: `ex-rust` declared the 32-bit APC environment
argument as `u8`. The caller wrote `r8b`, while the callee compared `r8d`, allowing
index 2 to survive normalization. The accompanying fix imports the canonical DDK
declaration/constant and uses its typed system-ABI callback. The earlier twelve
PE comparisons establish equivalence of the store change **before** this intentional
kernel fix; they are not a claim that the repaired kernel is byte-identical.

The repaired amd64 UP and MP binaries initialize the full `r8d` argument with 2;
both i386 callback bodies now return with the required 20-byte stack cleanup.
Disassemblies and binary hashes are preserved under
`/home/curi/vibeos-rust-staging/artifact-executor/timer-apc-abi/repaired-*`.
The dependency edit produced new kernel action keys
`111c55be415e9e0601837ab741b33ac2fbf2dab75f94176125448ecb68b01bd3` and
`251fac6e43a6d5afadd6ecaee0cce5617469bb1f5a3d2d4e89fc04010de4c924`;
both required two independent matching compilations. The eight unchanged actions
kept their keys and each required one comparison build for the cold verification
session. Thus ten actions used twelve producer calls, with no repeated comparisons
for waiting consumers.

The final cold gate passed with exit 0 in 28m56s:
`/tmp/vibeos-cli/20261015-224639-lwfmynat-gate-i386+amd64+arm64`.
All three builds, desktop smokes, strict scans, cohab checks and conformance
sweeps passed, as did amd64's WoW64 ladder, BIOS smoke, network tests,
differential tests and all 150 tooling checks. Screenshots were inspected.
The amd64 serial records `apc32` rung 8's waitable-timer completion and `PASS`.
The aggregate verdict is `WARN` solely because the uncommitted tree receives no
push stamp; it does not claim mainline landing.

The final cold supervisor recorded **708 successful requests, zero failures,
ten actions and twelve producer calls**, with independent comparisons satisfied.
Its subsequent normal media-restage supervisor served **361 hits with zero
producer calls**. Every one of the 358 baseline material-consumer snapshots also
matches a current durable reference by package, body, feature set, default-feature
mode and kernel/user mode; the storage estimate does not count abandoned consumers.

An unchanged store-enabled amd64 build then passed at
`/tmp/vibeos-cli/20261015-231700-yb53obu1-build-amd64`: **zero requests and zero
producer calls**. The supervisor measured 13.80 s total, including 10.93 s outer
work and 2.87 s outside it (tool provisioning, reference validation and postchecks).
This confirms Cargo/Ninja freshness survives a new supervisor session, while
also confirming that the two-second overhead target is still unmet. It is not
an A/B wall-time speedup claim. The final store occupied 97,259,520 allocated
bytes, including the old and repaired kernel variants; the earlier 1.330 GiB
retention sample predates those additional verified outputs.
