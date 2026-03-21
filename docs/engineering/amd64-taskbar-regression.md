# amd64 Taskbar/Shell UI Regression

## Problem

On amd64, the kernel boots successfully and explorer.exe starts, but the shell UI (taskbar, desktop icons, LiveCD wizard) does not render. The screenshot shows only a blue wallpaper with no interactive elements.

**i386 renders the full desktop correctly** (Start button, taskbar, clock). ARM64 is not testable for this (ramdisk boot, separate display pipeline).

## Evidence

- `qemu/amd64_smoke.png` — blue wallpaper only, no taskbar
- `qemu/i386_smoke.png` — full desktop with Start button, taskbar, clock at 12:00 AM
- `qemu/amd64_desktop.png` (Feb 18, 2026) — old known-good: full desktop with taskbar, icons, LiveCD wizard

## What Works

- Kernel boots fully (MM, PS, IO, PnP all initialize)
- `"explorer.exe created"` serial marker fires (winlogon successfully launches explorer.exe)
- `"[WIN32K] shell ready"` serial marker fires (co_IntShowDesktop renders the desktop window)
- The blue wallpaper IS painted (GDI/display pipeline works)

## What's Broken

- No taskbar (explorer.exe's tray window doesn't render or isn't visible)
- No desktop icons (My Computer, Network Places, etc.)
- No LiveCD setup wizard
- User-mode DbgPrint doesn't work (DUMMY_PSEH eats the debug exception), so we can't trace explorer.exe internally via serial

## Timeline

The regression was introduced sometime between:
- **Last known good:** commit `5802b15e` (Feb 18, 2026) — "fix amd64+ARM64 desktop boot: all 3 arches pass smoke tests 4/4"
- **First known bad:** the IO subsystem Rust translation work (March 2026 sessions)

The `"explorer.exe created"` boot marker was hiding this regression because it only proves the process was launched, not that the shell rendered.

## Theories

### A. IO subsystem Rust translation broke a syscall explorer.exe depends on
Explorer.exe calls Nt* syscalls (NtCreateFile, NtQueryDirectoryFile, etc.) to enumerate desktop items and load shell extensions. If any of these return wrong results or fail silently on amd64, the shell initialization would fail without a visible error.

### B. File object or IRP handling regression
The IO translation changed file.rs, iofunc.rs, irp.rs, device.rs etc. A subtle difference in IRP completion, file object handling, or device IO control could cause explorer.exe's shell COM objects to fail initialization.

### C. PnP/device enumeration difference
With devaction.rs and pnpmgr now in Rust on amd64, device enumeration might produce different results that affect the shell (e.g., shell service objects that depend on device notifications).

### D. Win32k/user-mode callback issue
The amd64 Rust usercall.c workaround (Rust usercall.rs has undiagnosed return-address corruption) might have side effects on KeUserModeCallback dispatching, which explorer.exe uses heavily for shell window creation.

## Suggested Debug Approach

1. **Git bisect** between commit `5802b15e` (last known good) and current HEAD to find the exact commit that broke the taskbar
2. **Compare serial output** between i386 (working) and amd64 (broken) after `explorer.exe created` — look for error messages or missing initialization steps
3. **Try reverting specific IO modules** to C on amd64 one at a time to isolate which Rust translation caused the regression
4. **Check if the LiveCD wizard process starts** — look for `reactos.exe` or `setup` in the serial output

## Reference Files

| File | Description |
|------|-------------|
| `/home/curi/Projects/os-playground/reactos/CLAUDE.md` | Project instructions |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground/memory/MEMORY.md` | Memory index |
| `/home/curi/Projects/os-playground/qemu/amd64_smoke.png` | Current broken screenshot |
| `/home/curi/Projects/os-playground/qemu/amd64_desktop.png` | Old known-good screenshot (Feb 18) |
| `/home/curi/Projects/os-playground/qemu/i386_smoke.png` | Working i386 reference |
| `/home/curi/Projects/os-playground/reactos/win32ss/user/ntuser/desktop.c` | co_IntShowDesktop (shell ready marker) |
| `/home/curi/Projects/os-playground/reactos/base/system/winlogon/winlogon.c` | explorer.exe launch (line 822) |
| `/home/curi/Projects/os-playground/reactos/base/shell/explorer/explorer.cpp` | Explorer shell init (CreateTrayWindow, DesktopCreateWindow) |
| `/home/curi/Projects/os-playground/reactos/tests/qemu/vm.py` | Smoke test boot markers |
