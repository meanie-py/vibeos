# Driver Ecosystem Roadmap

> **Partly stale (checked 2026-10-05).** KMDF is built (`vendor/wdf`, `wdf01000.sys`,
> KMDF 1.17) and xHCI exists as a Rust-native driver (`usbxhci`, USB 3 working on the
> Pi 4); this page still lists both as missing. The current plan for Windows 10 drivers,
> real GPUs (XDDM → WDDM 1.1 → 2.x) and Chrome is
> [`../engineering/modern-windows-compat-roadmap.md`](../engineering/modern-windows-compat-roadmap.md).
> This page gets a full refresh in the next docs pass.

## Current State

ReactOS uses **WDM (Windows Driver Model)** — the NT5/XP-era driver framework. This supports:
- Storage: IDE/AHCI via UniATA, basic disk/partition/filesystem stack
- Network: A few NIC drivers (Realtek, some Intel), NDIS 5
- USB: OHCI/UHCI/EHCI host controllers, basic HID (keyboard/mouse)
- Display: VGA, VESA framebuffer, VBox/VMware SVGA virtual GPUs
- Audio: AC97, SB16
- Input: PS/2, basic USB HID

This is roughly equivalent to a Windows XP install with no third-party drivers.

## The Driver Model Gap: WDM → WDF

### What Happened

In Windows Vista (2006), Microsoft introduced **WDF (Windows Driver Frameworks)** — a higher-level abstraction layer on top of WDM. WDF has two variants:

- **KMDF (Kernel-Mode Driver Framework):** A library (`wdf01000.sys`) that handles PnP, power management, I/O queues, DMA, and other boilerplate. The driver developer writes callbacks, KMDF handles the state machine.
- **UMDF (User-Mode Driver Framework):** Runs drivers in user mode for stability. Used for USB devices, sensors, some HID.

**Almost all modern Windows drivers are WDF-based**, not raw WDM. This means:
- Even if ReactOS has a working WDM stack, modern drivers won't load without WDF
- The good news: Microsoft **open-sourced KMDF/UMDF** in 2018 under MIT license at `github.com/microsoft/Windows-Driver-Frameworks`

### The KMDF Porting Opportunity

KMDF is a library that sits on top of WDM. It calls into the kernel via standard WDM APIs (IoCreateDevice, KeAcquireSpinLock, ExAllocatePool, etc.) — all of which ReactOS already implements.

**Porting KMDF would be a massive unlock.** It would make a large class of modern Windows drivers loadable without modification:
- NVMe storage drivers (most are KMDF)
- USB 3.0 xHCI host controller (KMDF)
- Many network adapter drivers (KMDF + NDIS 6)
- Audio HD Audio class driver (KMDF)
- Many sensor/HID drivers (KMDF/UMDF)

**Effort estimate:** The WDF source is ~200K lines of C. However, much of it is test code, UMDF (lower priority), and platform abstraction. The core KMDF library is perhaps ~50-80K lines. It has clear interfaces to the underlying WDM layer.

**AI feasibility:** High. The code is C, well-structured, MIT-licensed, and the WDM interfaces it calls are already implemented in ReactOS. This is a "port and fix what breaks" task, not a "design from scratch" task. The main challenge would be handling version negotiation (drivers request specific WDF versions) and any internal APIs it uses that ReactOS doesn't expose.

---

## Per-Category Driver Assessment

### Storage

| Feature | Status | What's Needed | Priority |
|---------|--------|---------------|----------|
| IDE/PATA | Working | UniATA driver | — |
| AHCI/SATA | Working | UniATA driver | — |
| NVMe | **Working** (MSI-X interrupts, 2026-10-07) | `nvme.sys`, Rust, all arches; see `docs/engineering/modern-windows-compat-roadmap.md` | **Critical** |
| USB Mass Storage | Basic | Working for USB 2.0 | — |
| SD/eMMC | Missing | Class driver needed | Medium |
| VirtIO-blk | Working (ARM64) | Already ported for QEMU | — |

**NVMe is critical.** No modern PC boots without it. Options:
1. Port the Microsoft StorNVMe driver (requires KMDF)
2. Write a minimal WDM-based NVMe miniport from spec (NVMe spec is public, ~200 pages)
3. For VMs, VirtIO-blk/SCSI already works

### Network

| Feature | Status | What's Needed | Priority |
|---------|--------|---------------|----------|
| Ethernet (basic) | Working | A few NIC drivers | — |
| Modern Ethernet | Partial | NDIS 6 for modern NIC drivers | High |
| WiFi | Missing | NDIS 6 + Native WiFi model | **Critical** for laptops |
| VirtIO-net | Not ported | Simple spec, good for VMs | Medium |
| Bluetooth | Missing | Bluetooth stack + HCI driver | Low |

**WiFi is the laptop killer.** Without WiFi, no laptop user will try the OS. The dependency chain:
1. NDIS 6 miniport interface (replaces NDIS 5 completely)
2. Native WiFi filter driver (802.11 framing, WPA/WPA2)
3. A WiFi NIC miniport driver

This is a large effort. NDIS 6 is a near-complete rewrite of the network driver interface. However, it's well-documented in the WDK docs.

### GPU / Display

| Feature | Status | What's Needed | Priority |
|---------|--------|---------------|----------|
| VGA/VESA | Working | Basic framebuffer | — |
| VBox/VMware SVGA | Working | Virtual GPU in VMs | — |
| VirtIO-GPU | Missing | Miniport driver for QEMU virtio-gpu | High (VM) |
| Software rendering | Basic (GDI) | Mesa llvmpipe/softpipe port | Medium |
| WDDM (native GPU) | Missing | Entire dxgkrnl.sys subsystem | Long-term |
| Direct3D 9 | Basic (Wine) | Software rasterization via Wine/WineD3D | Medium |
| Direct3D 11/12 | Missing | DXGI + D3D11 runtime + WDDM | Long-term |
| OpenGL | Basic (Wine) | Software/Mesa path | Medium |

**GPU acceleration paths (ordered by feasibility):**

#### Near-term: VirtIO-GPU (VM only)
- QEMU's virtio-gpu device provides a virtual GPU with 2D/3D support
- Write an XDDM miniport that maps virtio-gpu commands
- virglrenderer on the host provides OpenGL acceleration
- **Effort:** Medium — virtio-gpu spec is public and simple
- Good for development, demos, and VM-based usage

#### Medium-term: Software Rendering (Mesa)
- Port Mesa3D's `llvmpipe` (LLVM-based software rasterizer) to ReactOS/PE
- Provides OpenGL 4.x and Vulkan 1.x in software
- Slow but runs on any hardware, no driver needed
- Wine's WineD3D can translate D3D9→OpenGL, giving basic game compat
- **Effort:** Large — Mesa has Unix dependencies but there are Windows ports

#### Long-term: WDDM + Native GPU Drivers
- **dxgkrnl.sys:** The DirectX Graphics Kernel subsystem. Manages GPU scheduling, memory, display output. ~100K+ lines.
- **DXGI:** DirectX Graphics Infrastructure. Swap chains, display enumeration, format negotiation.
- **D3D11/12 runtime:** User-mode runtime that talks to dxgkrnl via DXGI.
- This is a massive undertaking (500K+ lines across all components)
- Even Wine doesn't implement WDDM — they bypass it entirely with their own D3D→Vulkan/OpenGL translation
- **Realistic only with significant community involvement**

**Note on using Vulkan to power drivers:** We can't use Vulkan as an abstraction layer for GPU drivers because Vulkan itself needs a GPU driver to function. It's circular — Vulkan sits on top of the driver stack, not underneath it. The Linux approach works because Linux has its own kernel GPU driver framework (DRM/KMS) with open-source drivers (mesa/nouveau/radeonsi). Those are Linux kernel modules and can't be used in a Windows/PE environment.

### Audio

| Feature | Status | What's Needed | Priority |
|---------|--------|---------------|----------|
| AC97 | Working | Legacy codec | — |
| SB16 | Working | Legacy (VM only) | — |
| HD Audio (HDA) | Basic | Class driver exists but incomplete | High |
| USB Audio | Missing | USB Audio class driver | Low |

HD Audio is the standard on all modern PCs. ReactOS has a basic HDA driver. Improving it to handle more codec variants would cover most hardware. HDA is a well-specified standard with a public spec.

### USB

| Feature | Status | What's Needed | Priority |
|---------|--------|---------------|----------|
| OHCI (USB 1.x) | Working | Host controller driver | — |
| UHCI (USB 1.x) | Working | Host controller driver | — |
| EHCI (USB 2.0) | Working | Host controller driver | — |
| xHCI (USB 3.0) | Working in QEMU (USB2-protocol ports: keyboard/mouse) | SuperSpeed ports, devices behind hubs, bulk/isoch, the Pi 4 VL805 | **Critical** |
| HID | Basic | Keyboard/mouse work | — |
| Mass Storage | Working | USB sticks, external drives | — |
| Hub support | Working | Basic hub driver | — |

**xHCI is critical.** Modern PCs often have ONLY USB 3.0 ports (xHCI controller), and the Raspberry Pi 4's only host ports are xHCI. `usbxhci.sys` (2026-09-26) is a usbport miniport written in Rust, not a port of Microsoft's KMDF USBXHCI + UCX stack, so it does not wait on KMDF: keyboards and mice enumerate and type through it in QEMU on arm64 (`VIBEOS_QEMU_XHCI=1`). Design, milestones and what is still missing: `docs/engineering/usbxhci-design.md`.

### Input

| Feature | Status | What's Needed | Priority |
|---------|--------|---------------|----------|
| PS/2 keyboard | Working | i8042 driver | — |
| PS/2 mouse | Working | i8042 driver | — |
| USB HID | Working | Basic HID class driver | — |
| Touchpad | Basic | Precision Touchpad needs HID improvements | Medium |
| Touch screen | Missing | HID touch digitizer support | Low |
| VirtIO input | Working (ARM64) | Custom driver for QEMU | — |

Input is in decent shape. The main gap is precision touchpad support (multi-touch gestures), which is HID-based and incrementally improvable.

---

## Strategy

### Phase 1: VM-Ready (Pre-Launch)
Focus on virtual hardware — make VibOS work perfectly in VMs:
- VirtIO-blk (done for ARM64, port to i386/amd64)
- VirtIO-net (simple virtio NIC)
- VirtIO-GPU (2D acceleration in QEMU)
- Improve VBox/VMware SVGA support
- This gives a usable demo/development environment

### Phase 2: KMDF Port (High Priority, AI-Driven)
Port Microsoft's open-source KMDF to ReactOS:
- MIT-licensed, ~50-80K lines of core code
- Unlocks NVMe, xHCI, modern HDA, and many other KMDF-based drivers
- Highly automatable — it's a C library calling WDM APIs we already implement
- Test against specific KMDF driver samples from the WDK

### Phase 3: Critical Hardware Drivers (Post-KMDF)
With KMDF in place:
1. **NVMe** — load/test Microsoft's StorNVMe or write a minimal one
2. **xHCI** — load/test KMDF-based xHCI driver
3. **HD Audio** — improve existing driver
4. **NDIS 6** — unblock modern network drivers

### Phase 4: Community-Driven Hardware (Post-Launch)
- WiFi (NDIS 6 + Native WiFi)
- Bluetooth
- GPU acceleration (VirtIO-GPU → Mesa → WDDM long-term)
- USB Audio, webcams, sensors

---

## What's Automatable

| Component | Size | AI Feasibility | Notes |
|-----------|------|----------------|-------|
| KMDF port | 50-80K lines | High | MIT source, clear WDM interfaces |
| VirtIO drivers | ~5K lines each | High | Simple specs, well-documented |
| NVMe miniport | ~10K lines | High | Public spec, straightforward |
| xHCI driver | ~15K lines | Medium-High | Public spec, complex but mechanical |
| NDIS 6 | ~50K lines | Medium | Public docs, complex state machine |
| HD Audio improvements | ~5K lines | High | Public spec, incremental fixes |
| Mesa port | Very Large | Medium | Unix→PE porting, many dependencies |
| WDDM (dxgkrnl) | 100K+ lines | Low-Medium | Massive, poorly documented internals |
| WiFi stack | ~30K lines | Medium | Public 802.11 specs but complex |
