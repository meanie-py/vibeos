# Windows API Compatibility Roadmap

## Current State

ReactOS targets **NT 5.2 (Windows Server 2003)**. The kernel, Win32 subsystem, and user-mode DLLs implement this era's API surface. Windows 11 is **NT 10.0** — 20 years of API additions separate them.

However, the NT kernel API is remarkably stable. Most Nt* syscalls from 2003 still exist in Windows 11 unchanged. The gaps are mostly in user-mode libraries, new subsystems, and higher-level frameworks.

## Compatibility Tiers

### Tier 1: Classic Win32 Apps (Mostly Working Now)

**Target:** Applications built against Win32 API circa XP/2003 — no .NET, no WinRT, no modern runtime dependencies. Think: older business software, utilities, many open-source Windows ports, games from the 2000s era.

**What works:**
- Core Win32 GUI: CreateWindow, message loop, GDI drawing, dialogs
- File I/O: CreateFile, ReadFile, WriteFile, memory-mapped files
- Process/thread management: CreateProcess, CreateThread
- Registry: RegOpenKey, RegQueryValue, etc.
- Basic networking: Winsock 2
- Basic COM: CoCreateInstance, IUnknown, basic OLE
- Shell: basic Explorer shell, taskbar, file associations
- Console apps: cmd.exe, console API

**What's missing/buggy (fixable):**
- Many Win32 API stubs that return ERROR_CALL_NOT_IMPLEMENTED
- Various GDI rendering edge cases (font rendering, complex clipping)
- Print subsystem gaps
- Some security descriptor / ACL edge cases
- MSI installer support is partial

**AI-automatable:** Yes — these are well-documented APIs with clear specs. Most fixes are "implement the stub" or "fix the edge case." MSDN has complete documentation. ReactOS already has Wine-derived implementations for many DLLs that just need updating.

---

### Tier 2: Vista/Win7 Era Apps (Achievable, High Impact)

**Target:** Applications requiring Vista+ APIs. This covers most "modern" desktop software that isn't UWP — Chrome, Firefox, VS Code (Electron), Discord, Slack, most Steam games, Office 2016+.

**Key missing components:**

#### kernelbase.dll (Vista+)
- In Vista, Microsoft split `kernel32.dll` — many functions moved to `kernelbase.dll` with kernel32 becoming a forwarder
- Most modern apps link against kernelbase
- **Effort:** Medium. The functions already exist in kernel32, this is mostly a reorganization + adding some new APIs
- **AI-automatable:** Yes — it's plumbing. The function signatures are documented, the implementations already exist in kernel32

#### ALPC (Advanced Local Procedure Call)
- Replaced the NT5 LPC mechanism in Vista
- Used by RPC, COM, LSASS, and many system services
- ReactOS still uses LPC
- **Effort:** Large but well-specified. The ALPC interface is documented in the NDK and reverse-engineered extensively
- **AI-automatable:** Partially. The protocol is complex but mechanical

#### api-ms-win-* API Sets
- Modern Windows has hundreds of "API set" DLLs that are just forwarder shims
- Apps link against `api-ms-win-core-file-l1-1-0.dll` instead of `kernel32.dll`
- The loader resolves these to real DLLs at runtime
- **Effort:** Small per DLL (they're just export forwarding tables), but there are hundreds
- **AI-automatable:** Highly. Each API set is a simple mapping file. Could be generated in bulk

#### Universal CRT (ucrtbase.dll)
- VS2015+ apps link against ucrtbase.dll instead of msvcrt.dll
- Many modern apps bundle vcruntime140.dll + ucrtbase.dll
- **Effort:** Medium. ucrtbase is a refactored msvcrt with some new functions
- **AI-automatable:** Yes — well-documented C runtime functions

#### SxS (Side-by-Side) Assemblies & Manifests
- Modern apps use application manifests for DLL versioning, DPI awareness, visual styles
- ReactOS has basic manifest support but many gaps
- **Effort:** Medium
- **AI-automatable:** Yes — XML parsing + loader behavior, well-documented

#### NDIS 6.x (Networking)
- Vista replaced NDIS 5 with NDIS 6 (completely different driver model for network adapters)
- Required for modern network drivers (WiFi especially)
- **Effort:** Large. Different miniport interface, new filter driver model, new OID set
- **AI-automatable:** Partially. The NDIS 6 spec is public but complex

#### DWM (Desktop Window Manager)
- Vista+ compositing window manager
- Not required for most apps to function — they fall back to classic rendering
- But some apps query DWM state or use DWM-specific APIs (glass, thumbnails)
- **Effort:** Very large. Requires a GPU-accelerated composition engine
- **AI-automatable:** Difficult. Visual output verification needs screenshot-based iteration which is slow and context-heavy
- **Skip for now:** Most apps work without DWM. Stub the query APIs to report DWM disabled.

#### COM Enhancements
- COM is Component Object Model — Microsoft's binary interface standard for reusable components
- Underpins the registry, shell extensions, drag-and-drop, clipboard, OLE, ActiveX, and WinRT
- ReactOS has basic COM via Wine. Missing: COM activation contexts, DCOM improvements, WinRT activation
- **Effort:** Incremental. COM is extremely well-documented (MSDN has every interface)
- **AI-automatable:** Yes — each COM interface is a vtable with documented methods. Very mechanical

---

### Tier 3: Win10/Win11 Modern Apps (Long-Term)

**Target:** UWP/WinUI apps, Microsoft Store apps, modern Windows features.

**Key missing components:**

#### WinRT (Windows Runtime)
- Entirely new component model built on top of COM
- Used by UWP apps, WinUI, and increasingly by Win32 apps via C++/WinRT
- Requires: RoActivateInstance, IInspectable, Windows.* namespace activation
- **Effort:** Enormous. WinRT is a full runtime with metadata, activation factories, async patterns
- **Not a priority for VibOS launch**

#### DXGI / Direct3D 11/12
- Modern graphics pipeline (see driver roadmap)
- **Blocked by WDDM driver infrastructure**

#### Windows.UI.Composition
- Modern UI composition layer, replaces DWM for new apps
- **Not a priority**

#### WIC (Windows Imaging Component)
- Modern image codec framework
- ReactOS has basic WIC via Wine
- **Incrementally improvable**

#### DirectWrite / Direct2D
- Modern text rendering and 2D graphics
- Many apps fall back to GDI if unavailable
- **Medium priority — needed for good font rendering in modern browsers**

---

## Strategy

### Phase 1: Solidify Tier 1 (Pre-Launch)
- Fix remaining Win32 API stubs in critical paths
- Ensure the apps we demo at launch work reliably
- Use `verify_constants.py` pattern: build tools that automatically detect API gaps when specific apps fail

### Phase 2: Tier 2 Core (Post-Launch, AI-Driven)
Priority order based on app compatibility impact:
1. **API set forwarder DLLs** — unblocks many modern apps immediately, highly automatable
2. **kernelbase.dll** — forwarder + new APIs, unblocks Vista+ apps
3. **ucrtbase.dll** — unblocks VS2015+ compiled apps
4. **SxS manifests** — unblocks apps with DPI awareness, visual styles
5. **NDIS 6** — unblocks modern network drivers (see driver roadmap)
6. **ALPC** — unblocks modern system services

### Phase 3: Tier 2 Extended (Community)
- COM enhancements
- DirectWrite/Direct2D (could use Wine's implementation as starting point)
- Better GDI compatibility
- Print subsystem

### Phase 4: Tier 3 (Long-Term)
- WinRT (if demand exists)
- DXGI/D3D11 (coupled with WDDM driver work)

## What's Automatable

| Component | Size | AI Feasibility | Notes |
|-----------|------|----------------|-------|
| API set DLLs | Small × 200 | Very High | Forwarding tables, can be generated |
| kernelbase.dll | Medium | High | Function moves + new APIs, well-documented |
| ucrtbase.dll | Medium | High | C runtime, exhaustively specified |
| COM interfaces | Large | High | Mechanical vtable implementations, full MSDN docs |
| SxS manifests | Medium | High | XML parsing, documented loader behavior |
| ALPC | Large | Medium | Complex protocol but well reverse-engineered |
| NDIS 6 | Large | Medium | Public spec, complex state machine |
| DWM | Very Large | Low | Visual output, GPU composition, screenshot debugging |
| WinRT | Enormous | Medium | Well-documented but massive surface area |
