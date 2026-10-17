# Version flip audit: 5.2 -> 10.0 (Windows 11 = 10.0 build >= 22000)

Read-only audit of the tree at branch `research`, 2026-10-17 (line numbers are as of then).
Nothing was built or booted. "live" means compiled into a shipping binary; "dead" means a
cohab-flipped `.c` that is commented out of its CMakeLists (the Rust twin is what runs);
"oracle" means built only as a difftest oracle (`c_oracle.cmake`, msvcrt `msvcrt_static`).

**Today the OS reports NT 5.2 build 3790 SP2**, not 6.x: `NtMajorVersion = 5`,
`NtMinorVersion = 2`, `NtBuildNumber = 3790` (`sdk/lib/rust/ex-rust/src/init.rs:168-176`),
registry `CurrentVersion = "5.2"`, `CSDVersion = 0x200`.

## Summary

| Class | Rows | Meaning |
|---|---|---|
| (a) BREAKS | 11 | wrong at 10.0 (or at the 6.2 an unmanifested caller gets) |
| (b) FINE | 38 | correct at 10.0 (>= checks, pass-through, display, Win9x-platform tests) |
| (c) XP FALLBACK | 27 | correct comparison, but it flips from the XP/5.x branch to the Vista+ branch: re-examine that the newer branch works here |
| (d) DEFINITION | 19 | where the version is produced or stored, all to be changed or decided |

Counts are table rows. Some rows cover several lines of the same check; the Win9x
`GetVersion() & 0x80000000` row stands for 24 call sites. Dead C files are listed
separately (about 60), and each one's live Rust twin is classified in the tables.

### (a) BREAKS list

1. **`sdk/lib/rust/ps-rust/src/psmgr.rs:748`** (live): `PsGetVersion` returns `NtBuildNumber & 0x3FFF`. Build 22000 is 0x55F0, which is larger than 0x3FFF, so it comes back as **5616**. Windows 10+ masks with 0xFFFF.
2. **`sdk/lib/rust/rtlk-rust/src/misc.rs:72`** (live): the kernel `RtlGetVersion` uses `dwBuildNumber = NtBuildNumber & 0x3FFF`. Same truncation. Kernel `RtlVerifyVersionInfo` build checks (`rtl-version`) inherit it.
3. **`sdk/lib/rust/ntoskrnl-rust/src/ARM3/procsup.rs:497`** (live): `Peb->OSBuildNumber = NtBuildNumber & 0x3FFF`. Every user-mode reader inherits the truncation: ntdll `RtlGetVersion`, `RtlGetNtVersionNumbers`, kernel32 `GetVersion`/`GetVersionEx`/`VerifyVersionInfo`. A build check like `IsWindows11` (build >= 22000) fails. (`procsup.rs:590` applies the same mask to the image `Win32VersionValue` override, but that matches Windows' field layout, so it is fine.)
4. **`dll/win32/uxtheme/themehooks.c:747,752`** (live C, no Rust): `ThemeHooksInstall` accepts only `== 5.1` (XP UAH signature) or `== 5.2` (2003 signature). Anything else hits `UNIMPLEMENTED; ret = FALSE`, so the user-API theming hook is never installed. It uses `GetVersionEx`, so it **breaks at the lied 6.2 too**. This is the most visible break.
5. **`vendor/wdf/kmdf/src/core/fxbugcheckcallback.cpp:1057-1072`** (live, KMDF): `FX_DRIVER_TRACKER_CACHE_AWARE::Register` runs this check only when neither `KeQueryMaximumProcessorCountEx` nor `KeQueryMaximumProcessorCount` resolves. Both are `@ stub -version=0xA00+` in `ntoskrnl/ntoskrnl.spec:2452-2453` and `DLL_EXPORT_VERSION=0x502`, so neither does and the check is reached. It accepts only `(5.x, minor>0)` or `6.0`; any other version gives `ASSERT(FALSE); STATUS_NOT_SUPPORTED`. The version comes from `RtlGetVersion` (`globals.cpp:655`), which never lies. **Not determined:** whether vibeos builds KMDF with live ASSERTs (debug break) or whether the failure is only "tracker disabled". Fix: export the Ke*ProcessorCount pair, or accept major >= 6.
6. **`dll/win32/shlwapi/ordinal.c:4307,4309,4321`** (live): `IsOS(OS_XPORGREATER)`, `IsOS(OS_HOME)` and `IsOS(OS_PERSONALTERMINALSERVER)` test `majorv >= 5 && minorv >= 1`. That is FALSE at 10.0 and TRUE at 6.2. No in-tree caller uses these three (callers use OS_ANYSERVER, OS_SERVERADMINUI and OS_WOW6432); third-party callers do.
7. **`dll/win32/shcore/main.c:2473,2485`** (live): the same `IsOS` table, for OS_HOME and OS_PERSONALTERMINALSERVER. OS_XPORGREATER already has an `__REACTOS__` `|| majorv >= 6` fix at 2468. Copy that fix to both rows.
8. **`sdk/lib/rust/cicero-rust/src/cicero/cicbase_cpp_h.rs:110`** (live): `cicGetOSInfo` sets `CIC_OSINFO_XPPLUS` only if `major >= 5 && minor > 0`, so a manifested (10.0) host loses XPPLUS. The consumers are `msutb-rust` (9 sites: the language-bar desk band is disabled) and `msctf-rust/src/msctf/utils_cpp_h.rs:375`. It works at the lied 6.2.
9. **`sdk/lib/rust/oleaut32-rust/src/oleaut.rs:515`** (live): `OaBuildVersion` switches on `GetVersion() & 0x8000ffff`, and the table has no `0x0000000A` entry. 10.0 takes the `_ =>` arm, which prints a FIXME and returns the same `MAKELONG(0xffff,50)`. Cosmetic only: add the `0x0000000A` arm.
10. **`sdk/lib/rust/ntdll-rust/src/ldr/ldrinit.rs:1698`** (live; the C at `dll/ntdll/ldr/ldrinit.c:1692` does the same): `LdrpInitializeProcessCompat` walks `KnownCompatGuids` from **Vista upward** and records the **lowest** `supportedOS` GUID the manifest declares as `dwRosProcessCompatVersion`. A normal manifest lists Vista through Win10 and gets **0x0600**. This breaks the plan: the GetVersionEx manifest rule needs "declares the Win10 GUID", and this field cannot answer that. It also feeds `RosGetProcessEffectiveVersion` (shell32) and `LdrpApisetVersion` (`ntdll-rust/src/rtl/libsupp.rs:1184`), so a typical modern manifest selects Vista api-set behaviour. Record the highest GUID, or a separate "declares Win10" bit.
11. **`dll/win32/setupapi/devinst.c:474-475,613-614`**, version taken at `setupcab.c:656` (live): INF `[Manufacturer]` decoration scoring uses `GetVersionExW`. Under the planned manifest rule, every unmanifested host (umpnpmgr/services, newdev, rundll32) sees **6.2**, so a driver INF decorated only `NTamd64.10.0` is rejected. Windows' setupapi uses the real version. Use `RtlGetVersion` here. This one is conditional on how the manifest rule is implemented.

### Definition sites to change for the flip

| # | Site | Now | Change |
|---|---|---|---|
| D1 | `sdk/lib/rust/ex-rust/src/init.rs:168,170` `NtMajorVersion`/`NtMinorVersion` (live) | 5 / 2 | 10 / 0 |
| D2 | `ex-rust/src/init.rs:173,176` `NtBuildNumber` (live) | 3790 (\|0xC0000000 in debug) | >= 22000 (e.g. 22631); keep the 0xC/0xF top-nibble convention |
| D3 | `ex-rust/src/init.rs:206` `NtBuildLab: [u8; 22] = "3790.reactos.0.x86fre"` (live) | fixed 22-byte array; also says x86 on every arch | resize, new build, per-arch tag |
| D4 | `ex-rust/src/init.rs:1153-1162` `CmVersionString` printf of the literal `b"5.2"` (live) | `"5.2"` | **`"6.3"`**, which is what Windows 10/11 keep in `CurrentVersion` (apps read it), then add the values in D6 |
| D5 | `ex-rust/src/init.rs:1221-1222` `SharedUserData->NtMajor/MinorVersion = Nt*Version` (live) | copies D1 | none; follows D1 |
| D6 | `sdk/lib/rust/config-rust/src/cmsysini.rs:2667-2700` writes `CurrentVersion`, `CurrentBuildNumber` (`NtBuildNumber & 0xFFFF`, OK), `BuildLab`, `CurrentType`, `CSDVersion` (live) | no `CurrentMajorVersionNumber`/`CurrentMinorVersionNumber` (DWORD 10/0), `CurrentBuild`, `UBR`, `DisplayVersion`, `EditionID`, `InstallationType` | add them; Win10+ detectors read `CurrentMajorVersionNumber` first |
| D7 | `boot/bootdata/hivesys.inf:1472` `Control\Windows\CSDVersion = 0x200` | SP2 | 0 (Win10/11 have no service pack; feeds `CmNtCSDVersion` -> PEB `OSCSDVersion`/`wServicePackMajor`) |
| D8 | `boot/bootdata/hivesft.inf:594` `ProductName = "ReactOS"` | ReactOS | **decision**: Win11 reports "Windows 10 Pro" here; some apps parse it |
| D9 | `sdk/lib/rust/ntoskrnl-rust/src/ARM3/procsup.rs:495-497` PEB init (live) | `OSBuildNumber = NtBuildNumber & 0x3FFF` | `& 0xFFFF` (BREAKS #3) |
| D10 | `sdk/lib/rust/ps-rust/src/psmgr.rs:742-748` `PsGetVersion` (live) | `& 0x3FFF` | `& 0xFFFF` (BREAKS #1) |
| D11 | `sdk/lib/rust/rtlk-rust/src/misc.rs:67-72` kernel `RtlGetVersion` (live) | `& 0x3FFF` | `& 0xFFFF` (BREAKS #2) |
| D12 | `sdk/lib/rust/ntdll-rust/src/rtl/version.rs:188-264` user `RtlGetVersion` (live) | reads the PEB, and `SetRosSpecificInfo` overrides product type from `HKLM\...\ReactOS\Settings\Version\ReportAsWorkstation` | stays truthful (the plan: never lies); no change beyond D9 |
| D13 | `sdk/lib/rust/kernel32-rust/src/client/version.rs:11` `GetVersion`, `:22` `GetVersionExW`, `:52` `GetVersionExA`, `:118` `VerifyVersionInfoW`, `:151` `VerifyVersionInfoA` (live) | plain pass-through of `RtlGetVersion`/PEB | **add the manifest rule to all five**. Windows also lies in `GetVersion` and `VerifyVersionInfo`, not only `GetVersionEx`. `VerifyVersionInfoW` -> `RtlVerifyVersionInfo` -> the truthful `RtlGetVersion` today. Depends on fixing BREAKS #10 |
| D14 | `ntdll-rust/src/ldr/ldrinit.rs:1638-1740` `LdrpInitializeProcessCompat` (live) | records the lowest declared GUID | must yield "declares Win10 GUID" (BREAKS #10) |
| D15 | `sdk/lib/rust/rtl-strings/src/core_rs/process.rs:258` `RtlRosGetAppcompatVersion` = shim version, else `(major<<8)\|minor` | becomes 0x0A00 | fine; consumer `ps-rust/src/query.rs:1528` listed in (c) |
| D16 | `sdk/include/psdk/ntverp.h:19-28` `VER_PRODUCTBUILD 3790`, `VER_PRODUCTMAJORVERSION 5`, `MINOR 2`, `VER_PRODUCTVERSION_W 0x0502` | 5.2.3790 | feeds the `common.ver` FILEVERSION of **177** `.rc` files, plus the C `ex/init.c` (dead). **Decision**: bump it, or leave it as the SDK-header value |
| D17 | `sdk/include/reactos/version.cmake:23` `REACTOS_DLL_VERSION_MAJOR = KERNEL_VERSION_MAJOR + 42` -> **kernel32.dll/ntdll FILEVERSION 42.x** (543 `.rc` use `reactos/version.rc`) | 42.4.x | **Decision**: Chrome's `OSInfo::Kernel32BaseVersion` and Python's `platform.win32_ver` read kernel32's file version, and 42.x tells them a very new Windows. Not verified what those programs do with it |
| D18 | `sdk/lib/rust/ex-rust/src/init.rs:820-826` `ExpIsLoaderValid` (loader extension >= 5.2) and `sdk/lib/rust/freeldr-rust/src/setupldr.rs:766,793` / `winldr.rs:213` (`0x0502` loader block) | loader ABI version | **do NOT change** with the flip. This is the loader-block layout handshake, not the reported version |
| D19 | `sdk/lib/rust/aclayers-rust/src/aclayers/versionlie.rs:9-297` shim lie tables (Win95..Win10 1809), `APIHook_RtlGetVersion` `:468` (live) | no Win11 entry | optional: add a Win11 layer |

Display-only consumers that follow D1/D2 with no change: `win32k-rust/src/desktop.rs:749-751,785-787`
(watermark, `& 0xFFFF`), `kd64-rust/src/kdinit.rs:242-243`, `cmd-rust/src/cmd/ver.rs:143`. Both
`winver` (`winver-rust` -> `ShellAboutW`) and `shell32-rust/src/wine/shell32_main.rs:237-238` print
`KERNEL_VERSION_STR` ("0.4.16"), not the NT version.

### Could not determine

- Whether KMDF is built with ASSERTs live, so whether BREAKS #5 is a debug break or only a disabled tracker.
- Whether the HAL accepts `DEVICE_DESCRIPTION_VERSION3` (the KMDF DMA path at `fxdmaenabler.cpp:313` takes it at >= 6.2), and whether passive-level interrupts (`IoConnectInterruptEx` passive) work. Both turn on at 10.0, see (c).
- Whether `DwmIsCompositionEnabled == TRUE` (`dwmapi_main.c:61`, via `RtlGetVersion`, so 10.0 regardless of manifest) leads apps into dwmapi calls that are unimplemented. I did not audit dwmapi's other exports.
- Whether win32k `SPI_GETNONCLIENTMETRICS` (`sdk/lib/rust/win32k-rust/src/sysparams.rs:1893,2412`) rejects the Vista-size `NONCLIENTMETRICSW` (with `iPaddedBorderWidth`). Apps that see >= 6 pass that size. This is not a version check, but it is the classic flip-day regression. Unverified.
- Which `.rapps` DB entries carry an NT-version upper bound (`rapps-rust/src/rapps/appinfo_cpp_h.rs:333`). Entries capped below 0x0A00 will hide.
- `BaseStaticServerData->WindowsMajorVersion/MinorVersion/BuildNumber` (`smss-csrss-rust/src/basesrv/init.rs:236-238`) is never assigned in the Rust; only CSDNumber/RCNumber are set at `:928`. I did not determine whether anything reads it.
- I did not audit runtime checks inside third-party vendored trees beyond `vendor/wdf` (`vendor/fex` hits are `.def`/header declarations only).

---

## (a) BREAKS: full rows

| Site | Live? | Reason |
|---|---|---|
| `sdk/lib/rust/ps-rust/src/psmgr.rs:748` | live | `& 0x3FFF` truncates builds >= 16384 |
| `sdk/lib/rust/rtlk-rust/src/misc.rs:72` | live | same, kernel `RtlGetVersion` |
| `sdk/lib/rust/ntoskrnl-rust/src/ARM3/procsup.rs:497` | live | same, PEB `OSBuildNumber` (every user-mode reader) |
| `dll/win32/uxtheme/themehooks.c:747,752` | live C | `==5.1` / `==5.2` else UNIMPLEMENTED; breaks at 6.2 as well |
| `vendor/wdf/kmdf/src/core/fxbugcheckcallback.cpp:1057` | live | only XP / Vista 6.0 accepted when the Ke*ProcessorCount exports are absent (they are) |
| `dll/win32/shlwapi/ordinal.c:4307,4309,4321` | live C | `minorv >= 1` excludes 10.0 |
| `dll/win32/shcore/main.c:2473,2485` | live C | same; OS_XPORGREATER already patched at :2468 |
| `sdk/lib/rust/cicero-rust/src/cicero/cicbase_cpp_h.rs:110` | live | `minor > 0` excludes 10.0, so msutb desk band and msctf XPPLUS paths turn off |
| `sdk/lib/rust/oleaut32-rust/src/oleaut.rs:515` | live | lookup table misses 0x000A (FIXME spam only) |
| `sdk/lib/rust/ntdll-rust/src/ldr/ldrinit.rs:1698` | live | records the lowest supportedOS GUID, so it cannot drive the Win10 manifest rule |
| `dll/win32/setupapi/devinst.c:474,613` (+`setupcab.c:656`) | live C | `GetVersionExW` would see 6.2 under the manifest rule, so `.10.0` INF decorations stop matching |

## (b) FINE

| Site | Live? | Reason |
|---|---|---|
| `sdk/lib/rust/rtl-version/src/lib.rs:117-260` `RtlVerifyVersionInfo` | live | hierarchical major->minor->SP, as Windows does: `>= 6.2` passes at 10.0. Build is compared independently (needs D9/D11) |
| `sdk/lib/rust/rtl-version/src/lib.rs:265` `VerSetConditionMask` | live | no version data |
| `sdk/lib/rust/ntdll-rust/src/rtl/version.rs:157-182` `RtlGetNtVersionNumbers` | live | `<5` clamps only; `0xF0000000 \| OSBuildNumber` |
| `sdk/lib/rust/ntdll-rust/src/rtl/version.rs:297,304` `RtlGetProductInfo` | live | `<6` -> UNDEFINED; 6.0 -> BUSINESS; else PROFESSIONAL (10.0 OK) |
| `sdk/lib/rust/rtl-sync/src/critical.rs:332` | live | `>= _WIN32_WINNT_WIN7` enables extra CS flags |
| `sdk/lib/rust/config-rust/src/cmsysini.rs:2679` | live | `NtBuildNumber & 0xFFFF` |
| `sdk/lib/rust/kd64-rust/src/kdinit.rs:242-243` | live | `& 0xFFFF`, `>>28` |
| `sdk/lib/rust/ps-rust/src/psmgr.rs:757` `PsIsCheckedBuild` | live | top nibble only |
| `sdk/lib/rust/win32k-rust/src/desktop.rs:656,749,785` | live | watermark display |
| `sdk/lib/rust/pcix-rust/src/utils.rs:194-195` | live | suite-mask check only |
| `sdk/lib/rust/classpnp-rust/src/xferpkt.rs:223,261` | live | product type only |
| `sdk/lib/rust/wdfldr-rust/src/wdfldr/wdfldr.rs:259-264` | live | logged only |
| `sdk/lib/rust/logon-rust/src/logon/logon.rs:21` | live | product type only |
| `sdk/lib/rust/msgina-rust/src/shutdown.rs:204` | live | result unused |
| `sdk/lib/rust/cmd-rust/src/cmd/ver.rs:39-145` | live | registry ProductName; the 4.x branch is Win9x platform only |
| `sdk/lib/rust/dxdiag-rust/src/dxdiag/system.rs:386` | live | display |
| `sdk/lib/rust/eventlog-rust/src/eventlog/eventlog.rs:234-267` | live | logged |
| `sdk/lib/rust/srvsvc-rust/src/srvsvc/rpcserver.rs:254,296,323` | live | reported to NetServerGetInfo (10.0 correct) |
| `sdk/lib/rust/wkssvc-rust/src/wkssvc/{wkssvc.rs:71,rpcserver.rs:188-236}` | live | reported to NetWkstaGetInfo |
| `sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/data.rs:427-447` | live | `_winver/_winmajor/_osver`, crtdll base version (from the lied GetVersionEx) |
| `sdk/lib/crt/misc/crt_init.c:27-32` | live | same, static CRT |
| `sdk/lib/rust/localspl-rust/src/localspl/{printerdata.rs:237,253,printers.rs:835}`, `winspool-rust/src/winspool/printerdata.rs:953` | live | pass-through to the spooler "OSVersion" data |
| `sdk/lib/rust/ole32-rust/src/ring/stg_prop.rs:630` | live | stored in the property-set header |
| `sdk/lib/rust/user32-rust/src/controls/edit.rs:365-367` | live | `min(process version, OS version)` |
| `sdk/lib/rust/user32-rust/src/controls/{combo.rs:205,listbox.rs:285}`, `comctl32-rust/src/comctl32/{combo.rs:2143,listbox.rs:2265,2405,3661}` | live | `LOWORD(GetVersion()) == 0x0a03` is the Win 3.10 hack; 10.0 gives 0x000A |
| Win9x tests `GetVersion() & 0x80000000` (24 sites): `advapi32-rust/src/wine/security.rs:2618,3147,3168`; `localspl-rust/src/localspl/printerdrivers.rs:1372`; `regedit-rust/src/regedit/treeview.rs:533`; `shell32-rust/src/shell32/shlexec.rs:1620`, `shellutils.rs:340`, `wine/{pidl.rs:351,shellole.rs:394,shellord.rs:984,shellpath.rs:432,shellstring.rs:92,shlmenu.rs:411}`; `version-rust/src/lib.rs:1625`; `winegdi-rust/src/winegdi/enhmetafile.rs:1304,1368,1400,2018`; `dll/directx/wine/ddraw/clipper.c:170`; `dll/win32/msi/install.c:709`; `dll/win32/riched20/txthost.c:1523,1631`; `dll/win32/shlwapi/ordinal.c:3138`; `dll/win32/shlwapi/thread.c:73` | live | bit 31 stays 0 at build 22000 (`22000<<16 = 0x55F00000`) |
| `dll/directx/wine/wined3d/directx.c:762-814,859-861` | live | has `case 10:` (driver_os_version 31); feature-level caps only for 6.0/<6 |
| `dll/directx/wine/dxdiagn/provider.c:668-678` | live | reported |
| `dll/win32/dbghelp/minidump.c:645-665` | live | recorded in minidump |
| `dll/win32/msi/package.c:751-770` | live | `VersionNT` capped at 603/9600 (Wine = Windows behaviour) |
| `dll/win32/msi/automation.c:1963` | live | display |
| `dll/win32/wbemprox/builtin.c:4148-4226,4282` | live | knows 10.0 and Win11 by build >= 22000 (needs D9). Another lane has this file dirty |
| `dll/win32/urlmon/session.c:553-568` | live | UA string "Windows NT 10.0" |
| `dll/win32/comdlg32/filedlg.c:1695-1699` | live | `> 4` checks |
| `dll/win32/msacm32/msacm32_main.c:81`, `base/services/tftpd/tftpd.cpp:350`, `base/applications/network/telnet/src/tnmisc.cpp:106` | live | platform id only |
| `base/applications/msconfig_new/utils.c:241-242` | live | `(5 && >=2) \|\| >5` |
| `sdk/lib/drivers/aux_klib/aux_klib.c:35-36`, `sdk/lib/drivers/rtlver/rtl.c:37-40`, `vendor/wdf/shared/irphandlers/pnp/fxpkgpnp.cpp:3689`, `vendor/wdf/kmdf/src/librarycommon/fxlibrarycommon.cpp:289-294` | live | `>=` checks / logging |
| `sdk/lib/rust/shell32-rust/src/shell32/brfolder.rs:1723`, `CDefaultContextMenu.rs:1735-1736` | live | `>= XP/VISTA/WIN7` via `RosGetProcessEffectiveVersion`. Note BREAKS #10: a manifested app reads as 0x600 |

## (c) XP FALLBACK becomes dead: re-examine the Vista+ branch

All of these are correct `>= 6` / `< 6` tests. Today (5.2) they take the XP branch. At 10.0, and
also at the lied 6.2 since every in-tree host is unmanifested, they take the newer one.
**None of the in-tree `.manifest` files declares any Win10 `supportedOS`** (only two rostests
manifests contain the GUID), so after the flip every in-tree component reads 6.2 through
`GetVersion*`.

| Site | Live? | What flips |
|---|---|---|
| `sdk/lib/rust/shell32-rust/src/control_panel_folder.rs:1184,1201` | live | GetDefaultSearchGUID -> E_NOTIMPL; GetDefaultColumnState becomes implemented |
| `sdk/lib/rust/shell32-rust/src/drives_folder.rs:1776,2417` | live | "c:" parsing accepted; search GUID -> E_NOTIMPL |
| `sdk/lib/rust/shell32-rust/src/recycle_bin.rs:2097` | live | search GUID -> E_NOTIMPL |
| `sdk/lib/rust/shell32-rust/src/shell32/CDefaultContextMenu.rs:2481` | live | unknown verb error E_INVALIDARG -> ERROR_NO_ASSOCIATION |
| `CDefaultContextMenu.rs:2524` | live | ICommDlgBrowser filtering |
| `CDefaultContextMenu.rs:2797` | live | multi-folder open: `wFlags = 0` (FIXME SBSP_NEWBROWSER) instead of one window per folder |
| `sdk/lib/rust/shell32-rust/src/shell32/shlexec.rs:286` | live | ShellExecute hooks need REST_SH32_ENABLESHELLEXECUTEHOOKS |
| `sdk/lib/rust/shell32-rust/src/shell32/shlfileop.rs:2364,2393,2446` | live | Vista error-code remap for missing files |
| `sdk/lib/rust/shell32-rust/src/shell_link.rs:2710` | live | `> 6`: NT6 link flags kept (true at 10.0, still false at 6.2) |
| `sdk/lib/rust/comctl32-rust/src/comctl32/imagelist.rs:1182,3805,3810` | live | ILC_SYSTEM handling |
| `sdk/lib/rust/rapps-rust/src/rapps/misc_cpp_h.rs:591` | live | NT6 user program-files path |
| `sdk/lib/rust/rapps-rust/src/rapps/appinfo_cpp_h.rs:333` (via `misc_cpp_h.rs:410-415` `RtlGetVersion`) | live | DB `NTVER` ranges: entries capped below 0x0A00 hide |
| `sdk/lib/rust/kernel32-rust/src/file/filemap.rs:101` | live | `PAGE_EXECUTE_WRITECOPY` accepted in CreateFileMapping |
| `sdk/lib/rust/ps-rust/src/query.rs:1528` | live | amd64 `ProcessUserModeIOPL` -> STATUS_NOT_IMPLEMENTED for every user process (appcompat > WS03) |
| `sdk/lib/rust/cnt-rust/src/cabi/generated.rs:438` / `core_rs/mem.rs:48` `_memicmp` (C `sdk/lib/crt/mem/memicmp.c:11` is REMOVE_ITEM'd) | live | the Rust deliberately did NOT model the C's `OSMajorVersion >= 6` NULL/`_NLSCMPERROR` guard ("dead under NT 5.x"). At 10.0 the C would validate NULL and the Rust will not. `cnt-difftest/build.rs:115-120` pins `OSMajorVersion=5`, so the difftest will not catch it |
| `sdk/lib/rust/ntdll-rust/src/rtl/libsupp.rs:1184-1205` `LdrpApisetVersion` | live | api-set table choice follows the shim version (see BREAKS #10) |
| `dll/win32/browseui/shellbrowser.cpp:3990` | live C | Backspace: NavigateToParent -> GoBack |
| `dll/win32/shlwapi/querysrc.cpp:108-112` | live C | Vista-only assoc elements enabled |
| `dll/win32/shlwapi/utils.cpp:40` (`GetVersionMajorMinor`, used by SHInvokeCommand*) | live C | Vista+ invoke behaviour |
| `dll/win32/dwmapi/dwmapi_main.c:61` | live C | `DwmIsCompositionEnabled` -> **TRUE** (uses `RtlGetVersion`, so 10.0 even unmanifested) |
| `vendor/wdf/shared/object/globals.cpp:448-459` | live | `>= 6.2` -> `PassiveLevelInterruptSupport = TRUE` |
| `vendor/wdf/kmdf/src/dma/base/fxdmaenabler.cpp:313` | live | `>= 6.2` -> DMA `DEVICE_DESCRIPTION_VERSION3` |
| `dll/directx/wine/wined3d/directx.c:859-861` | live | feature level no longer capped at 14 |
| `base/applications/msconfig_new/utils.c:284` | live C | `< 6` "is pre-Vista" helper -> FALSE |
| `sdk/lib/rust/ntdll-rust/src/rtl/version.rs:296-317` `RtlGetProductInfo` | live | callers asking with major >= 6 get a SKU (today `GetProductInfo` is passed 5.2 by most apps -> UNDEFINED) |
| `dll/win32/setupapi/devinst.c:282-321` (scoring) | live C | at 10.0 (if RtlGetVersion is used) every `.6.x`/`.10.0` decoration matches; XP-only INF sections lose priority |
| `sdk/lib/rust/shell32-rust/src/shellutils.rs:399-405` `RosGetProcessEffectiveVersion` | live | becomes 0x0A00, or the shim's version, which is the lowest manifest GUID |

## Dead C (cohab-flipped): Rust twin classified above

Commented out of their CMakeLists (or REMOVE_ITEM'd), so they are inert: ntoskrnl `ex/init.c`,
`ps/psmgr.c`, `mm/ARM3/procsup.c`, `rtl/misc.c`, `rtl/libsupp.c`, `kd64/kdinit.c`,
`config/cmsysini.c`; `dll/ntdll/{rtl/version.c,rtl/libsupp.c,ldr/ldrinit.c}`;
`sdk/lib/rtl/{version.c,critical.c,process.c}`; `kernel32/client/{version.c,versionansi.c,proc.c,file/filemap.c}`;
shell32 `{CDefaultContextMenu,CShellDispatch,CShellLink,shell32,shlexec,shlfileop}.cpp`,
`folders/{CControlPanelFolder,CDrivesFolder,CRecycleBin}.cpp`, `wine/shellord.c`;
comctl32 `{commctrl,imagelist,combo,listbox}.c` (built only as the `c_oracle.cmake` oracle);
`user32/controls/{combo,edit,listbox}.c`; `win32ss/user/rtl/image.c`; `gdi32/wine/enhmetafile.c`;
`winspool/printerdata.c`; localspl `{printerdata,printerdrivers,printers}.c`; `advapi32/wine/security.c`;
`oleaut32/oleaut.c`; `ole32/stg_prop.c`; `mswsock/nsplookup.c`; `version/version.c`; `msgina/shutdown.c`;
`cabinet/cabinet_main.c`; `appcompat/shims/layer/{versionlie.c,forcedxsetupsuccess.c}`;
`base/ctf/cicero/{cicbase,cicuif}.cpp`; cmd `ver.c`; calc, dxdiag, net, rapps, regedit,
`screensavers/logon`, explorer `syspager.cpp`, eventlog, srvsvc, wkssvc, `cpl/desk/theme.c`,
`drivers/bus/pcix/utils.c`, `sdk/lib/drivers/wdfldr/wdfldr.c`, `drivers/storage/class/classpnp/xferpkt.c` (cargo link).
`dll/win32/msvcrt/data.c` is oracle-only: "No DLL links it", per `msvcrt/CMakeLists.txt:81`.

Not built at all: `dll/win32/kernelbase/wine/*` (`#add_subdirectory(wine)`, including its
`version.c` GetVersionEx/manifest code), `dll/directx/d3d9/adapter.c`
(`#add_subdirectory(d3d9)`), and all of `modules/rosapps` (`ENABLE_ROSAPPS` OFF; that covers
gettype, systeminfo, rosperf, vfd).

The `.c` twins of these Rust sites carry the same logic: uxtheme has no Rust twin; cicero's C
`cicbase.cpp:114-116` has the same XPPLUS bug (dead); `kernel32/client/proc.c:131`
`BasepIsImageVersionOk` only logs (Rust `kernel32-rust/src/proc_helpers.rs:99` same).

## Wine-imported DLLs that consult the version

| DLL | Site | Behaviour chosen |
|---|---|---|
| wined3d | `directx.c:753-861` | driver_os_version / driver model / max feature level (has a 10.0 case) |
| dxdiagn | `provider.c:668` | reports |
| ddraw | `clipper.c:170` | Win9x platform test |
| comctl32 (Rust) | imagelist ILC_SYSTEM `>=6`; combo/listbox Win3.1 hack | (c) / (b) |
| comdlg32 | `filedlg.c:1695` | win98plus/win2000plus |
| shlwapi | `ordinal.c` IsOS (BREAKS #6), `:3138`, `thread.c:73`, `querysrc.cpp:108`, `utils.cpp:40` | |
| shcore | `main.c` IsOS (BREAKS #7) | |
| msi | `package.c:751` VersionNT (capped at 603), `install.c:709` | |
| dwmapi | `dwmapi_main.c:61` | composition-enabled answer |
| msacm32 | `msacm32_main.c:81` | acmGetVersion by platform |
| riched20 | `txthost.c:1523,1631` | Win9x platform |
| urlmon | `session.c:553` | UA string |
| wbemprox | `builtin.c:4177-4199` | Win32_OperatingSystem Caption by version + build |
| oleaut32 (Rust) | `oleaut.rs:515` | OaBuildVersion table (BREAKS #9) |
| advapi32 (Rust, wine/security) | `security.rs:2618,3147,3168` | Win9x platform |
| gdi32 (Rust winegdi enhmetafile) | 4 sites | Win9x platform |
| dbghelp | `minidump.c:645` | recorded |
| setupapi (wine-derived) | `devinst.c` (BREAKS #11) | INF decoration match |
| version (Rust) | `version-rust/src/lib.rs:1625` | Win9x platform |

**dwrite, dxgi, d3d11, d2d1, d3d8/d3d9 (wine): no runtime version checks found.**
`DllGetVersion` constants (cabinet, comctl32, shell32, msi, riched32, msftedit, shdocvw,
browseui, shlwapi_main) and `HTTP_VERSION_INFO` (wininet), `DXAPI` 1.0 report their own
component version, not the OS, and are excluded.

## Tests that hard-code the version

- `tests/host/include/nt_host.h:1489`: host `KUSER_SHARED_DATA = { 5, 2 }`.
- `sdk/lib/rust/cnt-difftest/build.rs:115-120`: pins `OSMajorVersion = 5` for memicmp (see (c)).
- `sdk/lib/rust/rtl-strings/src/core_rs/process.rs:317-332`: unit tests for appcompat_version (already cover 10.0).
- rostests (excluded from classification, noted): 491 uses of `GetNTVersion()`/`GetWinVersion()` choose expectations by version, so the flip changes which expectations apply. Examples: `apitests/crt/_mbsncat.c:68,89` (`<= 0x502`), `apitests/kernel32/MultiByteToWideChar.c:65-116` (`{0x000,0x502}` ranges), `apitests/kernel32/ConsoleCP.c:1519` (`== 6 && >= 2`), `apitests/appshim/versionlie.c:278-279`, `apitests/apphelp/apphelp_apitest.h:30`, `apitests/kernel32/FLS.c:23`.
- `tools/win10_surface.py:6` and `tools/payloads/regstress/regstress.c:134`: comments stating "we report NT 5.2" (docs to update).

## Method

`git grep` for GetVersion/GetVersionEx(A/W)/VerifyVersionInfo/VerSetConditionMask/RtlGetVersion/
RtlVerifyVersionInfo/IsWindows*OrGreater/versionhelpers.h/OS{Major,Minor}Version/OSBuildNumber/
NtMajorVersion/NtBuildNumber/PsGetVersion/RtlGetNtVersionNumbers/dw{Major,Minor}Version/
dwBuildNumber, plus `RosGetProcessEffectiveVersion`/`RtlRosGetAppcompatVersion`/`IsOS(OS_*)`/
`CIC_OSINFO_*`. Excluded: `modules/rostests` (noted above), `types_*.rs`/bindgen islands and
`offset_check_*.rs`, Rust `//` comment-blocks of the original C, and compile-time `#if` gates.
Liveness was checked against each module's CMakeLists, ignoring `c_oracle.cmake`; Rust crates
were treated as live. The Rust crates were not individually verified as linked.
