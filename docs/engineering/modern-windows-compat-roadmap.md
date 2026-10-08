# Final bosses: modern drivers, real GPUs and Chrome

The long-range plan for the point where vibeos stops being "ReactOS in Rust" and starts
running **today's Windows binaries unchanged**: Windows 10 drivers on real hardware, a
real GPU, and the latest Chrome. Written 2026-10-05 from a design discussion with the
owner. Product summary: [`../product/vibeos-roadmap.md`](../product/vibeos-roadmap.md) §12.

**Read the confidence markers.** Lines marked *(tree, 2026-10-05)* were checked against
this repo. Lines marked *(unverified)* are recollection about external software (vendor
driver history, Chromium internals, Libreboot payloads). Check each one against the thing
itself before building on it.

---

## Where we start

Our kernel's export surface and driver models are NT 5.2 (Server 2003). Modern drivers
and apps target Windows 10. Most of the gap is **additive** (new exports, new framework
hosts, new class extensions). Only the graphics stack is a genuinely new subsystem.

Already in the tree *(tree, 2026-10-05)*:

| Piece | State |
|---|---|
| KMDF (`vendor/wdf`) | Microsoft's MIT-licensed WDF, **built** as `wdf01000.sys`, KMDF 1.17. Vendored, `translate: planned` |
| StorPort | built (`storport.sys`), Rust port in `sdk/lib/rust/storport-rust` |
| xHCI | **Rust-native** `usbxhci` (~20K lines, written in Rust from the start); SuperSpeed devices work on the Pi 4 |
| HD Audio / PortCls | `drivers/wdm/audio/hdaudbus`, `portcls` |
| D3DKMT thunks | `win32k-rust/src/d3dkmt.rs`: the syscall surface exists, but every adapter call returns `STATUS_PROCEDURE_NOT_FOUND` because nothing populates the dxgkrnl table |
| NVMe | `nvme.sys` (2026-10-06), Rust from the start, all three arches: MSI-X interrupts since 2026-10-07 (the pin as fallback, a 10 ms timer behind both), one queue pair 63 commands deep with DMA to the requester's pages (also 2026-10-07), each namespace a disk.sys disk. Owed: a queue pair per CPU, controller reset on a hung command |
| dxgkrnl / WDDM | **missing** |

[`../product/driver-roadmap.md`](../product/driver-roadmap.md) predates the KMDF and xHCI
work and still lists both as missing. Where they disagree, this doc and the tree win.

---

## Rung 0, shared by everything: the Windows 10 kernel export surface

Every boss below needs the same layer: the kernel APIs that Windows 10-built drivers and
apps import, which a 2003-era kernel does not export. So the first move is a **tool, not a
driver**:

- **The import-gap census.** Point it at a folder of real Windows 10 PEs (a stock
  `System32\drivers`, a few vendor driver packages, `chrome.exe`/`chrome.dll`). It reports
  every import we do not export, ranked by how many binaries need it. That turns "make
  Windows 10 binaries load" into a burn-down agents can grind, the way `port_audit.py`
  did for translation. User-mode binaries also resolve APIs at runtime with
  `GetProcAddress`, which no import table shows. Loader logging of failed lookups during a
  run covers those.
- **Layout drift.** WDK macros compile struct field offsets into drivers (`IRP`,
  `IO_STACK_LOCATION`, `MDL`, `DEVICE_OBJECT`, dispatcher headers). A driver built with the
  Windows 10 WDK assumes Windows 10 layouts, and a mismatch is silent corruption. Extend
  the offset verifiers to check our layouts against a Windows 10 WDK.

Known contents, from the design discussion: the newer pool and MDL APIs
(`ExAllocatePool2/3`, `MmMapIoSpaceEx`, no-execute MDL mappings), processor-group calls
(trivially one group), `EtwRegister` and WPP plumbing (stub success first),
`KeSaveExtendedProcessorState`, DMA adapter v3, `PoFx` runtime power (a keep-everything-on
stub first). Real subsystem work, not stubs:

1. **MSI/MSI-X interrupts** through `IoConnectInterruptEx`. Modern GPUs and NICs expect
   them. **MSI-X works (2026-10-07), the Windows way:** a driver opts in with
   `MSISupported` under its device key; pci.sys adds one message requirement per table
   entry (in FILTER_RESOURCE_REQUIREMENTS, since ReactOS queries requirements before
   AddDevice); the HAL translates each into a vector (x86: an APIC vector with no IOAPIC
   pin; arm64: a GICv2m SPI, or on a GICv3 an LPI that the ITS translates from the
   device's requester ID through the IORT) and supplies the address/data pci.sys writes
   into the table; `IoConnectInterruptEx` connects message-based, or falls back to the
   line. The arm64 HAL drives a GICv3 (Graviton's) as well as the Pi's GICv2 since
   2026-10-07: `smoke --arch arm64 --gic 3 [--nvme] [--smp N] [--smmu]`. The ITS backend
   brings up every ITS the MADT lists, uses a two-level device table where the ITS can,
   and spreads each device's messages across the CPUs (one collection per CPU); the
   IORT walk follows a root complex through an SMMU, which the HAL puts in bypass at
   boot since no driver programs an IOMMU. Since 2026-10-08: a device's `Affinity
   Policy` (DevicePolicy, AssignmentSetOverride) rides on its message requirements and
   the HAL places each message on a processor it allows, on all three arches (proved
   by `--tripwire NVME_AFFINITY=<mask>`); arm64 `KeConnectInterrupt` enables on the
   interrupt object's processor, as x86's does, and an ITS LPI enabled elsewhere is
   moved there (MOVI). The PnP manager gives message vectors back to the HAL when a
   device is removed or a translation is abandoned (ITS: DISCARD, then the device
   unmapped), and pci.sys turns MSI/MSI-X off at STOP and REMOVE. A device with MSI
   but no MSI-X gets one MSI message. Owed: multi-message MSI (an aligned block of
   data values), `CONNECT_MESSAGE_BASED_PASSIVE` (no consumer: KMDF does passive
   handling itself over a DIRQL interrupt), and driving an SMMU for real (isolation)
   rather than bypassing it. The arm64 HAL's PCI config space comes from the MCFG since
   2026-10-08: every bus of every segment, bus numbers being PCI_SEGMENT_BUS_NUMBERs
   (pci.sys keeps the segment through bridges and in the requester ID the ITS routes
   by), and the HAL publishes a root bus for each segment past 0. `smoke --arch arm64
   --nvme-bridge` runs nvme.sys behind a PCIe root port on bus 1. Segments past 0 are
   untested: QEMU's virt machine has one.
2. **Write-combined mappings** (`MmWriteCombined` through a correctly programmed PAT).
   Getting it wrong is mostly a performance problem, but it can show up as visible
   corruption.

None of this re-architects ntoskrnl.

---

## Boss 1: modern hardware drivers

Since Vista, Windows drivers are written against **frameworks plus class extensions**, not
raw WDM. Support a framework and a whole category of hardware runs its Windows 10 driver
unchanged.

| Order | Area | What it takes | Notes |
|---|---|---|---|
| 1 | **KMDF wired up** | KMDF is built. What's left: version negotiation (drivers request specific WDF versions; ours is 1.17) and real drivers loaded on it | The master key: most modern drivers are KMDF |
| 2 | **NVMe** | **First version done** (`nvme.sys`, 2026-10-06): a PnP driver that presents namespaces straight to disk.sys, not a StorPort miniport, because our StorPort never completes an SRB (its PDO handler and bus relations are stubs) and scsiport's legacy PCI scan cannot assign resources on arm64. INTx interrupts since the same day, MSI-X and a 63-deep queue with direct DMA since 2026-10-07. Owed: per-CPU queues, controller reset, and finishing StorPort, which vendor NVMe and RAID drivers still need | No modern PC boots without it. The test box has one (below) |
| 3 | **USB 3** | Our xHCI driver exists and works. The remaining question is compatibility with Windows 8+'s USB stack shape (`usbxhci` + `ucx01000` class extension + `usbhub3`) for vendor USB drivers that bind to it | Much further along than the rest |
| 4 | **NDIS 6.x, then NetAdapterCx** | Modern Intel/Realtek NIC drivers are NDIS 6; the newest are NetAdapterCx (Windows 10, on KMDF) | Our NDIS is 5-era |
| 5 | **HD Audio and the Vista audio stack** | Kernel: WaveRT miniports on PortCls (Realtek HDA drivers). User mode: the audio engine (`audiodg`) and WASAPI, which every modern app (Chrome included) talks to | Wine has WASAPI (`mmdevapi`) |
| 6 | **Wi-Fi** | NDIS 6 + Native WiFi / WDI + the `wlansvc` service | Large; it's what makes laptops viable |
| 7 | **Laptop input and sensors** | HID over I²C (`hidi2c`) on `SpbCx` (the I²C/SPI bus) and `GpioClx` (GPIO interrupts). Precision touchpads are this stack | Same KMDF + class-extension pattern |
| — | **UMDF** | User-mode driver host plus its kernel reflector. *(unverified)* The host and reflector were not part of Microsoft's open-source drop, so they would be ours to write | For sensors, some HID, printers |
| — | **ACPI 5.x** | Newer firmware tables; lids, batteries, power buttons, GPIO-signalled devices. Modern Standby stays out of scope; S3 is enough | |

**The pitch:** vibeos boots a modern PC from NVMe, with USB 3, Ethernet, Wi-Fi and audio
all running their own Windows 10 drivers, unmodified.

---

## Boss 2: a real GPU — XDDM, then WDDM 1.1, then WDDM 2.x

### How different WDDM is

| | XDDM (what we have) | WDDM |
|---|---|---|
| Where the driver lives | Mostly kernel: a display DLL inside win32k plus a miniport under `videoprt` | A thin kernel driver, plus a heavy **user-mode driver** loaded into each process (builds GPU command buffers, compiles shaders) |
| GPU memory | The driver manages it alone | **dxgkrnl** manages it: allocations, paging between VRAM and system RAM, GPU virtual addresses from WDDM 2.0 on |
| Sharing the GPU | One owner, no scheduling | dxgkrnl **schedules** work from many processes |
| GPU hang | Bugcheck | **TDR**: dxgkrnl resets the GPU and the OS survives |
| Display setup | Mode-set calls | Display topology management (VidPN) |
| Desktop drawing | GDI draws straight into the driver | GDI draws into memory; a compositor (DWM) presents it. DWM is optional on Windows 7, mandatory from Windows 8 |

**What keeps win32k from a rewrite:** since Windows 7, a **Canonical Display Driver**
(`cdd.dll`) sits in win32k's display-DLL slot and translates the XP-era drawing interface
into dxgkrnl calls. We'd write a CDD-equivalent as an adapter. win32k keeps its interface,
and composition can come later because drivers don't require it.

The rest of the stack: `dxgkrnl.sys` + `dxgmms2.sys`, the `D3DKMT*` bridge (our thunks,
populated), D3D runtimes and DXGI wired to the real path instead of Wine's OpenGL path, and
the Windows 10 kernel exports from rung 0. Our product driver roadmap sizes dxgkrnl at
100K+ lines. *(unverified)* Upstream ReactOS has an experimental dxgkrnl effort; check how
far it got before starting ours.

### The ladder

The test card is a Radeon HD 5450 (AMD, ~19 W, fanless) in the OptiPlex 7010.

1. **XDDM driver: AMD's XP x64 Catalyst.** It uses the driver model we already have.
   amd64 needs the XP x64 driver; the i386 build would take the 32-bit XP one. Old NVIDIA
   cards work the same way with NVIDIA's XP drivers. *(unverified)* Exact last-supporting
   driver versions.
2. **WDDM 1.1: the card's Windows 7-era driver.** *(unverified)* AMD's support for this
   card family ended around Windows 7/8.1 with a WDDM 1.1–1.3 driver. That is a much
   smaller dxgkrnl than WDDM 2.x: no GPU virtual memory, a simpler scheduler. WDDM has
   version negotiation, so the driver declares which versions it speaks and dxgkrnl
   tells it which one it gets.
3. **WDDM 2.x: a modern NVIDIA card.** Adds GPU virtual memory and the newer scheduler.

### Oracles

There's no C to translate here, which is new for this project. These replace ReactOS
C as the correctness reference:

- **The documented interface.** Every dxgkrnl ↔ driver call is public in the WDK
  (`d3dkmddi.h`) and Microsoft's docs, because third parties write the drivers. What isn't
  documented is dxgkrnl's internal behaviour: call order, timing.
- **Real Windows as a black-box oracle.** Run Windows on the same box with the same driver
  and record the call trace at the driver boundary: dxgkrnl's event tracing (what GPUView
  reads) plus kernel-debugger breakpoints on the vendor driver's entry points. Then make
  our dxgkrnl drive the same driver and compare traces: `difftest-contract` at the driver
  boundary. **Clean-room rule: documented interfaces and observed traces only; never
  disassemble Microsoft's binaries.** Where exactly that line sits is the owner's call.
- **Open-source WDDM drivers, both sides visible, in QEMU:** Microsoft's KMDOD sample
  (display-only, the smallest real WDDM driver), then the virtio-gpu and QXL WDDM drivers
  (`viogpudo`, `qxldod`). Most of dxgkrnl's basics can be built and smoke-tested in the
  normal QEMU harness before any real GPU is involved.

### Hardware safety

With a vendor driver on board, our bugs are bugchecks, hangs, black screens and GPU-hang
timeouts, not damage. The vendor driver does all clock, voltage, fan and power
programming, and GPUs throttle and shut down on their own firmware. Rules for agents:

- **Never write firmware:** not the card's VBIOS, not the board's flash. That is the
  only path in this plan that can brick hardware.
- **We host vendor drivers; we do not program GPUs.** Writing our own GPU driver is
  where real risk starts (voltage regulators, clocks). It is out of scope.
- **Scratch disk only.** Bad DMA addresses can scribble over RAM and from there over the
  disk. That's data loss, not hardware damage.
- **LCD monitors only.** A bad mode shows "out of range"; only a CRT could be harmed.

---

## Boss 3: the latest Google Chrome

Wine runs Chrome on Linux, and nearly all of what that takes is user-mode DLLs we already
take from Wine or can: DirectWrite (Chrome's only text renderer on Windows), WASAPI,
`bcrypt`, `winhttp`, `iphlpapi`, `combase`, the `api-ms-win-*` forwarders. What Wine does
not give us is whatever Wine does on the Linux side of its kernel boundary. For us, that's
ntoskrnl work, and it's mostly the sandbox.

### What it needs, in rough order

1. **To think it's on Windows 10.** Chrome 110+ refuses older versions and is manifested
   for Windows 10. Reporting 10.0 is the obvious fix, with side effects elsewhere in the
   OS to manage.
2. **The Windows 10 user-mode API surface**: Vista thread pools and SRW locks, Windows 8's
   `WaitOnAddress` and `GetSystemTimePreciseAsFileTime`, Windows 10's
   `SetThreadDescription`. *(unverified)* PartitionAlloc's use of `VirtualAlloc2` /
   `MapViewOfFile3` (Windows 10 1803). Most of this is in Wine; ReactOS's `*_vista` DLLs
   cover part of it.
3. **The sandbox** (below). The real gap.
4. **Graphics is not a blocker.** Chrome composites with D3D11 through ANGLE, but with
   `--disable-gpu`, or no usable GPU, it uses SwiftShader, its built-in software renderer.
5. **The rest is moderate:** Winsock with I/O completion ports, network change
   notifications, WinHTTP for proxy settings, Crashpad (exception handlers and minidumps),
   TSF for IME input. Chrome brings its own TLS (BoringSSL) and root store.
6. **Skip the installer at first.** Its installer and updater need services, the Task
   Scheduler COM API and a lot of COM. A portable Chromium zip avoids all of it.

### The sandbox: the contract, not Windows' internals

Chrome's sandbox has the same shape on Windows and Linux: a privileged **broker** plus a
restricted **target**, with blocked calls forwarded to the broker. **Chrome writes the
broker on both platforms.** Our share is four kernel primitives, and three of them have a
Linux design to copy.

**Contract only** (one-way restrictions: Chrome checks that setup succeeded, then only
needs forbidden things to fail):

| Windows feature | What Chrome needs | Linux design to copy |
|---|---|---|
| win32k lockdown (`ProcessSystemCallDisablePolicy`) | Renderers' win32k system calls fail | **seccomp**: a per-process filter flag checked in the syscall dispatcher |
| Job objects | One-process limit (renderers can't spawn), UI limits, memory caps | cgroups / `RLIMIT_NPROC` |
| Named-object isolation (AppContainer) | A private object namespace per container | mount and IPC namespaces |
| Integrity levels / restricted tokens | Documented access-check results: low integrity can't write up; restricted tokens are checked twice | **No Linux analogue.** The NT model is its own, but the algorithm is documented and our `Se` access check is already Rust; integrity levels are an addition to it |

**Bring-up:** because these are restrictions, Chrome *works* if the calls succeed before
we enforce them. Phase one accepts them, **loudly marked not enforced** (a log line and a
status flag, never a quiet success). Real enforcement follows. An unenforced sandbox must
never read as a working one.

**Mechanism** (must match Windows exactly):

- **ntdll stub bytes.** The sandbox patches a child's ntdll in memory before it runs:
  it recognises Windows' exact syscall-stub byte pattern and redirects the call to the
  broker. Our stubs for the intercepted calls (`NtCreateFile`, `NtOpenFile`,
  `NtOpenProcess`, `NtMapViewOfSection`, …) must match Windows 10's layout, including the
  `KUSER_SHARED_DATA` syscall-mode check, so that field must exist at the right offset.
  *(unverified)* Whether an unrecognised pattern is fatal: read
  `sandbox/win/src/service_resolver*` in Chromium.
- **Creating the child process.** `CreateProcess` with the extended attribute list
  (inherited-handle list, mitigation policies, AppContainer capabilities). Then the
  parent writes into the suspended child and duplicates handles into it before resuming
  it. These are the cross-process primitives the spawn32 WoW64 ladder guest already tests.
- **Token API details**: specific SIDs and info classes, `NtCreateLowBoxToken`.
- **A separate desktop for renderers**: window-station and desktop object security in
  win32k.

### Oracles

- **Chromium's source** (`sandbox/win/src/`): which byte patterns it accepts, which calls
  must succeed, what it checks afterwards. No reverse engineering needed.
- **Chromium's sandbox test binaries** (`sbox_integration_tests`, `sbox_validation_tests`):
  portable Windows programs that check the OS enforces each restriction. They make a
  ready-made conformance suite for the sandbox rung, like the WoW64 parity guests.

### The ladder

A smoke-harness ladder test, like spawn32:

1. `chrome --headless --no-sandbox --dump-dom file://…`: no UI, no GPU, the smallest real run
2. The same against an `https://` URL (networking, TLS, DNS)
3. A normal Chrome window with `--no-sandbox --disable-gpu`
4. Sandbox on, unenforced first and then enforced. The kernel work lands here
5. GPU on, when Boss 2 gets there

Rungs 1–3 are mostly user-mode Wine plumbing: probably the fastest "wow" in this whole
doc. Chrome also ships native ARM64 Windows builds, so the arm64 port is a real target
too: Chrome on a Pi.

---

## The test bench: OptiPlex 7010 (homelab)

- Has an **NVMe drive on a PCIe adapter**, booted through **Libreboot** (owner flashed
  it). So the NVMe miniport gets real hardware from day one.
- Libreboot's payload is SeaBIOS, not UEFI. **amd64 boots from legacy BIOS again**
  (restored 2026-10-05, gated as `smoke:amd64-bios`; see `bios-boot.md`). That was
  built for the owner's OptiPlex 9020, which runs Libreboot with SeaBIOS.
- First milestone: amd64 boots on it at all, using the integrated Intel graphics through a
  firmware framebuffer. We have only ever booted x86 in QEMU.
- To make it an agent bench like the Pi 4 (`pi4-bench.md`): **remote power** (a smart
  plug), **a serial console** for kernel output (*(unverified)* whether this board has a
  port), and **network boot** so test builds boot without re-flashing anything.

---

## Order of attack

1. Bench bring-up: amd64 boots on the 7010 with firmware graphics.
2. Rung 0: the import-gap census tool and the Windows 10 export surface it ranks.
3. In parallel, by owner priority:
   - **Chrome rungs 1–3** (mostly Wine plumbing; the cheapest big demo)
   - **NVMe**, then **KMDF on real drivers**, then NDIS 6, then audio
   - **GPU rung 1**, the XP x64 driver
4. Chrome sandbox (the four primitives, plus the stub bytes).
5. dxgkrnl, oracle-first: KMDOD in QEMU, viogpu, then the HD 5450 on WDDM 1.1, then 2.x.
6. Wi-Fi, laptop input, UMDF.
