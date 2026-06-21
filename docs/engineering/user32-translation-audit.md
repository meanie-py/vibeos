# user32.dll C→Rust Translation Audit

Lane: **user32** (branch `user32-port`). Owner: the `user32` swarm agent.
Created **2026-06-20**.

> **2026-06-20 — Wave 0 + Wave 1 core DONE (i386 cold-clean GREEN).** 9 `.c` files now
> Rust and cohab-linked into user32.dll: `misc/timer.c` (leaf) + the windowing core
> `windows/{caret,dc,nonclient,paint,winpos,class,window,message,defwnd}.c`. The
> **message-dispatch core** (Send/Post/Get/Dispatch/PeekMessage + the wndproc call path
> through the i386 `wndproc_fixup.S` thunk) and the **default window proc** run in Rust
> through boot. Crate compiles **0-error on all 3 arches**; i386 **cold `--clean` smoke
> ✓boot** (1m51s). amd64/arm64 boot pending the ×3 landing gate (control-plane; arm64 via
> explicit user32.dll relink+inject, like kernel32/win32k).
>
> **Substrate built (the reusable foundation for all future waves):**
> - `sdk/lib/rust/user32-rust/` — staticlib cohab crate (mirrors kernel32-rust). Depends on
>   **win32k-sys** (the bindgen NtUser*/Nt* + win32k struct-layout surface; builds clean for
>   usermode targets — the rustconv opus output targets `win32k_sys::win32k::*` directly).
> - `src/ntwrapper.rs` — port of `include/ntwrapper.h` (143 `NtUserx*`/`NtUser*` inline
>   wrappers over the 8 `NtUserCall*` multiplexers + routine consts).
> - `src/internal.rs` — shared user32-internal substrate: ntdll `Rtl*`/`Zw*` externs,
>   `NtCurrentTeb`/`NtCurrentPeb`/`RtlGetProcessHeap` inlines, the `ValidateHwnd` family +
>   UserApiHook trio (cohab FASTCALL externs to surviving C), `guah`/`gfServerProcess`,
>   `LoadUserApiHook`, and the Win32 word macros.
>
> **Pipeline proven** (translate → integrate → fix → boot): rustconv **opus** per-function
> ($0.84 for all 9 files; c2rust is unsupported on win32ss headers — `CDeclId` AST issue,
> same as win32k — so plain opus) → parallel **integration subagents** (grep-verify symbols,
> swap `ntkrnl_sys::ddk`→`internal` externs, `#[no_mangle]` exports) → central residual-fix
> + cross-TU link-resolution subagents → cohab link → boot. See §-notes below for the
> reusable gotchas surfaced (FASTCALL i386-only cfg-split; bindgen-flattened `THRDESKHEAD`
> head → `head.h`/`.pti` via padding offsets; `LARGE_STRING` bitfields; RECT/RECTL distinct
> bindgen types; the `let Root` binding-vs-const-pattern trap from a win32k-sys glob const;
> the false-HANG/transient-BUGCHECK-under-host-contention class).
>
> Status: **Wave 0 + Wave 1 core integrated, i386 cold-green; ×3 landing pending.**

> **2026-06-20 — Wave 2 DONE (dialogs + logon controls; i386 link + boot GREEN).** 8 more
> `.c` → Rust and cohab-linked: `windows/{dialog,messagebox}.c` + `controls/{button,static,
> edit,combo,listbox,scrollbar}.c` (~19.4K LOC). user32.dll now runs **17 `.c` files in Rust**
> (Wave 1's 9 + these 8) — the dialog/message-box path and the six standard built-in controls.
> Crate compiles **0-error on all 3 arches**; i386 cohab **link clean** + **smoke ✓boot**.
> rustconv **opus** translate: **$8.68** for all 8 files (edit.c the big one at 5.5K LOC).
> amd64/arm64 cohab link pending the ×3 gate (control-plane). New reusable gotchas surfaced:
> the `*_builtin_class` cross-TU data exports regcontrol.c reads (need `#[no_mangle] pub
> static`); spec **forwarders** (`InvalidateRect`→`NtUserInvalidateRect` etc.) are NOT
> linkable internal symbols → route through `crate::ntwrapper`; the `IMM_FN(name)` =
> `gImmApiEntries.pname` runtime dispatch table (mirror the Imm32ApiTable layout, don't
> direct-import imm32); `set_control_clipping`/`get_icon_size` are cdecl C (not stdcall);
> `RtlStringCchCopyNW` is a strsafe inline (no ntdll export → reimplement); `InterlockedExchange`
> is an amd64/arm64 intrinsic (no extern symbol → `AtomicI32::swap`).
>
> Status: **Wave 0 + 1 + 2 integrated, i386 link+boot green; amd64/arm64 ×3 landing pending.**

> **What this lane is.** `win32ss/user/user32/` builds **user32.dll**, the usermode
> Win32 windowing API DLL — windows, classes, messages, menus, dialogs, the built-in
> controls (button/edit/listbox/combo/scrollbar/static), DDE, clipboard, MDI, hooks,
> cursors/icons. It is the usermode *client* half of the windowing system; the kernel
> half is **win32k.sys**, which is **already 100% Rust** (see
> `win32k-translation-audit.md`). This lane translates the user32.dll C bodies to Rust
> using the same **cohabitation per-file flip** mechanism kernel32 and the HAL use.

> **Method is settled — we mirror the kernel32 lane.** kernel32.dll (the sibling
> usermode base DLL) is already deep into its Rust wave via a `sdk/lib/rust/kernel32-rust`
> staticlib cohab-linked into the cmake-built DLL with the translated `.c` files commented
> out. user32 gets the identical treatment: a new `sdk/lib/rust/user32-rust` crate, a
> per-file flip in `win32ss/user/user32/CMakeLists.txt`, hardest/most-boot-critical first.
> See **Build Integration** below for the exact recipe.

---

## 1. Scope & Size

`win32ss/user/user32/` — **55 `.c` files, ~59.9K LOC**, three source dirs:

| Dir | Files | LOC | Role |
|---|---:|---:|---|
| `windows/` | 24 | 30,092 | window/class/message/menu/dialog/draw/cursoricon/clipboard/MDI/hook/input/paint/nonclient/defwnd/… — the windowing core |
| `controls/` | 10 | 17,094 | built-in window classes: edit, listbox, combo, scrollbar, button, static, + appswitch/ghost/icontitle/regcontrol |
| `misc/` | 20 | 12,549 | dllmain, the DDE family, desktop/winsta/display, imm, exit, object, timer, stubs, … |

**Out of this lane (for now):**
- `user32.spec` (802 lines, **788 exports**) — the export table. **128 exports forward
  directly to a win32k syscall** (`@ stdcall Foo(..) NtUserFoo`) and have **no C body** —
  the spec redirect resolves them at link/load with nothing to translate. ~660 exports
  have local C implementations (our targets).
- Static sublibs linked into user32.dll but living elsewhere:
  `user32_vista_static` / `user32_optional_vista` (`win32ss/user/user32_vista/`),
  `user32_wsprintf` (`sdk/lib/crt/`, `_USER32_WSPRINTF`). Separate lanes/leave-as-C.
- Shared `win32ss/user/rtl/image.c` + `text.c` — **already translated by the win32k lane**
  (`image.rs` / `rtl_text.rs`, commit `68028a7c719`). user32's CMakeLists statically
  compiles its own copy of these two `.c`; see **§5 Shared rtl** for the de-dup decision.

**user32 is a thin client over win32k.** ~310 distinct `NtUser*` syscalls are referenced
across the `.c` files; most user32 functions marshal arguments and call into win32k (now
Rust). This makes the translation mostly straight argument-shuffling + Wine-derived control
logic, *not* deep algorithm work — but with a lot of surface area and SEH.

---

## 2. Architecture & Gotchas (read before translating)

### 2.1 DUMMY_PSEH — the #1 recurring bug class
All arches use `USE_DUMMY_PSEH` (PSEH3 doesn't work with llvm-mingw). **13 user32 files use
`_SEH2_TRY`** (`message.c` ×14, `window.c` ×10, `class.c` ×5, `hook.c` ×4, `desktop.c` ×3,
plus nonclient/paint/defwnd/draw/text/usrapihk/scrollbar/listbox). Under DUMMY_PSEH the
`_SEH2_EXCEPT` body is **dead code**. The recurring lane-killer (kernel32 IsBad*Ptr, cdfs,
fastfat all hit it): a translated `_SEH2_EXCEPT` body that mutates an output/result must be
**dead code**, not an unconditional block — otherwise the "exception path" runs every call.
Validate pointers *before* deref in any SEH-protected path (the C relied on the filter to
catch the AV; we don't have it). **Grep every translated file for live except bodies.**

### 2.2 i386 `wndproc_fixup.S` — irreducible asm glue, keep as-is
`windows/wndproc_fixup.S` (i386-only) is `CALL_EXTERN_WNDPROC`: it calls app-provided
WNDPROCs that may not honour stdcall stack cleanup, so it captures/restores ESP around the
call. This is exactly the kind of irreducible glue that stays as asm (cf. win32k's
`*_rust_shims.c`). **Do not translate.** The Rust `message.rs` will `extern "C"` it.

### 2.3 ABI per arch (the silent i386 killer)
i386 NTAPI = stdcall (`_Name@N`); amd64/arm64 = cdecl. Use `extern "system"` for exported
NTAPI/stdcall entries (`#[no_mangle] extern "system"` on i686-gnu correctly emits `_Name@N`).
Internal cdecl helpers must be `extern "C"`. A wrong decl links silently on amd64/arm64 and
ESP-drifts on i386 (the kernel32 RPC-wait `TranslateStdHandle` bug). Run `verify_abi.py` /
`verify_ffi.py` after each wave.

### 2.4 User-mode no-SSE rule (i386)
Like kernel32-rust/ntdll-rust, the crate builds for the **standard** per-arch user targets
(`i686-pc-windows-gnu` / `x86_64-pc-windows-gnu` / `aarch64-pc-windows-gnullvm`) with i386
`-sse,-sse2` via `sdk/lib/rust/.cargo/config.toml` (SSE leaking into user-mode caused the
i386 taskbar deadlock).

### 2.5 captures(none) output-pointer writes
`#[no_mangle]` fns writing through an output pointer must `write_volatile` (the
`lint_kernel.py` / scan_ud2 class). user32 is full of `LPRECT`/`LPPOINT`/`out` params.

### 2.6 FFI surface
Imports resolve at runtime to **gdi32, advapi32, kernel32, ntdll, win32k (NtUser*)**, delay
`usp10`/`libpng`. We'll need an FFI decl crate (or reuse `ntdll-sys` + add a `win32u`/NtUser
decl module). Most NtUser* signatures live in `win32ss/include` — candidate for a small
`user32-sys` (bindgen) like `ntdll-sys`, decided in Wave 0. **Settled in Wave 0/1: we reuse
`win32k-sys` directly** (the bindgen `NtUser*`/Nt*/struct surface; the rustconv opus output
targets `win32k_sys::win32k::*`), plus the hand-written `src/ntwrapper.rs` + `src/internal.rs`
substrate. No separate `user32-sys` was needed.

### 2.7 Cohab link-resolution traps (the Wave-2 learnings — read before any wave)
The crate compiling 0-error ×3 is **necessary but not sufficient** — the cohab *link* of
user32.dll exposes a whole class of cross-TU issues the compiler can't see (because the other
half lives in the surviving C objects / the `.spec` / win32k). i386 is the strictest linker
(decorated `_Name@N`), so **do the i386 cohab link first and iterate it to zero undefined
symbols** before handing to the ×3 gate. The recurring traps, with the fix:

- **Spec FORWARDERS are not linkable internal symbols.** ~128 user32 exports are
  `@ stdcall Foo(..) NtUserFoo` in `user32.spec` — the `.def` resolves them at export time;
  there is **no callable `_Foo@N`** inside user32. A translated body that calls `Foo(...)`
  via a direct `extern "system" { fn Foo; }` → `undefined symbol _Foo@N`. **Fix:** route
  through the `crate::ntwrapper` inline (which calls the `NtUser*` syscall). To classify a
  name: `grep "stdcall NAME" user32.spec` — a trailing forward target (`… NtUserNAME`) ⇒
  forwarder ⇒ use ntwrapper; no target ⇒ real own-export ⇒ keep the extern. Hit by
  InvalidateRect, SetWindowPos, ShowWindow, Begin/EndPaint, SetFocus, SetCapture, KillTimer,
  SetTimer, ShowScrollBar, DestroyWindow, MoveWindow, RedrawWindow, SetParent, DeleteMenu, …
- **`*_builtin_class` are cross-TU DATA exports** that the still-C `controls/regcontrol.c`
  reads by address (`&BUTTON_builtin_class` …). The control's Rust file must define a
  `#[no_mangle] pub static NAME_builtin_class: builtin_class_descr` with the **exact C layout**
  (`regcontrol.h`: name/style/procA/procW/extra/cursor/brush) and the WndProcs transmuted to
  `WNDPROC` (`transmute::<extern "system" fn(..), WNDPROC>` — calling-convention-identical per
  arch). The rustconv translator routinely **drops** these statics → re-add from the C.
  Active ones: DIALOG/BUTTON/EDIT/COMBO/COMBOLBOX/LISTBOX/STATIC.
  **Correction (Wave 3b, verified):** `MENU`/`POPUPMENU`/`DESKTOP`/`ICONTITLE` are **NOT**
  live cross-TU symbols — regcontrol.c's `MENU_builtin_class` entry is inside `#if 0`,
  POPUPMENU is comment-only, DESKTOP isn't referenced at all. So `menu.rs`'s
  `POPUPMENU_builtin_class` / `desktop.rs`'s `DESKTOP_builtin_class` stay plain `pub static`
  (no `#[no_mangle]`). Always grep regcontrol.c for the *active* (non-`#if 0`) reference
  before assuming a builtin_class is cross-TU.
- **`IMM_FN(name)` is a runtime dispatch table, not imm32 imports.** `edit.c` calls IMM via
  `IMM_FN(ImmGetContext)` = `gImmApiEntries.pImmGetContext` — a global `Imm32ApiTable` filled
  at runtime in the still-C `misc/imm.c`; **user32 does not import imm32**. A direct
  `extern "system" { fn ImmGetContext; }` → undefined. **Fix:** mirror the `Imm32ApiTable`
  struct layout (`include/immtable.h` field order, up to the highest field used; unused slots
  = pointer pads), `extern "C" { static gImmApiEntries; }`, and inline wrappers that call
  through it.
- **cdecl C helpers wrongly given stdcall.** `set_control_clipping` / `get_icon_size` are
  plain-cdecl C (`_name`, no `@N`); declaring them `extern "system"` asks for `_name@8`.
  **Fix:** `extern "C"`. (Note: `get_icon_size` lives in `cursoricon.c` → becomes a Rust
  export when Wave 3 flips cursoricon; until then it's the surviving C symbol.)
- **strsafe inlines have no exported symbol.** `RtlStringCchCopyNW` is a `strsafe.h __inline`,
  not an ntdll export → reimplement locally (bounded copy + NUL-terminate).
- **MSVC intrinsics have no extern symbol on amd64/arm64.** `InterlockedExchange` (and the
  `_Interlocked*`/`_byteswap`/`_BitScan` family) are inline intrinsics on amd64/arm64 (only
  i386 has the CRT stdcall export). A direct extern links on i386, **fails amd64/arm64**.
  **Fix:** `core::sync::atomic::Atomic*` (e.g. `AtomicI32::swap`). Scan each wave for these
  before the gate — i386-green does NOT imply amd64-green. (`memcpy`/`memset`/`strlen` are
  fine — compiler-rt provides them on all arches.)
- **i386 winuser.h macros vs the W-export.** `SetWindowLongPtrW` (i386: macro → `SetWindowLongW`),
  generic `GetWindowLong`/`SendMessage`/`SystemParametersInfo` (UNICODE macros → the `…W`
  export) — cfg-split to the W-variant / the `class.rs` forwarder; the generic decorated form
  has no symbol on i386.

### 2.8 The wave integration playbook (proven on Waves 1–2)
1. `rustconv translate <file> -m opus -o ~/vibeos-rust-staging/...` per `.c` (parallel, in the
   background; **never kill mid-run** — it re-pays per function). Stage out, integrate manually.
2. Place files in the crate, register modules centrally (`mod.rs`/`lib.rs` — shared files, the
   lead only). Build a small `shared.rs` for any genuinely cross-file consts/structs.
3. Fan out **one integration subagent per file** (source-only, no build — they contend on the
   crate target dir): swap `ntkrnl_sys::ddk`→usermode externs/`internal`, `#[no_mangle]` the
   spec exports + cross-TU WndProcs/builtin_class, FASTCALL cfg-split, neutralize live SEH
   except bodies, **reconstruct any translator-dropped function bodies faithfully from the C**.
4. **Build centrally** (i386 first), then residual-fix rounds (extract per-file error slices →
   targeted fix agents). Compile errors collapse from ~1000s to 0 in 2–3 rounds (most are a few
   shared-symbol roots — fix the root, re-count; don't bail at the big number).
5. **i386 cohab link**, iterate the §2.7 undefined-symbol classes to zero, then `smoke`.
6. Scan for the amd64-intrinsic trap (§2.7); commit; hand the SHA to the control-plane ×3 gate
   (amd64/arm64 cohab link + boot; arm64 needs the explicit user32.dll relink+inject).

---

## 3. Translation Strategy — hardest/most-boot-critical first

Boot exercises a specific user32 slice: **winlogon** puts up the logon dialog (dialog +
messagebox + button/edit/static controls + class/window/message/defwnd/nonclient/paint),
then **explorer** renders the desktop/taskbar (window/menu/draw/cursoricon/winpos). The DDE
family (~5.3K LOC) and exotic controls are **not** boot-critical → deferred to late waves.
We translate the boot path first so smoke (the north star) exercises Rust early, then sweep
the rest.

### Wave 0 — Crate + FFI scaffold (no behaviour change)
Create `sdk/lib/rust/user32-rust` (staticlib, `crate-type=["staticlib"]`), wire the
`add_custom_command` + cohab link into `user32/CMakeLists.txt` (mirror kernel32 §104–158),
stand up the NtUser*/Nt*/Rtl* FFI decls (decide `user32-sys` bindgen vs hand-decls), and
flip **one trivial leaf** end-to-end (candidate: `misc/timer.c` 38 LOC, or `misc/object.c`
80 LOC) to prove the cohab link + ×3 smoke before any bulk work.

### Wave 1 — Window core (boot-critical, hardest)
`windows/window.c` (2003), `class.c` (1820), `message.c` (3453), `defwnd.c` (1295),
`nonclient.c` (247), `winpos.c` (280), `paint.c` (386), `windows/dc.c` (61), `caret.c` (62).
The SEH-heaviest, most boot-exercised files. `message.c` first (hardest + most central; pulls
in `wndproc_fixup.S`).

### Wave 2 — Dialogs + logon controls (boot-critical)
`windows/dialog.c` (2893), `messagebox.c` (1380), `controls/button.c` (1369),
`controls/static.c` (888), `controls/edit.c` (5548), `controls/combo.c` (2318),
`controls/listbox.c` (3290), `controls/scrollbar.c` (1755). Edit/listbox are the big control
bodies; needed for any real dialog interaction.

### Wave 3 — Desktop/menu/draw (explorer path) — **NEXT**
`windows/menu.c` (1776), `draw.c` (1651), `cursoricon.c` (3095), `text.c` (467),
`font.c` (497), `accel.c` (365), `rect.c` (288), `hook.c` (851), `input.c` (1124),
`misc/desktop.c` (660), `display.c` (619). **~11.4K LOC, 11 files.**

This is the **explorer/desktop-render path** (Wave 1+2 were the winlogon path). Notes that
shape the wave:
- **Less SEH than Wave 1** — only `hook.c` (×4), `desktop.c` (×3), `text.c` (×2), `draw.c` (×1)
  use `_SEH2_TRY`; menu/cursoricon/input/font/accel/rect have none. The §2.1 dead-except
  discipline still applies to the four that do.
- **Cross-TU exports to expect** (§2.7): `MENU_builtin_class` is **NOT** live (regcontrol.c's
  ref is `#if 0` — see §2.7 correction), so `menu.rs`'s static stays plain `pub static`. The
  *real* cross-TU exports that Wave 3b had to re-add (`#[no_mangle]`): `MenuInit`/`MenuCleanup`
  (cdecl, called by dllmain.c), `IntLoadPreloadKeyboardLayouts` (cdecl, logon.c),
  `IntNotifyWinEvent` (`@…@20` **fastcall**, dialog/scrollbar). `cursoricon.c` **defines `get_icon_size`** —
  the helper Wave-2 `static_.rs`/`edit.rs` currently `extern "C"` from the surviving C; when
  cursoricon flips it becomes the Rust export (keep the symbol name + cdecl). `draw.c` provides
  `DrawFrameControl`/`DrawEdge`/`DrawState*` and the internal `set_control_clipping` peers that
  Wave-2 controls call — watch for the same forwarder/cdecl classification.
- **`cursoricon.c` is the big/risky one** (3095 LOC): icon/cursor loading, the PNG path
  (`libpng` delay-import — arm64 stubs it, see CMakeLists `arm64_user32_stubs.c`), animated
  cursors, the icon cache. Highest translation risk in the wave; do it last and review.
- **Suggested split:** **3a (small/leaf):** rect, accel, text, font, display (~1.9K, mostly
  geometry + GDI marshalling, no SEH). **3b (core):** menu, draw, input, desktop, hook (~5.4K,
  the explorer interaction path; menu has the builtin_class). **3c (heavy):** cursoricon (3.1K,
  alone). Gate each sub-slice ×3 before the next, same as Wave 2a/2b would have been.
- **Wave 3b done (2026-06-21) — extra learnings (all in the field-manual class):**
  - **bindgen-opaqued structs.** `tagSERVERINFO` buries its `_PERUSERSERVERINFO` tail
    (aiSysMet/hbrGray/argbSystem/dwLastRITEventTickCount…) in `__bindgen_padding_0`;
    access via the shared `crate::internal::gpsi_perusr()` accessor (promoted from defwnd.rs),
    **not** `(*gpsi).field`. `_SECURITY_ATTRIBUTES` is fully opaque (`_unused:[u8;0]`) →
    hand-declare a local `#[repr(C)]` for `nLength/lpSecurityDescriptor/bInheritHandle`.
  - **mixed-sign bindgen consts break `match`.** `WH_*`: positives (WH_CBT=5) are `u32` but
    `WH_MSGFILTER`=-1 is `i32`; `HCBT_*` are `u32`. A `match HookId as u32` arm `WH_MSGFILTER`
    won't typecheck → alias the lone negative (`const WH_MSGFILTER_U: u32 = WH_MSGFILTER as u32`).
  - **C macros translated as calls.** `RGB(r,g,b)` (wingdi.h macro) → translator emits a
    `_RGB@12` extern → undefined; reimplement inline. Same risk for any FOO()-macro.
  - **strsafe `StringCch*`/`StringCb*`** have no exported symbol → use `wsprintfW` (user32
    __cdecl variadic, like messagebox.rs) for the printf cases + a local bounded wide-concat.
  - **`ValidateHandle`/`ValidateHandleNoErr`** are `PVOID FASTCALL` (user32p.h) → `extern "fastcall"`
    (@Name@N on i386), and call args need `as HANDLE` (HMENU/etc. don't coerce).
  - amd64 width: `dwHookData`/`CI_flags` are `ULONG_PTR` (u64) — local mirror vars + flag-const
    bitops must cast `as ULONG_PTR`, else i386-green / amd64-red.

### Wave 4 — Secondary surface
`windows/clipboard.c` (409), `prop.c` (489), `mdi.c` (2513), `spy.c` (2687),
`misc/winsta.c` (442), `imm.c` (1302), `exticon.c` (1023), `resources.c`, `misc.c`,
`exit.c`, `dllmain.c` (739), `usrapihk.c`, `winhelp.c`, `logon.c`, `rtlstr.c`,
`controls/{appswitch,ghost,icontitle,regcontrol}.c`, `misc/stubs.c`.

### Wave 5 — DDE family (deferred — large, not boot-critical)
`misc/ddemisc.c` (2511), `ddeclient.c` (1425), `ddeserver.c` (1114), `dde.c` (314). ~5.4K
LOC of legacy IPC; translate last (or leave-as-C cohab indefinitely if cost/benefit is poor —
user decision at the time).

---

## 4. Build Integration (the cohab recipe — mirror kernel32)

Per-file flip, exactly like `dll/win32/kernel32/CMakeLists.txt:104–158`:

1. **Crate** `sdk/lib/rust/user32-rust/` (`crate-type=["staticlib"]` →
   `libuser32_rust.a`), workspace member. Deps: the FFI decl crate(s).
2. **`add_custom_command`** building `libuser32_rust.a` per-arch:
   `i686-pc-windows-gnu` / `x86_64-pc-windows-gnu` / `aarch64-pc-windows-gnullvm`,
   `CARGO_TARGET_DIR=${BIN}/rust-user32-target`, **`GLOB_RECURSE`** the `src/**/*.rs`
   (non-recursive glob silently used a stale archive in kernel32 — copy their fix).
   `add_custom_target(rust_user32 DEPENDS …)`.
3. **Flip**: comment the translated `.c` out of `list(APPEND SOURCE …)` with a
   `# replaced by Rust (user32-rust/src/<path>.rs)` line (never-delete: the `.c` stays on
   disk). Once a TU's symbols are *all* gone, no `--allow-multiple-definition` needed
   (conutils-style); during a partial swap-in, `--allow-multiple-definition` + lld
   first-match wins.
4. **Link**: append `${USER32_RUST_LIB}` to `target_link_libraries(user32 …)`, add
   `rust_user32` to `add_dependencies(user32 …)`.

**Build/verify each wave ×3 arches** (`./vibeos-cli build --arch <a>` then `smoke`), cold
`--clean` before trusting green or handing off. **Smoke is the north star.**

---

## 5. Shared rtl files (`win32ss/user/rtl/{image,text}.c`)

These two are compiled into **both** win32k.sys (already flipped → `image.rs`/`rtl_text.rs`)
and user32.dll (still the `.c`, via `${REACTOS_SOURCE_DIR}/win32ss/user/rtl/*.c` in user32's
CMakeLists). Options when we reach them: (a) leave user32's copy as C (cheapest — it's a
separate TU in user32's link), or (b) factor the existing win32k Rust translation into a
shared rlib both DLLs link. **Default: leave as C** until there's a reason; flag in `#swarm`
if win32k lane wants to share. Low priority — not boot-blocking.

---

## 6. Cross-lane coordination

- **win32k lane** (Rust, done): user32 calls ~310 `NtUser*` syscalls into it. ABI/struct
  layout of the NtUser* boundary must match — if a syscall signature looks off, check
  win32k's Rust decl, not just the C header. Coordinate in `#swarm`.
- **kernel32 lane**: shares the usermode-DLL cohab pattern + the no-SSE / DUMMY_PSEH /
  ABI gotchas. Reuse their FFI decls (`ntdll-sys`) where possible; steal their CMake recipe
  verbatim.
- **csrss/smss**: user32 talks to the window-server side; watch for CSR message ABI.
- **Smoke serialization**: per AGENTS.md, never run two same-arch QEMU smokes concurrently
  on the host — check `./vibeos-cli ps` and `#swarm` first.

---

## 7. Status Ledger

| Wave | Files | LOC | Status |
|---|---|---:|---|
| 0 — crate + FFI scaffold + 1 leaf | timer | ~40 | ✅ i386 cold-green |
| 1 — window core | window/class/message/defwnd/nonclient/winpos/paint/dc/caret | ~8,200 | ✅ integrated, i386 cold-green; ×3 gate pending |
| 2 — dialogs + logon controls | dialog/messagebox/button/static/edit/combo/listbox/scrollbar | ~19,400 | ✅ integrated, i386 link+boot green; amd64/arm64 ×3 gate pending |
| 3a — leaf (landed) | rect/accel/text/font/display | ~1,900 | ✅ landed on mainline (LAND TRAIN 1) |
| 3b — core | menu/draw/input/hook/desktop | ~6,100 | ✅ integrated; cohab-link ×3 green, i386+amd64 boot green (arm64 compile/link green, flake-tolerant); ×3 gate pending |
| 3c — cursoricon (heavy) | cursoricon | ~3,100 | ✅ integrated; compile + cohab-link ×3 green; ×3 gate pending — **Wave 3 COMPLETE** |
| 4 — secondary | clipboard/prop/mdi/spy/winsta/imm/exticon/dllmain/+misc/+minor controls | ~12,000 | ☐ |
| 5 — DDE (deferred) | ddemisc/ddeclient/ddeserver/dde | ~5,400 | ☐ |

**Next action:** Wave 4 — secondary surface (clipboard/prop/mdi/spy/winsta/imm/exticon/dllmain
+ misc + minor controls, ~12K LOC). Not boot-critical; can be batched.

**Wave 3c done (2026-06-21):** `cursoricon.rs` (90 fns, 3.4K LOC) integrated; compile + cohab-link
×3 green. **libpng was a non-issue:** delay-load was never functional in vibeos (see
`delay-import-llvm-mingw-design.md` §1 — `add_delay_importlibs` silently degrades to a regular
import), so the ~23 `png_*` calls are plain `extern "C"` regular imports resolving to libpng.dll
on i386/amd64 and to the `arm64_user32_stubs.c` NULL-stubs on arm64 — identical to the old C.
`png_check_sig`→`png_sig_cmp(..)==0`, `PNG_LIBPNG_VER_STRING`→`b"1.6.39\0"`. `get_icon_size`/
`CursorIconToCursor`/`get_best_icon_file_offset` are the cdecl cross-TU exports (draw.rs/static_.rs/
exticon.c). Bug classes hit: `bmiColors[N]` indexing on a `[_;1]` flexible-array-member trips
Rust's compile-time bounds check → use `.as_ptr().add(N)`; `GetObject`→`GetObjectW` (i386 UNICODE
macro); HICON/HCURSOR → `as HANDLE` at NtUser* boundaries; opaque `gpsi.hIconSmWindows` via
`gpsi_perusr()`. With 3a+3b landed, **Wave 3 (the explorer/desktop-render path) is complete.**

~~Wave 3 — the explorer/desktop-render path (menu/draw/cursoricon/text/font/
accel/rect/hook/input/desktop/display, ~11.4K LOC). Run it in 3 sub-slices (3a leaf →
3b core → 3c cursoricon), gating each ×3, per the §2.8 playbook. Watch `MENU_builtin_class`
(cross-TU), `cursoricon.c`'s `get_icon_size`/PNG path, and the §2.7 link classes.
(Wave 2's amd64/arm64 cohab link is still owed to the control-plane ×3 gate.)

> **Rules reminder (AGENTS.md):** never revert/delete/hide a Rust translation to dodge a
> regression; a broken-but-integrated translation visible in smoke is the correct state.
> Paid translation outputs → `~/vibeos-rust-staging/` + `tools/rustconv/.responses/`, never
> `/tmp`. Don't push without an explicit ask (the ×3-green auto-push authorization is the
> control-plane's, not this lane's).
