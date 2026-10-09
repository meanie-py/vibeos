# Lane: graviton-spike — our ARM64 on real ARM64

> **2026-09-20: superseded for active work.** The owner chose direct bare-metal
> boot on their Raspberry Pi 4B. Resume from [Pi 4B bring-up](pi4-bringup.md).
> The original TCG-only history below is disputed by the owner's report that
> ARM64 VibeOS already works under Linux KVM. Its ordering claims remain
> unmeasured; do not repeat them as findings. No AWS resources were created.
>
> **2026-10-06: the target is wrong, per the owner.** The aim is vibeos booting
> *as* the Graviton instance (bare metal, on Nitro), not vibeos running in QEMU/KVM
> on a Graviton host, which is what everything below plans. On Nitro the OS sees the
> hardware directly, so the work is drivers, not accelerator plumbing:
>
> - **Disks are NVMe** (EBS and instance store alike): `nvme.sys`, done on all three
>   arches (2026-10-06), on MSI-X interrupts. Nitro's device IDs are untested.
> - **The NIC is ENA** (Elastic Network Adapter), not e1000 or virtio: missing.
> - **PCIe interrupts:** Nitro devices expect MSI-X, which on arm64 is the GICv3 ITS:
>   done in QEMU (2026-10-07). The arm64 HAL drives a GICv3 (system-register CPU
>   interface, redistributors, IPIs by `ICC_SGI1R_EL1`) and an ITS (LPIs, DeviceID from
>   the IORT, through an SMMU if one sits in between; every ITS the MADT lists; messages
>   spread over the CPUs); `smoke --arch arm64 --gic 3 --nvme --smp 4 --smmu` runs
>   nvme.sys on ITS-translated MSI-X behind an SMMUv3. Any SMMU the IORT lists is put in
>   bypass at boot, since no driver programs one. The boot log prints the IORT's shape
>   and where each segment's requester 0 lands, so the first Graviton serial log answers
>   the layout questions. Unmeasured there: whether firmware leaves LPIs on in a
>   redistributor (that CPU is then passed over; if it is CPU0, nvme.sys falls back to
>   INTx). Moving an LPI to another CPU (MOVI) is proven by `--tripwire ITS_MISSTEER`,
>   which binds every LPI one CPU off so only the move at connect delivers it.
> - **PCIe config space:** from the MCFG since 2026-10-08, every bus of every segment;
>   the HAL publishes a root bus per segment past 0 and pci.sys follows bridges
>   (`smoke --arch arm64 --nvme-bridge`: NVMe behind a root port). A segment past 0 is
>   proven in QEMU by `--nvme-segment` (2026-10-09): the HAL presents a pxb-pcie
>   expander's buses as segment 1, and NVMe there runs on ITS MSI-X with requester
>   `0x18100`. That boot found the ECAM mapping every bus up to the highest one for
>   good and exhausting system PTEs; fixed. Graviton's real layout is still unmeasured.
> - **Boot and console:** UEFI and ACPI, which arm64 vibeos already uses. EC2's
>   Graviton console is not a PL011: it is Amazon's 16550-compatible PCI UART
>   (1d0f:8250), which the SPCR names at an MMIO address with byte access (FreeBSD
>   D19507, from an a1 instance). Since 2026-10-08 the loader, kernel and KD drive
>   whichever UART the SPCR/DBG2 describes, PL011 or a 16550 in memory space
>   (`ARM_LOADER_BLOCK.MachineType` carries its register model). Host-tested only:
>   QEMU's virt machine cannot present a firmware-enabled MMIO 16550.
>
> **What QEMU proves before hardware (2026-10-08):**
>
> - `--cpu neoverse-n1|neoverse-v1|neoverse-n2` (Graviton2/3, closest to 4) with
>   `--gic 3 --nvme --smp 4`: desktop.
> - `--headless` (no display, as a server): desktop, after four fixes the loader and
>   display needed (field manual, "A UEFI boot with no display hangs in the loader");
>   arm64vid runs the desktop on a frame in memory.
> - `--boot-nvme`: UEFI boots the image from an NVMe namespace, the way EC2 boots
>   an instance from its EBS root volume, and nvme.sys publishes that volume.
> - `--smp 16` with 16 GiB on Neoverse-N1: desktop. `--smp 64`: QEMU's UEFI never
>   leaves its boot manager under TCG (15 minutes serial-silent, before any of our
>   code runs); `--smp 32` is the largest tried.
> - `--smmu --tripwire SMMU=TRANSLATE`: the HAL drives the SMMUv3 for real (an
>   identity map of RAM and MSI doorbells, a stream table entry per device), and
>   `NVME_BAD_DMA=<hex>` shows a stray DMA reported as a fault. Bypass stays the
>   default, as on Windows without DMA remapping.
>
> **Still unanswerable without the instance:** Graviton's ACPI tables (dump them
> from a Linux instance first: they become host-test fixtures), firmware-left LPI
> state, PCI segment layout, SMMU state at handoff, and whether the 16550 driver
> matches the real UART.
>
> The rest of this document is kept for its history, not as the plan.


**Worktree** `/home/curi/Projects/os-playground/vibeos-graviton` · **branch** `graviton-spike`
· written 2026-09-20 by the control plane · **owner: the user**

Get the arm64 branch running on an AWS Graviton instance. This is a **spike**: the
deliverable is a measured answer and a written-up verdict, not a productionised CI runner.

---

## Why this is worth doing, and it is not the speed

Speed is the obvious pitch — arm64 smoke is **93.9s** under TCG today, and KVM on a native
aarch64 host should be a large multiple faster. Take it; it is real. But it is the smaller
half, and if the lane optimises for it the bigger prize gets missed.

**Every line of ARM64 kernel code we have ever written has only ever executed under TCG,
which does not faithfully model ARM64's weak memory ordering.** Our aarch64 guest runs on an
x86-64 host, and x86 is TSO — near-sequentially-consistent. Loads and stores that on real
hardware may be reordered are, under TCG on x86, very often not. So:

- A missing `dmb`/barrier is invisible.
- A Rust atomic with `Ordering::Relaxed` where it needed `Acquire`/`Release` passes.
- A spinlock that only *accidentally* works because the host wouldn't reorder the payload
  past the lock release passes.
- An SMP race that requires real store buffering to manifest never manifests.

Graviton is real ARM64 silicon. Under KVM, our kernel executes with **real weak ordering for
the first time.** That makes this lane a correctness oracle in the same shape as the
upstream-sync lane: not "make it faster," but **"find the class of bug the current
environment structurally cannot show us."**

Nothing in `docs/engineering/` currently records this (I grepped: zero hits for memory
ordering / weak memory / TCG ordering). The closest relative is
`qemu-record-replay.md:129` — *"MTTCG has no recorded global instruction order across vCPU
threads."* Related problem, different axis.

**Treat the paragraph above as a HYPOTHESIS, not a measurement.** It is the well-known
general property of TCG, and I have not proved it on our tree. **Task 1 proves or kills it**
(below). If it turns out TCG is stricter than assumed on our configuration, say so loudly and
the lane's value reverts to performance — which is still worth having.

This also matters because SMP is a live lane (`vibeos-smp`, `smp-bringup-quest.md`). Anything
this spike learns about ordering belongs in front of that lane.

---

## What the code currently assumes, with line numbers

Two places hardcode "arm64 cannot be accelerated," and **both are correct today and wrong on
Graviton.** They are the whole porting surface for the spike.

**`tests/qemu/vm.py:2353`** — `_is_cpu_heavy()`:
```
- native arm64: always TCG (no aarch64 KVM in WSL2)
```
```python
if self.config.arch.startswith("arm64"):
    return True                      # <- unconditional: arm64 is always CPU-heavy
return not Path("/dev/kvm").exists()
```
On Graviton `/dev/kvm` exists **and works for an aarch64 guest**, so this returns the wrong
answer. Consequence if left alone: every arm64 boot takes a host CPU-broker lease it does not
need, throttling the instance to `build-policy.json` limits for no reason.

**`tests/qemu/backend.py:253`** — `resolve_accel()` returns `"tcg"` for arm64
unconditionally. That one is about the **windows-host** backend, where the reasoning holds
absolutely (WHPX is x86-only). On Linux/native it is not in the path, but read it before
touching anything so you do not "fix" a function that is already right.

**`tests/qemu/vm.py:473-475`** — the arm64 machine is `-M virt -cpu cortex-a72`. Under KVM you
almost certainly want `-cpu host`. **This is the most likely place the spike fails**, and the
interesting failure: `cortex-a72` is an A72 feature set, Graviton is Neoverse (N1/V1/V2). Our
kernel may depend on what A72 advertises, or trip over what Neoverse adds.

**`tests/qemu/vm.py:317` `per_arch_cap`** — serializes same-arch boots for accelerator-*sharing*
backends. Today arm64 is windows-host TCG and early-returns before the cap, so same-arch
smokes run in parallel. Switch arm64 to KVM and **the cap starts biting**: parallel arm64
boots serialize. Astra's qemu-perf lane recorded this same tension for x86+WHPX. Net
throughput could get *worse* even as each boot gets faster. Measure both, and report
wall-clock for a full `gate`, not just single-boot latency.

---

## Tasks, in order

**1. Prove or kill the memory-ordering premise. Do this FIRST — it decides the lane's value.**
Write a litmus test (classic store-buffer / message-passing shape: two threads, no barriers,
observe whether the forbidden-under-SC outcome occurs) and run it three ways: TCG-on-x86 here,
KVM-on-Graviton, and native aarch64 userspace on the instance. If the forbidden outcome shows
up only on Graviton, the premise holds and it is the headline result. If TCG also shows it,
say so — the claim above is then wrong and should be struck from this doc.
*This is a userspace test first; do not start with the kernel.*

**2. Does our arm64 kernel boot under KVM at all?** This is the real risk and it is binary.
TCG is forgiving of things KVM is not: unimplemented system registers, cache maintenance,
PSCI calls, GIC version assumptions, and the CPU model gap above. Expect this to fail the
first time and expect the failure to be informative. Get to `[EXPLORER] _tWinMain entered` +
`shell ready` — **the same marker, unchanged.** Do not lower it.

**3. Can we even build there?** The host becomes aarch64 Linux while the *target* stays
aarch64 PE-COFF, so this is still a cross-compile, just from a different host arch. Check the
pinned clang, `rustup` targets, bindgen/libclang, and the `x86_64-w64-mingw32-*` binutils the
tree reaches for. Cheapest answer may be "build here, ship the ISO" — decide with a
measurement, not a preference.

**4. A/B the numbers.** Today's baselines, TCG on the windows-host backend
(`qemu-perf-plan.md`): `smoke:i386` 78.6s · `smoke:amd64` 54.9s · **`smoke:arm64` 93.9s** ·
`smoke:net` 99.8s. Report single-boot latency **and** full-gate wall clock, and state the
instance type and cost/hour. **Do not fabricate an A/B** — astra explicitly declined to
publish an unmeasured WHPX comparison and that was the right call.

**5. Write the verdict.** What works, what does not, what it costs, and whether it should
become a real runner. A "no, and here is why" is a complete deliverable.

---

## Rules with teeth here

- **Never weaken a smoke test or lower a boot marker** to get a green on new hardware. The
  marker is the product. If arm64 will not reach `shell ready` under KVM, that is the finding.
- **Never broad-kill QEMUs.** `pkill -f qemu` matches your own ancestors and has SIGKILLed an
  agent's whole process tree twice. Use `./vibeos-cli kill`.
- **Judge build success by artifact existence** (`ls livecd.iso`), never by reported exit code —
  `$?` after a pipe is the pipe's. This bites harder over SSH, where you will be tempted to
  pipe everything through `tail`.
- **Nothing in `/tmp` you care about**, on an instance you will terminate. Paid artifacts and
  results come home to the repo.
- **Path discipline**: resolve `$W=/home/curi/Projects/os-playground/vibeos-graviton` once and
  prefix every path. An absolute path into the main checkout writes to the main checkout.
- `./vibeos-cli land` is the merge queue; docs-only batches now skip the gate entirely, so
  write-ups land in seconds.
- **`verify_signatures` will report `SCANNER_ERROR` in this fresh worktree.** Correct, not
  broken: it needs generated headers from *this* worktree and refuses to borrow a sibling's,
  because a verdict that depends on which siblings exist is not a gate. Fix with
  `./vibeos-cli build --arch i386`. The `land` preflight warns rather than blocks.

## Open questions for the owner — do not spend money deciding these alone

1. **What instance, and what is the budget?** Graviton3/4, how many vCPUs, spot or on-demand,
   and is there a standing hourly cap.
2. **Is the goal CI capacity or correctness validation?** They pull in different directions:
   CI wants many cheap parallel boots, the ordering oracle wants one faithful machine and real
   `-smp`.
3. **How does the instance get the tree** — clone from origin, or ship a tarball? Affects
   whether AWS credentials or a deploy key are needed anywhere.
