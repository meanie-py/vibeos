//! Shared engine for cargo-driven ReactOS PE-module links (i386).
//!
//! Lifts the machinery duplicated across `ntoskrnl-cargo` and
//! `hal-cargo` build.rs files into one builder. A per-module build.rs
//! becomes a manifest:
//!
//! ```ignore
//! reactos_link::Linker::new("i686-kernel.json")
//!     .rust_staticlib("hal-rust")
//!     .asm(&["hal/halx86/generic/systimer.S", ..],
//!          reactos_link::AsmDefineSet::Hal)
//!     .resource("data/hal-i386.rc.res")
//!     .spec2def("hal/hal.spec", "hal.dll")
//!     .static_libs(&["sdk/lib/crt/liblibcntpr.a", ..])
//!     .link(reactos_link::ModuleKind::KernelDll {
//!         entry: "_HalInitSystem@8".into(), image_base: 0x0001_0000 });
//! ```
//!
//! Runs inside the consumer's build.rs, so env vars
//! (`CARGO_MANIFEST_DIR`, `OUT_DIR`, `CARGO_PKG_NAME`, `CARGO`, `HOST`,
//! `CARGO_CFG_TARGET_ARCH`) refer to the consumer crate.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::{Arc, Mutex};

/// One link argument for THIS link: `rustc-link-arg`, or `rustc-link-arg-bin=<bin>=`
/// when the Linker drives one bin of a family package (see `Linker::for_bin`).
macro_rules! link_arg {
    ($s:expr, $($fmt:tt)*) => { $s.emit_link_arg(&format!($($fmt)*)) };
}

/// Work a family build script would otherwise repeat once per bin — the body crate's
/// nested cargo build, the host spec2def compile, the parsed compile_commands.json —
/// done once per build-script process. Keyed by everything the result depends on.
fn memo<V: Clone>(cache: &Mutex<Vec<(String, V)>>, key: String, make: impl FnOnce() -> V) -> V {
    if let Some((_, v)) = cache.lock().unwrap().iter().find(|(k, _)| *k == key) {
        return v.clone();
    }
    let v = make();
    cache.lock().unwrap().push((key, v.clone()));
    v
}
static RUST_LIBS: Mutex<Vec<(String, PathBuf)>> = Mutex::new(Vec::new());
static HOST_TOOLS: Mutex<Vec<(String, PathBuf)>> = Mutex::new(Vec::new());
type CcdbEntries = Arc<Vec<Vec<(String, String)>>>;
static CCDBS: Mutex<Vec<(String, CcdbEntries)>> = Mutex::new(Vec::new());

fn ccdb_entries(db_path: &Path) -> CcdbEntries {
    memo(&CCDBS, db_path.display().to_string(), || {
        let db_text = std::fs::read_to_string(db_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", db_path.display()));
        Arc::new(parse_ccdb(&db_text))
    })
}

pub mod native_deps;
pub mod cargo_deps;
pub mod compiler_env;
mod storage;
mod body_storage;
mod rc;
use rc::RcSpec;
mod c_rule;
use c_rule::CSpec;
mod artifact_store;

// ---- cmake-off `--trace` (ninja-`-v`-style command list) ------------------
// When `VIBEOS_LINK_TRACE` names a DIRECTORY, every command the engine spawns is
// appended (one shell-quoted line, prefixed `cd <dir> && ` when a working dir is
// set) to `<dir>/<pkg>.trace` — a flat, inspectable record of the real cmake-off
// build, the cargo-side analogue of `ninja -v`. Per-package files avoid garbled
// lines when cells build in parallel; the orchestrator's `--trace` clears the dir
// and concatenates them in cell order. No-op (zero overhead) when the var is
// unset, so a normal build is never affected.

/// `VIBEOS_FORCE_CMAKE_OFF`: make the engine take the cmake-OFF paths (self-host
/// geninc, committed `.res`, committed-flags C compile) even when a configured
/// `build-<arch>/` is present — i.e. treat compile_commands.json / the build-dir
/// ks*.inc as absent. Lets the cmake-off path be exercised + byte-verified against
/// the cmake artifact without physically moving build-dir files aside (what the
/// `--verify` gate and manual testing otherwise need). Off → normal behaviour.
/// The pinned nightly toolchain name, single-sourced from `.rust-nightly` at the
/// repo root (the stable half is pinned by `rust-toolchain.toml` beside it).
///
/// HARD FAIL if it's missing: a silent fallback to a floating `nightly` is exactly
/// the failure this pin exists to prevent — upstream has already removed the
/// `VaListImpl`/`as_va_list()` c_variadic API this tree is written against, so an
/// unpinned toolchain is one `rustup update` away from a confusing build break.
fn pinned_nightly(repo_root: &Path) -> String {
    let pin = repo_root.join(".rust-nightly");
    let name = std::fs::read_to_string(&pin).unwrap_or_else(|e| {
        panic!(
            "reactos-link: cannot read the pinned nightly at {}: {e}\n\
             build-std needs a PINNED nightly; refusing to fall back to a floating one.",
            pin.display()
        )
    });
    let name = name.trim().to_string();
    assert!(!name.is_empty(), "reactos-link: {} is empty", pin.display());
    println!("cargo:rerun-if-changed={}", pin.display());
    name
}

fn cmake_off_forced() -> bool {
    env::var_os("VIBEOS_FORCE_CMAKE_OFF").is_some()
}

/// Append one command line to this package's trace file (if tracing is on).
fn trace_command(cmd: &Command) {
    let dir = match env::var_os("VIBEOS_LINK_TRACE") {
        Some(d) => PathBuf::from(d),
        None => return,
    };
    // The trace dir is per-arch (the orchestrator points it at out_dir/.trace),
    // so the package name alone is unique within one run.
    let pkg = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown".into());
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join(format!("{pkg}.trace"));
    let mut line = String::new();
    if let Some(d) = cmd.get_current_dir() {
        line.push_str("cd ");
        line.push_str(&shell_quote(d.as_os_str()));
        line.push_str(" && ");
    }
    line.push_str(&shell_quote(cmd.get_program()));
    for a in cmd.get_args() {
        line.push(' ');
        line.push_str(&shell_quote(a));
    }
    line.push('\n');
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&file) {
        use std::io::Write;
        let _ = f.write_all(line.as_bytes());
    }
}

/// Trace a cc-rs compile by reconstructing its per-source command line (the
/// configured compiler + flags + `-c <source>`). cc-rs runs the compile itself;
/// this only records what it will run, so the trace covers the cc-rs compiles
/// (asm, c-shims, the committed C floor) alongside the directly-spawned tools.
fn trace_cc(build: &cc::Build, source: &Path) {
    if env::var_os("VIBEOS_LINK_TRACE").is_none() {
        return;
    }
    let mut c = build.get_compiler().to_command();
    c.arg("-c").arg(source);
    trace_command(&c);
}

fn shell_quote(s: &std::ffi::OsStr) -> String {
    let s = s.to_string_lossy();
    let needs = s.is_empty()
        || s.chars()
            .any(|c| c.is_whitespace() || "\"'\\$`*?[]{}()<>|&;#~!".contains(c));
    if needs {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.into_owned()
    }
}

/// `Command::status()` that records the command first when `--trace` is on.
trait TracedStatus {
    fn traced_status(&mut self) -> std::io::Result<ExitStatus>;
    fn native_status(&mut self, source: &Path) -> std::io::Result<ExitStatus>;
}
impl TracedStatus for Command {
    fn traced_status(&mut self) -> std::io::Result<ExitStatus> {
        trace_command(self);
        native_deps::scheduled_status(self)
    }
    fn native_status(&mut self, source: &Path) -> std::io::Result<ExitStatus> {
        native_deps::run(self, source, |cmd| cmd.traced_status())
    }
}

/// CRT-startup personality for `.msvcrtex_crt()` — selects which
/// crt-startup-rust feature (+ overridable sibling) the consumer links,
/// mirroring how the C archive lazily pulled crtdll.o / crtexe.o /
/// wcrtexe.o (+ dllmain.o / crt0_c.o / crt0_w.o) per the module's entry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MsvcrtexKind {
    /// DLL: DllMainCRTStartup + _CRT_INIT (+ default DllMain/DllEntryPoint).
    Dll,
    /// ANSI exe: [Win]?mainCRTStartup; with a GUI WinMain → crt0 `main`.
    ExeAnsi,
    /// wide exe: w[main|WinMain]CRTStartup; with a GUI wWinMain → crt0 `wmain`.
    ExeWide,
    /// Just the shared CRT floor + asm (custom native entry, no main/dll init).
    Bare,
}

/// Final PE-module flavor. Carries the entry symbol (already i386
/// stdcall-decorated, e.g. `_HalInitSystem@8`) and preferred image base.
pub enum ModuleKind {
    /// `kernelexe` — not `-shared`, `--gc-sections` (e.g. ntoskrnl.exe:
    /// `_KiSystemStartup@4`, 0x00400000).
    KernelExe { entry: String, image_base: u64 },
    /// `kerneldll` — `-shared`, no `--gc-sections` (e.g. hal.dll:
    /// `_HalInitSystem@8`, 0x00010000).
    KernelDll { entry: String, image_base: u64 },
    /// `win32dll` — user-mode PE DLL (`-shared`), exports via `.def` so
    /// NO `--exclude-all-symbols`, default PE alignment (file 0x200), no
    /// `--gc-sections`. `entry` is the i386-decorated entry symbol —
    /// `___ImageBase` for ntdll.dll (AddressOfEntryPoint = 0) or
    /// `_DllMain@12` for ntdll_vista.dll. `subsystem`: `Some("console:5.01")`
    /// for ntdll.dll (the win32dll set_subsystem hack); `None` lets lld
    /// default (GUI 6.0) — matches ntdll_vista.dll, which passes no
    /// `--subsystem`.
    Win32Dll {
        entry: String,
        /// `None` relies on the common `--enable-auto-image-base`: cmake's i386
        /// win32dll link passes no `--image-base` for a DLL `baseaddress.cmake`
        /// does not list (indicdll, umandlg), and lld hashes the output name.
        image_base: Option<u64>,
        subsystem: Option<String>,
    },
    /// `efi_application` — a UEFI app (uefildr.efi). Mirrors the cmake
    /// `uefildr.efi` link (set_subsystem 10 / `set_entrypoint EfiEntry`):
    /// `--subsystem,efi_application:5.01`, `--gc-sections` + no `-shared`
    /// (it's an exe), `--exclude-all-symbols` (exports nothing — the
    /// freeldr.spec/.def is only for the freeldr import lib rosload consumes,
    /// not for uefildr's own exports). `entry` is already arch-decorated
    /// (i386 `_EfiEntry`, amd64/arm64 `EfiEntry`).
    ///
    /// Per-arch knobs (all verbatim from each `build-<arch>/build.ninja`):
    /// - `alignment`: file == section alignment. x86 (i386/amd64) = 0x200;
    ///   arm64 = 0x1000.
    /// - `image_base`: `Some(b)` emits `--image-base 0x{b}`; `None` relies on
    ///   the common `--enable-auto-image-base` (lld hashes the output name).
    ///   amd64 cmake pins `Some(0x10000)` to stay in the low 1GB freeldr maps;
    ///   i386 cmake auto-resolves to the exe default 0x00400000 (passed
    ///   explicitly as `Some(0x400000)` — identical header); arm64 cmake
    ///   passes no `--image-base`, so `None` to match its auto-hash base.
    EfiApplication {
        entry: String,
        image_base: Option<u64>,
        alignment: u64,
    },
    /// `nativeexe` WITH exports — the rosload.exe 2nd-stage loader. Mirrors
    /// the cmake `rosload.exe` link (`set_subsystem native` /
    /// `set_entrypoint RunLoader` / `set_image_base 0x10000`,
    /// `ENABLE_EXPORTS TRUE` + `rosload.def`):
    /// - `--subsystem,native:5.01`, `--gc-sections`, NOT `-shared` (it's an
    ///   exe), image base `0x10000`.
    /// - exports come from the `.spec2def()`-generated `rosload.def`, so NO
    ///   `--exclude-all-symbols` (the one tiny export is `RtlAssert`).
    /// - DEFAULT PE alignment: cmake's rosload link passes no
    ///   `--file-alignment`/`--section-alignment`, so neither does this kind
    ///   (lld defaults file 0x200 / section 0x1000) — unlike KernelExe which
    ///   forces 0x1000/0x1000.
    /// `entry` is already arch-decorated (i386 `_RunLoader`, amd64 `RunLoader`).
    /// The i386-only `-u _HalDispatchTable`/`-u _HalPrivateDispatchTable`
    /// force-undefined pulls (the C halstub.c always linked the Rust HAL
    /// dispatch tables in as a source object; as archive members they'd be
    /// GC'd) are supplied via `.force_undefined(...)` on the builder.
    NativeExe { entry: String, image_base: u64 },
    /// The BIOS FreeLDR body (`freeldr_pe`, the PE half of `freeldr.sys`; the
    /// 16-bit `frldr16.bin` is concatenated in front of it by cmake). Mirrors
    /// the pre-prune cmake link (pcat.cmake): native subsystem exe, image base
    /// `0x10000`, file AND section alignment `0x200` (the real-mode stub
    /// relocates the image as laid out on disk), `--gc-sections`, and EXPORTS
    /// from `freeldr.spec` via `.spec2def()`: rosload.exe imports its loader
    /// services from `freeldr.sys` by name, so unlike uefildr (which calls the
    /// boot manager directly) the BIOS loader must carry an export table.
    BiosLoader { entry: String, image_base: u64 },
    /// `win32gui`/`win32cui` — a user-mode Win32 exe (e.g. lsass.exe). Mirrors
    /// the cmake `set_module_type(<m> win32gui)` link: explicit `--subsystem`
    /// (`windows:5.01` for GUI, `console:5.01` for CUI), `--gc-sections`, NOT
    /// `-shared` (it's an exe), NO `--exclude-all-symbols` (nothing to export),
    /// and NO explicit `--image-base` (the common `--enable-auto-image-base`
    /// resolves it, exactly as cmake's win32 exe link does). `entry` is the CRT
    /// startup the C runtime provides — `_wWinMainCRTStartup` (GUI) /
    /// `_mainCRTStartup` (CUI), force-pulled from `libmsvcrtex.a`, which calls
    /// the Rust `wWinMain`/`main`. All the version/auto-base/allow-multiple-def/
    /// disable-stdcall-fixup flags come from the common block. A user-mode NATIVE app
    /// (`set_module_type nativecui`: autochk, usetup) is this kind too, with
    /// `native:5.01` and `NtProcessStartup`: cmake links it the same way.
    Win32Exe { entry: String, subsystem: String },
}

/// Which `-D` set the cc-rs asm/C compiles get. Mirrors cmake's
/// per-subsystem DEFINES line.
#[derive(Clone, Copy)]
pub enum AsmDefineSet {
    /// `_NTOSKRNL_`/`_NTSYSTEM_`/`KDBG`/`NTDDI_VERSION` (ntoskrnl).
    Kernel,
    /// `_NTHALDLL_`/`_NTHAL_`, no `KDBG`/`NTDDI` (hal).
    Hal,
}

/// Per-arch toolchain + codegen facts. Built from `CARGO_CFG_TARGET_ARCH`
/// in `Linker::new`. i386 was the original (only) arch; amd64/arm64 added
/// for the ntdll cargo-drive. Threaded through clang triple, build dir,
/// spec2def/dlltool/clang_rt selectors, and the `_M_*`/`_X_` define set.
#[derive(Clone, Copy)]
struct ArchCfg {
    /// ReactOS arch name + `build-<name>` dir (`i386`/`amd64`/`arm64`).
    name: &'static str,
    /// llvm-mingw target triple / binary prefix (`<triple>-clang`).
    triple: &'static str,
    /// Standard *user-mode* Rust target triple (hardfloat, precompiled std —
    /// NO build-std). Tier-B sibling crates (rtl-rust/utf8/cnt-rust-lib/…) are
    /// USER-mode and MUST build with this, exactly like cmake — building them
    /// against the kernel json (`x86-softfloat`) corrupts cnt-rust-lib's
    /// by-value f64 ABI and crashes i386 boot (2026-06-02).
    rust_user_target: &'static str,
    /// spec2def `-a=` value.
    spec2def: &'static str,
    /// dlltool `-m` machine.
    dlltool_machine: &'static str,
    /// `libclang_rt.builtins-<rt>.a` suffix.
    clang_rt: &'static str,
    /// `_M_*` / `_X_` / `__arch__` defines (the only arch-varying part of
    /// the kernel/HAL define sets).
    arch_defines: &'static [(&'static str, Option<&'static str>)],
    /// true on x86 (i386+amd64) — gates `-mlong-double-64` etc.; false arm64.
    is_x86: bool,
    /// true on i386 only — gates `-march=pentium`/`-mstackrealign`.
    is_i386: bool,
}

const I386_ARCH_DEFINES: &[(&str, Option<&str>)] = &[
    ("_M_IX86", None),
    ("_X86_", None),
    ("__i386__", None),
    ("i386", None),
];
const AMD64_ARCH_DEFINES: &[(&str, Option<&str>)] = &[
    ("_M_AMD64", None),
    ("_AMD64_", None),
    ("_WIN64", None),
    ("__x86_64__", None),
];
const ARM64_ARCH_DEFINES: &[(&str, Option<&str>)] = &[
    ("_M_ARM64", None),
    ("_ARM64_", None),
    ("_WIN64", None),
    ("__aarch64__", None),
];

fn arch_cfg(target_arch: &str) -> ArchCfg {
    match target_arch {
        "x86" => ArchCfg {
            name: "i386",
            triple: "i686-w64-mingw32",
            rust_user_target: "i686-pc-windows-gnu",
            spec2def: "i386",
            dlltool_machine: "i386",
            clang_rt: "i386",
            arch_defines: I386_ARCH_DEFINES,
            is_x86: true,
            is_i386: true,
        },
        "x86_64" => ArchCfg {
            // spec2def's arch token is `x86_64`, NOT `amd64` (an unknown
            // token silently falls back to i386 → @N stdcall decoration on
            // a 64-bit .def → undefined-symbol link errors).
            name: "amd64",
            triple: "x86_64-w64-mingw32",
            rust_user_target: "x86_64-pc-windows-gnu",
            spec2def: "x86_64",
            dlltool_machine: "i386:x86-64",
            clang_rt: "x86_64",
            arch_defines: AMD64_ARCH_DEFINES,
            is_x86: true,
            is_i386: false,
        },
        "aarch64" => ArchCfg {
            name: "arm64",
            triple: "aarch64-w64-mingw32",
            rust_user_target: "aarch64-pc-windows-gnullvm",
            spec2def: "arm64",
            dlltool_machine: "arm64",
            clang_rt: "aarch64",
            arch_defines: ARM64_ARCH_DEFINES,
            is_x86: false,
            is_i386: false,
        },
        other => panic!("reactos-link: unsupported target arch {other}"),
    }
}

/// Common (arch-agnostic) ntoskrnl DEFINES — the arch-specific `_M_*` come
/// from `ArchCfg::arch_defines`.
const KERNEL_DEFINES_COMMON: &[(&str, Option<&str>)] = &[
    ("D3D_UMD_INTERFACE_VERSION", Some("0x000C")),
    ("DBG", Some("1")),
    ("DLL_EXPORT_VERSION", Some("0x502")),
    ("DXGKDDI_INTERFACE_VERSION", Some("0x1052")),
    ("KDBG", None),
    ("MINGW_HAS_SECURE_API", Some("1")),
    ("NTDDI_VERSION", Some("0x05020400")),
    ("NTKRNLVISTA", None),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", Some("0x502")),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTOSKRNL_", None),
    ("_NTSYSTEM_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", Some("0x502")),
    ("_USE_DUMMY_PSEH", Some("1")),
    ("_WIN32_IE", Some("0x603")),
    ("_WIN32_WINDOWS", Some("0x502")),
    ("_WIN32_WINNT", Some("0x502")),
    ("__REACTOS__", None),
    ("__ROS_ROSSYM__", None),
];

/// Common (arch-agnostic) HAL asm DEFINES.
const HAL_ASM_DEFINES_COMMON: &[(&str, Option<&str>)] = &[
    ("D3D_UMD_INTERFACE_VERSION", Some("0x000C")),
    ("DBG", Some("1")),
    ("DLL_EXPORT_VERSION", Some("0x502")),
    ("DXGKDDI_INTERFACE_VERSION", Some("0x1052")),
    ("MINGW_HAS_SECURE_API", Some("1")),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", Some("0x502")),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTHALDLL_", None),
    ("_NTHAL_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", Some("0x502")),
    ("_USE_DUMMY_PSEH", Some("1")),
    ("_WIN32_IE", Some("0x603")),
    ("_WIN32_WINDOWS", Some("0x502")),
    ("_WIN32_WINNT", Some("0x502")),
    ("__REACTOS__", None),
];

/// Common set for `set`, plus the arch's `_M_*`/`__arch__` defines, plus
/// `_USE_32BIT_TIME_T` on i386 only.
fn defines(set: AsmDefineSet, arch: &ArchCfg) -> Vec<(&'static str, Option<&'static str>)> {
    let common = match set {
        AsmDefineSet::Kernel => KERNEL_DEFINES_COMMON,
        AsmDefineSet::Hal => HAL_ASM_DEFINES_COMMON,
    };
    let mut v: Vec<_> = common.to_vec();
    v.extend_from_slice(arch.arch_defines);
    if arch.is_i386 {
        v.push(("_USE_32BIT_TIME_T", None));
    }
    v
}

#[derive(Clone)]
enum RustTarget {
    Json(PathBuf),
    Builtin(String),
}

impl RustTarget {
    fn new(manifest_dir: &Path, target_spec: &str) -> Self {
        if target_spec.ends_with(".json") {
            Self::Json(manifest_dir.join(target_spec))
        } else {
            Self::Builtin(target_spec.to_string())
        }
    }

    fn arg(&self) -> String {
        match self {
            Self::Json(path) => path.display().to_string(),
            Self::Builtin(triple) => triple.clone(),
        }
    }

    fn stem(&self) -> &str {
        match self {
            Self::Json(path) => path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("target json stem"),
            Self::Builtin(triple) => triple.as_str(),
        }
    }

    fn is_json(&self) -> bool {
        matches!(self, Self::Json(_))
    }
}

/// Builder for one cargo-driven module link.
pub struct Linker {
    repo_root: PathBuf,
    manifest_dir: PathBuf,
    build_dir: PathBuf,
    pkg_name: String,
    target: RustTarget,
    arch: ArchCfg,
    rust_crate: Option<String>,
    rust_features: Vec<String>,
    /// Force the `rust_staticlib` main body to build the STANDARD user-mode
    /// target (`arch.rust_user_target`, stable, NO build-std) even for a
    /// kernel-ish module kind (EfiApplication/NativeExe). FreeLDR's body
    /// (freeldr-rust) is built exactly like cmake did it — standard target,
    /// not the `x86-softfloat` kernel json — because the pre-boot loader has
    /// SSE disabled and the kernel json's softfloat ABI would diverge from the
    /// hardfloat+`-sse,-sse2` codegen the working binary ships.
    rust_user_target: bool,
    /// `RUSTFLAGS` for the `rust_staticlib` main-body cargo build. Mirrors a
    /// cmake `RUSTFLAGS=…` env on the custom_command — e.g. freeldr-rust's i386
    /// `-C target-feature=-sse,-sse2`. Only consulted in the `kernel=false`
    /// (user-target) build path; `None` → no override.
    rust_build_rustflags: Option<String>,
    /// Extra source dirs (repo-relative) to `rerun-if-changed`, beyond each
    /// rust crate's own dir (which build_rust_crate tracks). For path-DEPS of
    /// the body crate living in OTHER workspace crates — e.g. freeldr-rust
    /// depends on cmboot-rust, so editing cmboot-rust must also re-run build.rs
    /// to rebuild the body. cmake's EXTRA_DEPENDS only re-invokes cargo; this
    /// makes cargo actually re-run build.rs.
    rust_watch_dirs: Vec<String>,
    /// Extra sibling Rust staticlib crates (crate, features, default_features)
    /// cargo-built alongside the main one — Tier-B of the ntdll cargo-drive
    /// (replaces reuse of cmake-built `librtl_rust.a`/`libutf8.a`/… with a
    /// direct cargo build). All share the main's `target/<pkg>-rustlib` dir so
    /// build-std compiles core/alloc once.
    rust_libs: Vec<(String, Vec<String>, bool)>,
    c_shims: Vec<String>,
    /// Extra clang flags applied to every `c_shims` compile (on top of
    /// `apply_base`). Used for module-specific codegen requirements the kernel
    /// default doesn't carry — e.g. freeldr's `-mno-sse` (the pre-boot
    /// environment hasn't enabled SSE, so an emitted `movaps` would #UD).
    c_shim_flags: Vec<String>,
    asm: Vec<String>,
    asm_set: AsmDefineSet,
    /// Extra `-D` defines applied to the `asm()` compiles ON TOP of the
    /// `asm_set`'s list. Empty by default → unchanged for every existing
    /// consumer. Used by ntkrnlmp-cargo to assemble the kernel `.S` with
    /// `CONFIG_SMP` (the `#ifdef CONFIG_SMP` spinlock/fast-path blocks in the
    /// generated ks*.inc) — exactly cmake's `target_compile_definitions(
    /// ntkrnlmp PRIVATE CONFIG_SMP)` for the asm. NOT applied to the cntpr
    /// c_from_compile_db residue (that replays libcntpr's recorded CRT command).
    asm_extra_defines: Vec<(String, Option<String>)>,
    /// Extra `-I` dirs (repo-relative) for the asm compiles. Lets `asm()`
    /// assemble RAW `.S` files with `#include`s resolved against a MINIMAL set
    /// of source header dirs (e.g. freeldr's `sdk/include/asm` for asm.inc +
    /// `boot/freeldr/freeldr/include` for pcbios.h) instead of pre-expanding or
    /// reproducing the whole module compile env. Empty by default → unchanged
    /// for pre-expanded-`.s` consumers (hal).
    asm_include_dirs: Vec<String>,
    resource: Option<String>,
    /// Like `resource`, but the `.res` is a cmake-BUILT artifact (build-dir
    /// relative) rather than a checked-in consumer-crate file. Used by the
    /// spine binaries whose `.rc` is compiled by cmake (e.g. lsass.rc.res).
    resource_built: Option<String>,
    /// Compile the `.rc` IN-cargo by replaying cmake's flag-exact `windres`
    /// invocation from `compile_commands.json` (vs `resource_built`, which
    /// reuses a cmake-built `.res`). (rc-rel-path, cmake `_res` target name).
    /// Drops the `ninja <m>_res` prereq — see `resource_from_rc`.
    resource_from_rc: Option<(String, String)>,
    /// The `.rc` compiled by the engine from cmake's rule rebuilt out of the committed
    /// per-arch template (`cflags/rc.<arch>.flags`) and the module's own includes,
    /// extra flags and defines (see `rc`). No cmake target behind it.
    rc: Option<RcSpec>,
    /// C libraries the engine compiles with cmake's C rule rebuilt from the committed
    /// `cflags/c.<arch>.flags` and their own part (see `c_compiled`).
    c_compiled: Vec<CSpec>,
    spec: Option<(String, String)>,
    link_stubs: bool,
    static_libs: Vec<String>,
    /// cmake-built libs linked AFTER the Rust archives (see `static_libs_after_body`).
    late_static_libs: Vec<String>,
    /// cmake's own order of the body, import libs and static libs (`link_order`).
    link_order: Option<Vec<String>>,
    /// `--no-gc-sections` after rustc's own `--gc-sections` (`keep_all_sections`).
    keep_all_sections: bool,
    /// Pre-expanded asm assembled into a `.a` and linked INSIDE the
    /// `--start-group` as an on-demand archive (NOT a force-included object
    /// like `asm`). (archive-basename, asm-rels). Use this when the cmake
    /// equivalent was an archive (e.g. ntdllsys) and force-including it as an
    /// object relocates the image enough to surface a latent layout-sensitive
    /// miscompile — keeping it an in-group archive preserves the cmake layout.
    asm_archives: Vec<(String, Vec<String>)>,
    /// Deep-C residue compiled IN-cargo by replaying cmake's own clang
    /// invocation from `build-<arch>/compile_commands.json` (flag-exact),
    /// archived into an in-group `.a` in the C-archive position. Replaces
    /// reuse of a cmake-built `.a` via `static_libs` (e.g. ntdll's Tier-C
    /// librtl / librtl_vista / libcntpr, or the loaders' deep-C blcmlib /
    /// cportlib) — drops the `ninja <lib>` prerequisite.
    /// (archive-basename, source rel-paths). See `c_from_compile_db`.
    c_db_libs: Vec<(String, Vec<String>)>,
    /// c_db lib names whose source list includes the 3 gendib-GENERATED DIB
    /// tables (`dib{8,16,32}gen.c`). In a cmake build they exist under
    /// `build-<arch>/` (replay handles them via the generated-source fallback);
    /// cmake-off they're produced in-build by the self-hosted `native-gendib`.
    /// See `gendib`.
    gendib_libs: Vec<String>,
    /// (spec, dll, basename, delay-loaded), in call order: the link order decides which
    /// lib an `__imp_` symbol two of them define comes from (see `delay_import_lib`).
    import_libs: Vec<(String, String, String, bool)>,
    force_undefined: Vec<String>,
    allow_stdcall_fixup: bool,
    /// `Some(n)` emits `-Wl,--stack,<n>` (reserve, in bytes). For native exes
    /// with deep recursive import chains — e.g. arm64 csrss.exe, where
    /// llvm-dlltool converts delay-imports to regular ones so every dependency
    /// loads at once and LdrpLoadDll recursion overflows the default 1MB stack.
    stack_reserve: Option<u64>,
    ks_includes: native_deps::InvocationOutput,
    /// Raw link arguments appended after the kind's own (see `link_args`).
    extra_link_args: Vec<String>,
    /// The bin this link builds, when the package has several (see `for_bin`).
    bin: Option<String>,
}

impl Linker {
    /// `target_spec` is either a target json path relative to the consumer crate
    /// dir (e.g. `"i686-kernel.json"`) or a builtin Rust target triple (e.g.
    /// `"aarch64-pc-windows-gnullvm"`). Json kernel targets use nightly
    /// build-std; builtin targets use stable/precompiled std. Arch is taken from
    /// `CARGO_CFG_TARGET_ARCH` (x86 / x86_64 / aarch64).
    pub fn new(target_spec: &str) -> Linker {
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
        let arch = arch_cfg(&target_arch);
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        // sdk/lib/rust/<crate> → repo root is 4 levels up.
        let repo_root = manifest_dir
            .ancestors()
            .nth(4)
            .expect("repo root")
            .to_path_buf();
        // Absent is a cmake-off build: every path that reads the build dir has its own
        // cmake-off branch, or fails naming what it needs.
        let build_dir = repo_root.join(format!("build-{}", arch.name));
        let pkg_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME");
        let target = RustTarget::new(&manifest_dir, target_spec);
        println!("cargo:rerun-if-changed=build.rs");
        if let RustTarget::Json(path) = &target {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        Linker {
            repo_root,
            manifest_dir,
            build_dir,
            pkg_name,
            target,
            arch,
            rust_crate: None,
            rust_features: Vec::new(),
            rust_user_target: false,
            rust_build_rustflags: None,
            rust_watch_dirs: Vec::new(),
            rust_libs: Vec::new(),
            c_shims: Vec::new(),
            c_shim_flags: Vec::new(),
            asm: Vec::new(),
            asm_set: AsmDefineSet::Kernel,
            asm_extra_defines: Vec::new(),
            asm_include_dirs: Vec::new(),
            resource: None,
            resource_built: None,
            resource_from_rc: None,
            rc: None,
            c_compiled: Vec::new(),
            spec: None,
            link_stubs: false,
            static_libs: Vec::new(),
            late_static_libs: Vec::new(),
            link_order: None,
            keep_all_sections: false,
            asm_archives: Vec::new(),
            c_db_libs: Vec::new(),
            gendib_libs: Vec::new(),
            import_libs: Vec::new(),
            force_undefined: Vec::new(),
            allow_stdcall_fixup: false,
            stack_reserve: None,
            ks_includes: Default::default(),
            extra_link_args: Vec::new(),
            bin: None,
        }
    }

    /// Suppress the default `--disable-stdcall-fixup`, letting lld resolve a
    /// decorated `_Sym@N` reference against an undecorated `_Sym` definition
    /// (its default fixup-with-warning behavior). winsrv.dll needs this: its
    /// cmake link relies on the fixup to paper over winsrv-rust's cross-module
    /// extern "C"-def / extern "system"-call ABI mismatches (a pre-existing
    /// translation-quality issue). Reproduces cmake's link exactly. Do NOT use
    /// to hide a fresh ABI bug — fix the source instead.
    /// Link only bin `name` of a package that has several. A FAMILY cell (the 95
    /// keyboard layouts) is one package with a `[[bin]]` per module: its build.rs
    /// makes one Linker per bin, and every argument is emitted as
    /// `rustc-link-arg-bin=<name>=…`, so each PE gets exactly its own link line while
    /// the shared body crate is built once. Intermediate files go in a per-bin dir.
    pub fn for_bin(mut self, name: &str) -> Self {
        self.bin = Some(name.to_string());
        self
    }

    /// Raw link arguments, emitted after the kind's own flags, for what the cmake
    /// link passes that no ModuleKind models (the keyboard layouts' `/MERGE`s into
    /// one `.data` section).
    pub fn link_args(mut self, args: &[&str]) -> Self {
        self.extra_link_args.extend(args.iter().map(|a| a.to_string()));
        self
    }

    /// One import lib (regular or delay).
    fn emit_import_lib(&self, lib: &(String, String, String, bool)) {
        let (spec_rel, dll_name, basename, delay) = lib;
        let a = if *delay {
            self.generate_delay_import_lib(spec_rel, dll_name, basename)
        } else {
            self.generate_import_lib(spec_rel, dll_name, basename)
        };
        link_arg!(self, "{}", a.display());
        println!(
            "cargo:rerun-if-changed={}",
            self.repo_root.join(spec_rel).display()
        );
    }

    /// The module's own compiled archives: the compile_commands.json replays
    /// (c_from_compile_db) and the pre-expanded asm archives.
    fn emit_own_archives(&self, clang: &Path, ar: &Path) {
        // ---- Deep-C residue compiled in-cargo (compile_commands.json replay).
        // In the same C-archive region as static_libs (before the Rust
        // archives). Flag-exact: replay cmake's own clang command per source,
        // strip the ninja-built .pch (keep the configure-time cmake_pch.h
        // force-include), redirect -o into OUT_DIR. Drops `ninja <lib>` for the
        // residue that static_libs() used to reuse.
        if !self.c_db_libs.is_empty() {
            let db_path = self.build_dir.join("compile_commands.json");
            if cmake_off_forced() || !db_path.exists() {
                // cmake-off (WS1 DEEP): no compile_commands.json. Compile each
                // c_db lib from its committed per-arch flag template
                // (`sdk/lib/rust/reactos-link/cflags/<name>.<arch>.flags`) plus
                // the native-hpp-generated xdk headers (`winnt.h` etc.). Mirrors
                // the WS2 (`ks*.inc`) / WS3 (`.res`) committed-fallback pattern:
                // cmake's compile_commands.json wins when a build is configured
                // (the replay branch below — it can never go stale against the
                // committed flags), the committed flags fill in only when there
                // is no cmake build at all. Regen via `tools/gen_cflags.py`.
                self.compile_c_db_committed(clang, ar);
            } else {
            let db_text = std::fs::read_to_string(&db_path).unwrap_or_else(|e| {
                panic!(
                    "c_from_compile_db needs {} — configure cmake with \
                     -DCMAKE_EXPORT_COMPILE_COMMANDS=ON: {e}",
                    db_path.display()
                )
            });
            let entries = parse_ccdb(&db_text);
            println!("cargo:rerun-if-changed={}", db_path.display());
            for (name, sources) in &self.c_db_libs {
                let rels: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
                let lib_a = self.replay_c_archive(name, &rels, &entries, ar);
                link_arg!(self, "{}", lib_a.display());
            }
            } // end else (cmake build configured → compile_commands.json replay)
        }
        for spec in &self.c_compiled {
            let lib_a = self.compile_c_spec(spec, ar);
            link_arg!(self, "{}", lib_a.display());
        }
        // Pre-expanded asm assembled into on-demand archives, IN the group
        // alongside the cmake C archives (same archive semantics + layout
        // region as the cmake equivalent — see asm_archive()).
        for (name, rels) in &self.asm_archives {
            let rels: Vec<&str> = rels.iter().map(|s| s.as_str()).collect();
            let lib_a = self.build_archive(name, &rels, &[], clang, ar);
            link_arg!(self, "{}", lib_a.display());
        }
    }

    fn emit_link_arg(&self, arg: &str) {
        match &self.bin {
            Some(b) => println!("cargo:rustc-link-arg-bin={b}={arg}"),
            None => println!("cargo:rustc-link-arg={arg}"),
        }
    }

    fn emit_objs(&self, build: &cc::Build, source: &Path) {
        trace_cc(build, source);
        let objs = native_deps::compile(build, source);
        if objs.is_empty() {
            panic!("cc-rs produced no objects for {}", source.display());
        }
        for obj in objs {
            link_arg!(self, "{}", obj.display());
        }
    }

    pub fn allow_stdcall_fixup(mut self) -> Self {
        self.allow_stdcall_fixup = true;
        self
    }

    /// Reserve `bytes` of stack (`-Wl,--stack,<bytes>`) instead of the default
    /// 1MB. Needed where a native exe's import graph forces a deep recursive
    /// LdrpLoadDll chain (arm64 csrss.exe — delay-imports collapse to regular
    /// on llvm-dlltool, loading every dependency at once). Chainable.
    pub fn stack_reserve(mut self, bytes: u64) -> Self {
        self.stack_reserve = Some(bytes);
        self
    }

    /// Workspace crate (`crate-type = ["staticlib"]`) that is the module
    /// body. build.rs shells out to build it (custom target json +
    /// build-std nightly) since cargo can't link a staticlib as a dep.
    pub fn rust_staticlib(mut self, crate_name: &str) -> Self {
        self.rust_crate = Some(crate_name.to_string());
        self
    }

    /// Extra cargo `--features` for the `rust_staticlib` build (kept
    /// additive — default features stay on, mirroring cmake's
    /// `cargo build … -p ntdll-rust --features vista`). e.g. ntdll_vista.dll
    /// builds ntdll-rust with `["vista"]` (DllMain/LdrpHeap/ldrnotify).
    pub fn rust_features(mut self, feats: &[&str]) -> Self {
        self.rust_features = feats.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the `rust_staticlib` main body with the STANDARD user-mode target
    /// (stable, no build-std) instead of the kernel json — see
    /// `rust_user_target` field. Used by the freeldr loaders so freeldr-rust is
    /// built byte-for-byte as cmake did (the custom_command this replaces ran
    /// `cargo build --target i686-pc-windows-gnu`, NOT the kernel json).
    pub fn rust_user_target(mut self) -> Self {
        self.rust_user_target = true;
        self
    }

    /// `RUSTFLAGS` for the `rust_staticlib` main-body build (user-target path).
    /// Mirrors the cmake custom_command's `RUSTFLAGS=…` env — e.g. freeldr-rust
    /// i386 `-C target-feature=-sse,-sse2` (pre-boot has no SSE). Caller sets it
    /// per-arch. Chainable.
    pub fn rust_build_rustflags(mut self, flags: &str) -> Self {
        self.rust_build_rustflags = Some(flags.to_string());
        self
    }

    /// Emit `rerun-if-changed` for extra repo-relative source dirs (path-deps
    /// of the body crate that live in other workspace crates, e.g.
    /// `cmboot-rust/src`). See `rust_watch_dirs`. Chainable.
    pub fn rust_watch(mut self, dirs: &[&str]) -> Self {
        self.rust_watch_dirs
            .extend(dirs.iter().map(|s| format!("sdk/lib/rust/{s}")));
        self
    }

    /// An additional sibling Rust staticlib crate cargo-built alongside the
    /// main `rust_staticlib` and linked into the module — the Tier-B path of
    /// the ntdll cargo-drive that replaces reuse of a cmake-built rust `.a`
    /// (e.g. `librtl_rust.a`, `libutf8.a`, `libcnt_rust_lib.a`,
    /// `libsmss_csrss_rust.a`) with a direct cargo build. `default_features`
    /// = false maps to `--no-default-features` (e.g. smss-csrss-rust built
    /// `csrlib_only`). The produced `lib<crate_>.a` is linked in the
    /// start-group after the main crate, before the cmake static libs.
    /// Chainable — call once per sibling crate, in link order.
    pub fn rust_lib(mut self, crate_name: &str, features: &[&str], default_features: bool) -> Self {
        self.rust_libs.push((
            crate_name.to_string(),
            features.iter().map(|s| s.to_string()).collect(),
            default_features,
        ));
        self
    }

    /// Header-free C shims (repo-relative). Compiled with the Kernel
    /// define set + clang's own stdarg/stddef only.
    pub fn c_shims(mut self, rels: &[&str]) -> Self {
        self.c_shims = rels.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Extra clang flags for the `c_shims` compiles (e.g. freeldr's `-mno-sse`).
    /// The caller is responsible for arch-validity (don't pass `-mno-sse` on
    /// aarch64). Chainable; applies to ALL shims in this `c_shims` set.
    pub fn c_shim_flags(mut self, flags: &[&str]) -> Self {
        self.c_shim_flags = flags.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Asm sources (repo-relative). Either pre-expanded `.s` blobs (all
    /// `#include`s resolved out-of-band) OR raw `.S` files whose `#include`s
    /// resolve against `asm_include_dirs()` — see that method. Compiled with
    /// `set`'s define list + the arch defines + `__ASM__`.
    pub fn asm(mut self, rels: &[&str], set: AsmDefineSet) -> Self {
        self.asm = rels.iter().map(|s| s.to_string()).collect();
        self.asm_set = set;
        self
    }

    /// Extra `-D` defines layered on top of the `asm()` define set (e.g.
    /// `CONFIG_SMP` for ntkrnlmp). Empty by default; chainable.
    pub fn asm_extra_defines(mut self, defs: &[(&str, Option<&str>)]) -> Self {
        self.asm_extra_defines = defs
            .iter()
            .map(|(n, v)| (n.to_string(), v.map(|s| s.to_string())))
            .collect();
        self
    }

    /// Extra `-I` dirs (repo-relative) for the `asm()` compiles, so a RAW `.S`
    /// can `#include` real source headers (asm.inc, pcbios.h, …) directly —
    /// the MINIMAL set the file actually needs, NOT the module's whole `-I`
    /// env. Empty by default (pre-expanded `.s` consumers need none). Chainable.
    pub fn asm_include_dirs(mut self, dirs: &[&str]) -> Self {
        self.asm_include_dirs = dirs.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Pre-expanded asm assembled into `lib<name>.a` and linked as an
    /// on-demand archive INSIDE the `--start-group` (vs `asm()`, which
    /// force-includes objects before the group). Mirrors a cmake module that
    /// shipped the asm as an archive (e.g. `ntdllsys` = the Nt*/Zw* syscall
    /// stubs): force-including those stubs as an object relocated ~5% of the
    /// i386 image and surfaced a latent layout-sensitive hivewrt miscompile;
    /// the in-group archive preserves the cmake link layout. Uses the same
    /// `asm_set` define list as `asm()`.
    pub fn asm_archive(mut self, name: &str, rels: &[&str]) -> Self {
        self.asm_archives.push((
            name.to_string(),
            rels.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Link the mingw CRT-startup floor (`msvcrtex`) — the "hidden" CRT glue
    /// every user-mode PE needs: the entry point (`DllMainCRTStartup` /
    /// `[w]WinMainCRTStartup` / `[w]mainCRTStartup`), `_CRT_INIT` + the
    /// `.CRT$XI*/XC*` initializer-section walk (`_initterm`), the security
    /// cookie, `_pei386_runtime_relocator`, TLS callbacks, the `__main` ctor
    /// hook, `_fltused`, _invalid_parameter/rand_s/mbrtowc/wcrtomb, etc.
    ///
    /// The C floor is now Rust: `crt-startup-rust` (one `kind`-selected
    /// feature, mirroring the C archive's lazy crtdll.o/crtexe.o/wcrtexe.o
    /// pull) plus the overridable siblings `crt-dllmain-rust` (default
    /// DllMain/DllEntryPoint) and `crt0-rust` (the GUI WinMain→main bridge).
    /// Only the i386/amd64 MSVC stack-probe + 64-bit integer intrinsics
    /// (`_chkstk`/`_alldiv`/`_aulldiv`/i386 ceilf/floorf) remain asm — linked
    /// via the sibling `_asm` archive (the asm `#include`s asm.inc / ks386.inc,
    /// resolved by the kernel asm `-I` chain like ntdll's chkstk_ms). arm64
    /// needs no asm (clang_rt.builtins-aarch64 covers `__chkstk`, no MSVC
    /// integer-intrinsic names). Drop `sdk/lib/crt/libmsvcrtex.a` from
    /// `static_libs` when calling this.
    ///
    /// `dbgrpt.cpp` (the `_CrtDbgReport*` C++ debug-report facility) is NOT
    /// translated — it is lazy-pulled only by a `_DEBUG` libstdc++ and is
    /// unreferenced in our release links.
    pub fn msvcrtex_crt(mut self, kind: MsvcrtexKind) -> Self {
        let asm: &[&str] = match self.arch.name {
            "i386" => &[
                "sdk/lib/crt/except/i386/chkstk_asm.s",
                "sdk/lib/crt/except/i386/chkstk_ms.s",
                "sdk/lib/crt/math/i386/alldiv_asm.s",
                "sdk/lib/crt/math/i386/aulldiv_asm.s",
                "sdk/lib/crt/math/i386/ceilf.S",
                "sdk/lib/crt/math/i386/floorf.S",
            ],
            "amd64" => &["sdk/lib/crt/except/amd64/chkstk_ms.s"],
            _ => &[],
        };
        if !asm.is_empty() {
            self.asm_archives.push((
                "msvcrtex_asm".to_string(),
                asm.iter().map(|s| s.to_string()).collect(),
            ));
        }

        // crt-startup-rust personality (no default features).
        let feat = match kind {
            MsvcrtexKind::Dll => &["crtdll"][..],
            MsvcrtexKind::ExeAnsi => &["crtexe"][..],
            MsvcrtexKind::ExeWide => &["wcrtexe"][..],
            MsvcrtexKind::Bare => &[][..],
        };
        self = self.rust_lib("crt-startup-rust", feat, false);

        // overridable siblings: default DllMain (dll), WinMain→main bridge (exe).
        self = match kind {
            MsvcrtexKind::Dll => self.rust_lib("crt-dllmain-rust", &[], true),
            MsvcrtexKind::ExeAnsi => self.rust_lib("crt0-rust", &["ansi"], false),
            MsvcrtexKind::ExeWide => self.rust_lib("crt0-rust", &["wide"], false),
            MsvcrtexKind::Bare => self,
        };
        self
    }

    /// Checked-in `.res` (consumer-crate-relative), regenerated
    /// out-of-band (`tools/gen_*_res.py`).
    pub fn resource(mut self, manifest_rel: &str) -> Self {
        self.resource = Some(manifest_rel.to_string());
        self
    }

    /// A cmake-BUILT `.res` (build-`<arch>`-relative), e.g.
    /// `base/system/lsass/CMakeFiles/lsass.dir/lsass.rc.res`. For spine
    /// binaries whose `.rc` cmake already compiles; reuses it rather than
    /// re-hosting the resource compile in build.rs. Needs `ninja <module>`
    /// (or a configure+rc build) to have produced it.
    pub fn resource_built(mut self, build_rel: &str) -> Self {
        self.resource_built = Some(build_rel.to_string());
        self
    }

    /// Compile the `.rc` IN-cargo by replaying cmake's exact `windres`
    /// invocation from `build-<arch>/compile_commands.json` (flag-exact: the
    /// full RC_INVOKED / LANGUAGE_* / per-arch define set + `-I` chain), instead
    /// of reusing a cmake-built `.res` (`resource_built`). Drops the
    /// `ninja <m>_res` prereq. `rc_rel` is the repo-relative `.rc`; `res_target`
    /// is the cmake `_res` OBJECT-lib target name (its `CMakeFiles/<res_target>.dir`
    /// is the disambiguating marker, like `c_from_compile_db`). The `_res` cmake
    /// target must still exist (to populate compile_commands.json) but its `.res`
    /// is no longer linked. Mutually exclusive with `resource`/`resource_built`.
    pub fn resource_from_rc(mut self, rc_rel: &str, res_target: &str) -> Self {
        self.resource_from_rc = Some((rc_rel.to_string(), res_target.to_string()));
        self
    }

    /// Compile the `.rc` with cmake's windres rule (`CMAKE_RC_COMPILE_OBJECT` in
    /// gcc.cmake), rebuilt from the committed `cflags/rc.<arch>.flags` and this
    /// module's part of it, which tools/cargo_cell_gen.py read from cmake's own command:
    ///
    ///   windres -O coff --codepage=65001 <includes> <template flags> <flags>
    ///           <template defs: RC_INVOKED, the LANGUAGE_* set> <sorted defines> rc out
    ///
    /// `includes` are repo-relative, `@build/<rel>` for the build tree, or `@std` for
    /// the template's standard run; `defines` are the module's own (`NAME[=VALUE]`), which
    /// join the template's global ones, sorted as cmake sorts them. With a configured
    /// build `@build` is `build-<arch>`; without one it is a tree the engine fills itself
    /// (the xdk headers, and `version.h`/`buildno.h` from `$VIBEOS_GEN_DIR`).
    pub fn rc(mut self, rc_rel: &str, includes: &[&str], flags: &[&str], defines: &[&str]) -> Self {
        let own = |v: &[&str]| v.iter().map(|x| x.to_string()).collect();
        self.rc = Some(RcSpec {
            rc_rel: rc_rel.to_string(),
            includes: own(includes),
            flags: own(flags),
            defines: own(defines),
        });
        self
    }

    /// `lib<name>.a` from `sources`, compiled with cmake's C rule:
    /// `<CC> --target <sorted defines> <includes> <flags> -o obj -c src`, where the
    /// compiler, the standard include run (`@std`), the core flags (`@core`) and the
    /// global defines come from `cflags/c.<arch>.flags` and the rest is this library's,
    /// as tools/cargo_cell_gen.py read it from cmake's own commands. Linked where
    /// `c_from_compile_db` archives go. `@build/<rel>` names a generated source or a
    /// build-tree include; `{REPO}`/`{BUILD}` in a flag are the roots.
    pub fn c_compiled(
        mut self,
        name: &str,
        sources: &[&str],
        includes: &[&str],
        flags: &[&str],
        defines: &[&str],
    ) -> Self {
        let own = |v: &[&str]| v.iter().map(|x| x.to_string()).collect();
        self.c_compiled.push(CSpec {
            name: name.to_string(),
            sources: own(sources),
            includes: own(includes),
            flags: own(flags),
            defines: own(defines),
        });
        self
    }

    /// `.spec` (repo-relative) → `.def` via self-hosted spec2def.
    /// `dll_name` is the `-n=` arg (e.g. `hal.dll`, `ntoskrnl.exe`).
    pub fn spec2def(mut self, spec_rel: &str, dll_name: &str) -> Self {
        self.spec = Some((spec_rel.to_string(), dll_name.to_string()));
        self
    }

    /// Also compile + link the spec2def-generated stub bodies
    /// (`module_stubs.c`, the `-stub` spec entries — e.g. ntdll's 48
    /// `Etw*`/`Rtl*` unimplemented exports). Opt-in: kernel modules don't
    /// link them (their `-stub` symbols resolve elsewhere), but a user-mode
    /// DLL whose `.def` exports them needs the bodies or the link fails
    /// undefined. The stub TU only `#include <stubs.h>` (self-contained:
    /// declares its own types + `DbgPrint`/`RtlRaiseException`), so it
    /// compiles header-free with a single `-I sdk/include/reactos`.
    pub fn spec_stubs(mut self) -> Self {
        self.link_stubs = true;
        self
    }

    /// cmake-built sibling static libs (build-dir-relative), in link
    /// order, inside the `--start-group`/`--end-group` ahead of the Rust
    /// archives. An entry `engine:<name>` is a library cmake used to build that
    /// the engine now builds itself, at the same place in the link: see
    /// `engine_lib` for the ones it knows.
    pub fn static_libs(mut self, builddir_rels: &[&str]) -> Self {
        self.static_libs = builddir_rels.iter().map(|s| s.to_string()).collect();
        self
    }

    /// cmake-built libs that cmake linked AFTER the module's Rust body, as a user-mode
    /// module's `target_link_libraries(m ${m_rust_LIB}) … target_link_libraries(m uuid)`
    /// does. Order decides a duplicate definition: regedit-rust and libuuid.a both
    /// define CLSID_AutoComplete, cmake kept the body's, and `static_libs` (which
    /// precede the body, as ntdll and the kernel need) kept uuid's — which
    /// link_guard refuses. Emitted inside the group, after the Rust archives.
    pub fn static_libs_after_body(mut self, builddir_rels: &[&str]) -> Self {
        self.late_static_libs = builddir_rels.iter().map(|s| s.to_string()).collect();
        self
    }

    /// The libraries in cmake's exact order: `"body"` (the Rust archives), `"import:<basename>"`
    /// (an `import_lib`/`delay_import_lib` by its basename) and static libs as for
    /// `static_libs`. In a COFF link the first library on the command line that defines
    /// a symbol wins, and import libs and archives overlap: amd64's ntoskrnl exports
    /// `memcmp`, which a driver's `libmemcmp.a` after the body also defines, and usetup's
    /// `libinflib_rust.a` carries ntkrnl-sys's raw-dylib ntoskrnl import of
    /// NlsMbOemCodePageTag, which cmake's link resolved from ntdll's import lib first.
    /// The module's own objects (c_from_compile_db, asm archives) go first, as a
    /// target's objects do in cmake's link. Replaces static_libs / static_libs_after_body.
    pub fn link_order(mut self, items: &[&str]) -> Self {
        self.link_order = Some(items.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Keep every section, as cmake's kernel-mode driver links do: rustc passes
    /// `--gc-sections` to every bin link, and a KernelDll does not override it
    /// (arm64, with per-function sections, then drops code cmake keeps).
    pub fn keep_all_sections(mut self) -> Self {
        self.keep_all_sections = true;
        self
    }

    /// Generate a PE *import* lib from a `.spec` instead of consuming a
    /// cmake-built one (e.g. HAL importing ntoskrnl). `spec_rel`
    /// repo-relative; `dll_name` the `-n=` arg; `basename` the dlltool
    /// `-t` prefix / the imported DLL's logical name. Self-hosts
    /// spec2def `--implib` then runs `i686-w64-mingw32-dlltool` — no
    /// cmake-built `libNAME.a` needed. Emitted inside the link group.
    /// Chainable — call once per imported DLL (e.g. bootvid imports both
    /// `ntoskrnl` and `hal`, mirroring cmake `add_importlibs(...)`).
    pub fn import_lib(mut self, spec_rel: &str, dll_name: &str, basename: &str) -> Self {
        self.import_libs.push((
            spec_rel.to_string(),
            dll_name.to_string(),
            basename.to_string(),
            false,
        ));
        self
    }

    /// Delay-import `dll_name` instead of importing it: cmake's
    /// `add_delay_importlibs(m <basename>)`. The lib comes from the same
    /// `spec2def --implib` .def as `import_lib`'s, through `sdk/tools/gen-delaylib.py`
    /// (llvm-dlltool has no `--output-delaylib`), exactly as cmake builds
    /// `lib<basename>_delayed.a`. The module still needs the delay-load runtime
    /// (`libdelayimp.a`, a `static_libs` entry) and, once linked, `native-pefixup
    /// --delayimport` to point the DELAY_IMPORT directory at the table lld leaves
    /// unnamed: `add_cargo_module(... DELAY_IMPORTS)` runs it.
    ///
    /// Call it in cmake's order relative to `import_lib`. gen-delaylib puts every thunk
    /// of a DLL in one object, so pulling one drags in all its `__imp_` definitions:
    /// inetcpl delay-imports shell32, which also exports `StrChrW`, and imports shlwapi.
    /// cmake lists the delayed libs first, so `StrChrW` binds to shell32's (delayed) and
    /// shlwapi's member is never pulled; the other way round, both define it and lld
    /// crashes building the import table. Chainable.
    pub fn delay_import_lib(mut self, spec_rel: &str, dll_name: &str, basename: &str) -> Self {
        self.import_libs.push((
            spec_rel.to_string(),
            dll_name.to_string(),
            basename.to_string(),
            true,
        ));
        self
    }

    /// Force-undefined symbols (`-Wl,-u,SYM`) — make the linker keep an
    /// otherwise-unreferenced archive member. Mirrors cmake's
    /// `target_link_options(... LINKER:-u,_HalDispatchTable)`: rosload's i386
    /// HAL dispatch tables (from the Rust `halstub.rs` archive member) are
    /// unreferenced + unexported, so without `-u` `--gc-sections` drops them.
    /// Each symbol is already arch-decorated by the caller. Chainable.
    pub fn force_undefined(mut self, syms: &[&str]) -> Self {
        self.force_undefined
            .extend(syms.iter().map(|s| s.to_string()));
        self
    }

    /// Compile a set of C sources IN-cargo by replaying cmake's own clang
    /// invocation for each (looked up in `build-<arch>/compile_commands.json`),
    /// archiving the objects into `lib<name>.a` linked as an in-group archive
    /// in the C-archive position (before the Rust archives). This replaces a
    /// `static_libs("…/lib<name>.a")` reuse of a cmake-built archive with a
    /// flag-exact cargo-driven compile — dropping the `ninja <name>`
    /// prerequisite for that deep-C residue (e.g. the loaders' blcmlib /
    /// cportlib C bodies).
    ///
    /// `sources` are repo-relative `.c` paths; each MUST appear in
    /// `compile_commands.json`, so the cmake build must be configured with
    /// `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON` (a configure-time flag — no build
    /// needed). The replay strips the ninja-built `.pch` but keeps cmake's
    /// configure-time `cmake_pch.h` force-include, so it is self-contained.
    /// `name` IS the cmake target name (disambiguates a source compiled into
    /// multiple targets via its `CMakeFiles/<name>.dir` object dir). Chainable —
    /// call once per archive, in C-first link order. An empty `sources` adds nothing:
    /// a generated cell passes each arch's list, and acgenral compiles C on amd64 only.
    pub fn c_from_compile_db(mut self, name: &str, sources: &[&str]) -> Self {
        if sources.is_empty() {
            return self;
        }
        self.c_db_libs.push((
            name.to_string(),
            sources.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Mark a `c_from_compile_db` lib whose source list includes the 3 gendib
    /// DIB tables (`dib{8,16,32}gen.c`) — win32k's `libwin32k_residue`. List the
    /// gendib sources by their `build-<arch>/`-relative path (e.g.
    /// `win32ss/gdi/dib/dib8gen.c`): in a cmake build they exist there and the
    /// replay branch compiles them via the generated-source fallback; cmake-off
    /// they don't exist, so the committed branch runs the self-hosted
    /// `native-gendib` (`sdk/tools/gendib/gendib.c`, the spec2def precedent) to
    /// produce them in `OUT_DIR` first — byte-identical to cmake's. Chainable.
    pub fn gendib(mut self, c_db_name: &str) -> Self {
        self.gendib_libs.push(c_db_name.to_string());
        self
    }

    /// Run the self-hosted `native-gendib` → the 3 `dib{8,16,32}gen.c` in
    /// `OUT_DIR/gendib-gen/`; return a basename→path map for source resolution
    /// in the cmake-off (committed) c_db compile. cmake's `native-gendib` takes
    /// the output DIR as its sole argument.
    fn generate_gendib(&self) -> std::collections::HashMap<String, PathBuf> {
        let tool = self.compile_host_tool("sdk/tools/gendib/gendib.c", "native-gendib");
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("gendib-gen");
        std::fs::create_dir_all(&out).expect("mkdir gendib-gen");
        let status = Command::new(&tool)
            .arg(&out)
            .traced_status()
            .unwrap_or_else(|e| panic!("native-gendib: {e}"));
        if !status.success() {
            panic!("native-gendib failed");
        }
        ["dib8gen.c", "dib16gen.c", "dib32gen.c"]
            .iter()
            .map(|n| (n.to_string(), out.join(n)))
            .collect()
    }

    /// Add the kernel asm `#include` search chain to a cc-rs asm build: asm.inc
    /// (source tree), the generated per-arch `ks*.inc` (build dir), and
    /// ntoskrnl/include (sysfuncs.h / asmmacro.S / syscalls.inc deps). With it a
    /// SOURCE `.S` assembles directly via `-x assembler-with-cpp` — no offline
    /// pre-expansion (the old tools/gen_ntdll_asm.py step). Inert for an
    /// already-expanded `.s` (no `#include`s to resolve).
    ///
    /// cmake-off (WS2): the generated `ks*.inc` normally lives in the cmake build
    /// dir. The build-dir copy is added FIRST so it always wins when a cmake build
    /// is configured (the self-host can never make a configured build go stale).
    /// When that copy is ABSENT (the cmake-off path — cargo kernel build with no
    /// cmake configure/ninja), self-host `geninc`: `generate_ks_inc()` compiles
    /// `genincdata.c` and emits the per-arch ks*.inc into OUT_DIR, byte-identical
    /// to cmake's `native-geninc` output. No committed ks*.inc needed.
    fn add_kernel_asm_includes(&self, b: &mut cc::Build) {
        b.include(self.repo_root.join("sdk/include/asm"))
            .include(self.build_dir.join("sdk/include/asm"))
            .include(self.repo_root.join("ntoskrnl/include"));
        let generated = self
            .build_dir
            .join("sdk/include/asm")
            .join(self.ks_inc_name());
        if cmake_off_forced() || !generated.exists() {
            // cmake-off: self-host geninc — compile genincdata.c + emit ks*.inc
            // into OUT_DIR/asm and assemble against that (byte-identical to the
            // cmake-built copy). No committed ks*.inc needed.
            b.include(self.generate_ks_inc());
        }
    }

    /// Determinism (rung 1, cc-rs side): cc-rs compiles every shim/asm/stub with
    /// its ABSOLUTE source path (`repo_root.join(rel)`), which clang embeds into
    /// `__FILE__`/assert strings + debug info. Under the canonical build path
    /// (tools/canon-run.sh) `repo_root` is /tmp/vibeos for every worktree, so this
    /// flag is identical everywhere; it maps to the `/vibeos` token to match
    /// cmake's own C TUs (`-ffile-prefix-map` in the top-level CMakeLists). The
    /// rustc-side remap (det-remap.toml) was removed 2026-07-03 — it made every
    /// Rust compile non-cacheable in sccache; canonical paths replaced it.
    /// See docs/engineering/deterministic-builds-plan.md.
    fn apply_prefix_map(&self, b: &mut cc::Build) {
        b.flag(format!("-ffile-prefix-map={}=/vibeos", self.repo_root.display()));
    }

    /// The per-arch cross clang the asm/C compiles use (matches `link()`).
    fn clang_path(&self) -> PathBuf {
        self.repo_root
            .join(".build-tools/llvm-mingw")
            .join(format!("bin/{}-clang", self.arch.triple))
    }

    /// The per-arch generated kernel-offset include filename (geninc output):
    /// `ks386.inc` (i386), `ksamd64.inc` (amd64), `ksarm64.h` (arm64). Matches
    /// `sdk/include/asm/CMakeLists.txt`'s `_filename`.
    pub(crate) fn ks_inc_name(&self) -> &'static str {
        match self.arch.name {
            "i386" => "ks386.inc",
            "amd64" => "ksamd64.inc",
            "arm64" => "ksarm64.h",
            other => panic!("unknown arch for ks*.inc: {other}"),
        }
    }

    /// Run everything and emit the cargo: link directives.
    pub fn link(self, kind: ModuleKind) {
        let llvm_mingw = self.repo_root.join(".build-tools/llvm-mingw");
        let clang = llvm_mingw.join(format!("bin/{}-clang", self.arch.triple));
        // Output dir stem = cargo's target-name subdir: json file stem for a
        // custom target, or the full triple for a builtin target.
        let target_stem = self.target.stem().to_string();

        // Extra watched dirs (body-crate path-deps in other workspace crates).
        for dir in &self.rust_watch_dirs {
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join(dir).display()
            );
        }

        // Workspace resolution/configuration affects every nested invocation.
        let workspace_root = self.repo_root.join("sdk/lib/rust");
        for input in ["Cargo.toml", "Cargo.lock", ".cargo/config.toml"] {
            println!("cargo:rerun-if-changed={}", workspace_root.join(input).display());
        }

        // Watch every C SHIM too — SAME false-fresh bug as the note above, one input
        // further out. The shims (ntoskrnl/kmsafecopy.c, ntoskrnl/seh_poc.c, the
        // *_rust_shims.c, the CRT printf shells, ...) live OUTSIDE sdk/lib/rust, so
        // neither the workspace sweep above nor cargo's (already-disabled) default
        // whole-package watching covers them. Editing one therefore left cargo
        // reporting the module fresh while relinking a STALE object.
        //
        // Caught 2026-07-12 the expensive way: an edit to kmsafecopy.c's boot
        // self-test never reached the arm64 kernel, and the boot dutifully printed
        // the PREVIOUS build's verdict — a "result" for code that wasn't running.
        // The first arm64 run only picked the shim up because build.rs itself had
        // changed that time, which is exactly what makes this class so easy to miss.
        for rel in &self.c_shims {
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join(rel).display()
            );
        }

        // ---- Rust staticlib(s) (per-crate target dir) ----------------
        // A Win32Dll is USER-mode: its main body crate AND its rust_lib
        // siblings build the STANDARD user-mode target (hardfloat, precompiled
        // std, no build-std) on the default toolchain — exactly like cmake
        // (`ntdll-rust` builds `i686-pc-windows-gnu`, NOT the kernel json).
        // Building a user-mode crate against the `x86-softfloat` kernel json
        // corrupts the by-value f64 ABI and crashes boot. Kernel modules
        // (ntoskrnl/HAL/uefildr) normally build the custom kernel json with `-Z
        // build-std` on nightly. A kernel-ish module may also opt into a builtin
        // target triple; that path uses stable/precompiled std like cmake. The
        // main crate goes first, then siblings.
        // `.rust_user_target()` (the freeldr loaders) forces the user-target
        // path even for an EfiApplication/NativeExe — freeldr-rust is built
        // standard-target + `-sse,-sse2` exactly as cmake's custom_command did.
        // Win32Dll/Win32Exe (ntdll, lsass, userinit, …) are user-mode too.
        let kernel_rust = !matches!(kind, ModuleKind::Win32Dll { .. } | ModuleKind::Win32Exe { .. })
            && !self.rust_user_target;
        let rust_a = self.rust_crate.as_ref().map(|crate_name| {
            let feats: Vec<&str> = self.rust_features.iter().map(|s| s.as_str()).collect();
            self.build_rust_crate(
                crate_name,
                &feats,
                true,
                &target_stem,
                kernel_rust,
                self.rust_build_rustflags.as_deref(),
            )
        });
        let sibling_rust_a: Vec<PathBuf> = self
            .rust_libs
            .iter()
            .map(|(crate_name, feats, default_features)| {
                let f: Vec<&str> = feats.iter().map(|s| s.as_str()).collect();
                self.build_rust_crate(
                    crate_name,
                    &f,
                    *default_features,
                    &target_stem,
                    kernel_rust,
                    None,
                )
            })
            .collect();

        // ---- C shims (cc-rs) -----------------------------------------
        for rel in &self.c_shims {
            let abs = self.repo_root.join(rel);
            if !abs.exists() {
                panic!("missing C shim: {}", abs.display());
            }
            let mut b = cc::Build::new();
            apply_base(&mut b, &clang, AsmDefineSet::Kernel, &self.arch);
            self.apply_prefix_map(&mut b);
            b.file(&abs).out_dir(self.obj_dir()).flag("-std=gnu2x");
            for f in &self.c_shim_flags {
                b.flag(f);
            }
            self.emit_objs(&b, &abs);
            println!("cargo:rerun-if-changed={}", abs.display());
        }

        // ---- Pre-expanded asm (cc-rs, no kernel -I) ------------------
        for rel in &self.asm {
            let abs = self.repo_root.join(rel);
            if !abs.exists() {
                panic!(
                    "missing pre-processed asm: {} — regenerate via the matching tools/gen_preprocessed_asm*.py",
                    abs.display()
                );
            }
            let mut b = cc::Build::new();
            apply_base(&mut b, &clang, self.asm_set, &self.arch);
            self.apply_prefix_map(&mut b);
            // Caller-layered extra defines (e.g. ntkrnlmp's CONFIG_SMP) on top
            // of the asm_set list. Inert when empty (every non-ntkrnlmp module).
            for (n, v) in &self.asm_extra_defines {
                b.define(n, v.as_deref());
            }
            b.file(&abs)
                .out_dir(self.obj_dir())
                .define("__ASM__", None)
                .flag("-nostdinc")
                .flag("-isystem")
                .flag(self.repo_root.join("sdk/include/crt").to_str().unwrap())
                .flag("-isystem")
                .flag(llvm_mingw.join("lib/clang/19/include").to_str().unwrap());
            // Caller-configured repo-relative `-I` dirs for RAW `.S` files
            // (asm.inc, pcbios.h, …; a pre-expanded `.s` sets none of these).
            for dir in &self.asm_include_dirs {
                b.flag("-I")
                    .flag(self.repo_root.join(dir).to_str().unwrap());
            }
            // + the fixed kernel asm `#include` chain (asm.inc + the generated
            // build-dir ks*.inc + ntoskrnl/include) used by modules that
            // assemble kernel source `.S` directly (ntdll dispatch.S / ntdll.S).
            // Additive — extra `-I` is inert when unused (and the build-dir dir
            // is not expressible via the repo-relative asm_include_dirs above).
            self.add_kernel_asm_includes(&mut b);
            b.flag("-x").flag("assembler-with-cpp");
            self.emit_objs(&b, &abs);
            println!("cargo:rerun-if-changed={}", abs.display());
        }

        // ---- Resource (.res) compiled IN-cargo from the .rc -----------
        // Replay cmake's flag-exact `windres` command (positional in/out, no
        // `-o`) with the output redirected into OUT_DIR, then link the COFF
        // `.res` object. Drops the `ninja <m>_res` prereq.
        if let Some((rc_rel, res_target)) = &self.resource_from_rc {
            let db_path = self.build_dir.join("compile_commands.json");
            let dst = self.obj_dir().join("module.rc.res");
            if !cmake_off_forced() && db_path.exists() {
                // cmake build present: replay cmake's flag-exact windres.
                let entries = ccdb_entries(&db_path);
                let abs = self.repo_root.join(rc_rel);
                let marker = format!("CMakeFiles/{res_target}.dir");
                let (dir, cmd) = lookup_cc(&entries, &abs, &marker).unwrap_or_else(|| {
                    panic!(
                        "no compile_commands.json windres entry for {} in target {res_target} \
                         (marker {marker}) — is {res_target} a cmake OBJECT lib over that .rc?",
                        abs.display()
                    )
                });
                // windres: `... <input.rc> <output.res>` — the output is the LAST
                // positional token; replace it with our OUT_DIR path, keep all else.
                let mut args: Vec<String> =
                    cmd.split_whitespace().map(|s| s.to_string()).collect();
                *args.last_mut().expect("empty windres command") =
                    dst.to_str().expect("utf8 res path").to_string();
                let status = Command::new(&args[0])
                    .args(&args[1..])
                    .current_dir(&dir)
                    .traced_status()
                    .unwrap_or_else(|e| panic!("replay windres for {rc_rel}: {e}"));
                if !status.success() {
                    panic!("resource_from_rc: windres failed for {rc_rel} (target {res_target})");
                }
                println!("cargo:rerun-if-changed={}", abs.display());
                println!("cargo:rerun-if-changed={}", db_path.display());
            } else {
                // cmake-off (WS3): no build dir → use the committed byte-stable .res
                // (generated by replaying this exact windres command; see
                // tools/gen_committed_res.py). Convention:
                // <consumer crate>/data/<rc-stem>-<arch>.rc.res. The cmake path
                // above is preferred whenever a build dir exists, so the committed
                // copy can never make a configured build go stale.
                let stem = Path::new(rc_rel)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_else(|| panic!("bad rc path {rc_rel}"));
                let committed = self
                    .manifest_dir
                    .join(format!("data/{stem}-{}.rc.res", self.arch.name));
                if !committed.exists() {
                    panic!(
                        "cmake-off: resource_from_rc({rc_rel}) needs committed {} \
                         (regenerate via tools/gen_committed_res.py) or a configured build dir",
                        committed.display()
                    );
                }
                std::fs::copy(&committed, &dst)
                    .unwrap_or_else(|e| panic!("stage committed .res {}: {e}", committed.display()));
                println!("cargo:rerun-if-changed={}", committed.display());
            }
            link_arg!(self, "{}", dst.display());
        }

        if let Some(spec) = &self.rc {
            let res = self.compile_rc(spec);
            link_arg!(self, "{}", res.display());
        }

        // ---- Resource (.res) — checked-in or cmake-built -------------
        let res_src: Option<PathBuf> = self
            .resource
            .as_ref()
            .map(|r| self.manifest_dir.join(r))
            .or_else(|| self.resource_built.as_ref().map(|r| self.build_dir.join(r)));
        if let Some(src) = res_src {
            if !src.exists() {
                panic!(
                    "missing {} — regenerate via tools/gen_*_res.py (checked-in) \
                     or `ninja` the module (cmake-built .res)",
                    src.display()
                );
            }
            let dst = self.obj_dir().join("module.rc.res");
            std::fs::copy(&src, &dst).unwrap_or_else(|e| panic!("stage .res: {e}"));
            link_arg!(self, "{}", dst.display());
            println!("cargo:rerun-if-changed={}", src.display());
        }

        // ---- spec2def → .def (+ optional -stub bodies) ---------------
        if let Some((spec_rel, dll_name)) = &self.spec {
            let (def, stubs) = self.generate_def(spec_rel, dll_name);
            link_arg!(self, "{}", def.display());
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root.join(spec_rel).display()
            );
            println!(
                "cargo:rerun-if-changed={}",
                self.repo_root
                    .join("sdk/tools/spec2def/spec2def.c")
                    .display()
            );
            if self.link_stubs {
                if !stubs.exists() {
                    panic!(
                        "spec_stubs() set but spec2def produced no stubs at {}",
                        stubs.display()
                    );
                }
                let mut b = cc::Build::new();
                apply_base(&mut b, &clang, AsmDefineSet::Kernel, &self.arch);
                self.apply_prefix_map(&mut b);
                // module_stubs.c only `#include <stubs.h>` (self-contained).
                b.file(&stubs)
                    .out_dir(self.obj_dir())
                    .include(self.repo_root.join("sdk/include/reactos"))
                    .flag("-std=gnu2x");
                self.emit_objs(&b, &stubs);
            }
        }

        // ---- Static-lib group ----------------------------------------
        // ORDER MATTERS: emit the cmake C archives FIRST, then the Rust
        // archives (main staticlib + rust_lib siblings) LAST — exactly as
        // cmake does for every module (verified on ntdll.dll AND ntoskrnl.exe:
        // librtl.a / liblibcntpr.a precede libkernel_rust.a / librtl_rust.a /
        // libcntpr_rust.a). The cnt CORE mem/string ops (memcpy/memset/strcmp/
        // strncpy/wcs*/… ~25 on a CLEAN archive) are defined in BOTH the C
        // liblibcntpr.a (i386 asm) AND cnt-rust (cohabitation, C not yet
        // REMOVE_ITEM'd). Under --allow-multiple-definition lld takes the FIRST
        // by command-line order, so C-first selects the vetted i386-asm variant
        // cmake ships; Rust-first selected not-yet-fully-vetted Rust twins and
        // bugchecked at boot. See feedback_cargo_drive_usermode_target.md.
        link_arg!(self, "-Wl,--start-group");
        let ar = llvm_mingw.join(format!("bin/{}-ar", self.arch.triple));
        if let Some(order) = &self.link_order {
            assert!(
                self.static_libs.is_empty() && self.late_static_libs.is_empty(),
                "link_order replaces static_libs/static_libs_after_body"
            );
            self.emit_own_archives(&clang, &ar);
            for item in order {
                if item == "body" {
                    for a in rust_a.iter().chain(sibling_rust_a.iter()) {
                        link_arg!(self, "{}", a.display());
                        println!("cargo:rerun-if-changed={}", a.display());
                    }
                } else if let Some(base) = item.strip_prefix("import:") {
                    let lib = self.import_libs.iter().find(|l| l.2 == base);
                    let lib = lib.unwrap_or_else(|| panic!("link_order names import:{base}, which no import_lib declares"));
                    self.emit_import_lib(lib);
                } else {
                    let full = self.static_lib_path(item, &clang, &ar);
                    link_arg!(self, "{}", full.display());
                    println!("cargo:rerun-if-changed={}", full.display());
                }
            }
            for lib in &self.import_libs {
                assert!(
                    order.iter().any(|i| i.strip_prefix("import:") == Some(lib.2.as_str())),
                    "link_order leaves out import:{}",
                    lib.2
                );
            }
        } else {
        for lib in &self.static_libs {
            let full = self.static_lib_path(lib, &clang, &ar);
            link_arg!(self, "{}", full.display());
            // rerun-if-changed on each input .a: a rustc-link-arg is NOT a
            // tracked input, so without this cargo silently reuses a stale
            // binary when an input archive changes (e.g. a rebuilt
            // libcntpr_rust.a with a CRT miscompile fix) — the linked module
            // then boots the OLD code. Cost a debug session: a cargo uefildr
            // kept booting a pre-fix _strnicmp despite the .a being rebuilt.
            println!("cargo:rerun-if-changed={}", full.display());
        }
        self.emit_own_archives(&clang, &ar);
        if let Some(a) = &rust_a {
            link_arg!(self, "{}", a.display());
            println!("cargo:rerun-if-changed={}", a.display());
        }
        for a in &sibling_rust_a {
            link_arg!(self, "{}", a.display());
            println!("cargo:rerun-if-changed={}", a.display());
        }
        for lib in &self.late_static_libs {
            let full = self.static_lib_path(lib, &clang, &ar);
            link_arg!(self, "{}", full.display());
            println!("cargo:rerun-if-changed={}", full.display());
        }
        for lib in &self.import_libs {
            self.emit_import_lib(lib);
        }
        } // end else (no link_order)
        // memcmp FALLBACK, builtin targets only (stable-toolchain Axis B).
        //
        // The PRECOMPILED `core` calls `memcmp` (core::slice::cmp, reached from
        // e.g. `str` pattern search). `-Z build-std` used to supply it via
        // `-Z build-std-features=compiler-builtins-mem`, and every cell that moved
        // to build-std DROPPED its libmemcmp.a for exactly that reason. Dropping
        // build-std brings the need back.
        //
        // Most mem* are fine: ntoskrnl.spec exports memcpy/memset/memmove on every
        // arch — but `memcmp` ONLY on x86_64 (`@ cdecl -arch=x86_64 memcmp()`). So
        // amd64 kernel cells resolve it from the ntoskrnl import lib while arm64
        // ones have nothing, which is what `sdk/lib/crt/libmemcmp.a` is for.
        //
        // Emitted LAST — after the static libs, the Rust archives and the import
        // libs — so it is a pure FALLBACK: lld only pulls an archive member for a
        // still-undefined symbol, so any cell that already defines memcmp (libcntpr
        // in ntoskrnl, win32k's Rust memcmp, amd64's ntoskrnl import) keeps its own
        // and this contributes nothing. Only a cell with a genuine gap pulls it.
        if !self.target.is_json() {
            let mut memcmp_a = self.build_dir.join("sdk/lib/crt/libmemcmp.a");
            if cmake_off_forced() || !memcmp_a.exists() {
                // No cmake build: compile it from its recipe, as `engine:c:memcmp` does.
                memcmp_a = self.compile_recipe("memcmp", &ar);
            }
            if !memcmp_a.exists() {
                panic!(
                    "missing {} — run ninja memcmp first (needed by the precompiled \
                     core on the builtin triples)",
                    memcmp_a.display()
                );
            }
            link_arg!(self, "{}", memcmp_a.display());
            println!("cargo:rerun-if-changed={}", memcmp_a.display());
        }
        link_arg!(self, "-Wl,--end-group");

        let clang_rt = llvm_mingw.join(format!(
            "lib/clang/19/lib/windows/libclang_rt.builtins-{}.a",
            self.arch.clang_rt
        ));
        link_arg!(self, "{}", clang_rt.display());

        // ---- BUILTIN-TARGET CRT NEUTRALISATION (stable-toolchain Axis B) ----
        // On a builtin triple the target spec carries the user-mode mingw CRT
        // plumbing our old custom kernel json existed to strip:
        //   late-link-args:  -lmsvcrt -lmingwex -lmingw32 -lgcc -luser32 -lkernel32
        //   late-link-args-static: -lgcc_eh -l:libpthread.a   (+ -lgcc_s dynamic)
        //   crt-objects-fallback: mingw   (crt2.o / dllcrt2.o, rsbegin.o / rsend.o)
        // rustc appends the `-l`s AFTER our archives and there is no flag to
        // suppress them, so we shadow every one with an EMPTY archive on a `-L`
        // that precedes llvm-mingw's own lib dir. `-nostdlib` (below) is NOT
        // enough — it only stops clang adding its defaults, not these explicit -l.
        //
        // This is deliberately a HARD WALL, not a convenience: an empty libmsvcrt.a
        // means a symbol we fail to provide (e.g. memcpy, which used to come from
        // build-std's `compiler-builtins-mem`) is a LOUD undefined-symbol link
        // error naming it — instead of being silently satisfied by the user-mode
        // CRT, which would graft an msvcrt.dll IMPORT onto a kernel driver and
        // fail far away at boot. Kernel modules get mem* from the ntoskrnl import
        // lib (ntoskrnl.spec exports memcpy/memset/memmove/memcmp).
        //
        // crt startup OBJECTS (crt2.o/dllcrt2.o) are a separate problem — an archive
        // stub cannot shadow an object — killed by `-C link-self-contained=no` in
        // add_cargo_module's RUSTFLAGS. amd64 only: aarch64-pc-windows-gnullvm
        // rejects that flag and links no CRT objects to begin with.
        if !self.target.is_json() {
            let stub_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("crt-stubs");
            std::fs::create_dir_all(&stub_dir).expect("crt-stubs dir");
            for lib in [
                "msvcrt", "mingwex", "mingw32", "gcc", "gcc_eh", "gcc_s", "pthread", "user32",
                "kernel32",
            ] {
                let a = stub_dir.join(format!("lib{lib}.a"));
                // `llvm-ar rcs <a>` with no members → a valid, empty archive.
                let st = Command::new(&ar)
                    .arg("rcs")
                    .arg(&a)
                    .traced_status()
                    .unwrap_or_else(|e| panic!("crt-stub: cannot run {}: {e}", ar.display()));
                assert!(st.success(), "crt-stub: {} rcs {} failed", ar.display(), a.display());
            }
            // -L must precede llvm-mingw's sysroot lib dir so OUR empty libmsvcrt.a
            // wins the search. rustc emits link-search paths before the late args.
            println!("cargo:rustc-link-search=native={}", stub_dir.display());
            // i686-pc-windows-gnu ALSO links `rsbegin.o`/`rsend.o` from the sysroot
            // (not a self-contained object, so `link-self-contained=no` keeps them).
            // rsbegin's `.ctors` entry registers the image's DWARF `.eh_frame` with
            // libgcc's unwinder: `__register_frame_info` / `__deregister_frame_info`.
            // Our cells are panic=abort and link no libgcc (its stub is empty, above),
            // so nothing ever unwinds and there is no unwinder to register with. Give
            // the two calls no-op bodies; an undefined-symbol error is the alternative.
            if self.arch.name == "i386" {
                let src = stub_dir.join("eh_frame_register.c");
                std::fs::write(
                    &src,
                    "/* reactos-link: rsbegin.o's EH-frame registration, for a panic=abort\n\
                     * image with no unwinder (see lib.rs). */\n\
                     void __register_frame_info(const void *b, void *o) { (void)b; (void)o; }\n\
                     void __deregister_frame_info(const void *b) { (void)b; }\n",
                )
                .expect("write eh_frame_register.c");
                let obj = stub_dir.join("eh_frame_register.o");
                let clang = llvm_mingw.join(format!("bin/{}-clang", self.arch.triple));
                let st = Command::new(&clang)
                    .args(["-O2", "-c", "-ffreestanding", "-fno-asynchronous-unwind-tables"])
                    .arg(&src)
                    .arg("-o")
                    .arg(&obj)
                    .traced_status()
                    .unwrap_or_else(|e| panic!("eh_frame_register: {}: {e}", clang.display()));
                assert!(st.success(), "eh_frame_register: compile failed");
                link_arg!(self, "{}", obj.display());
                // rsbegin/rsend's `.ctors`/`.dtors` entry each become a 4-byte section
                // (lld's MinGW writer makes those sections itself, so no /MERGE reaches
                // them). crt-startup-rust walks __CTOR_LIST__ and calls the no-op above.
                //
                // The builtin spec's pre-link args also set --large-address-aware, which
                // cmake's link does not: a 32-bit module must not claim >2 GB pointers
                // on our say-so. The MinGW driver has no negative form; COFF's has.
                link_arg!(self, "-Wl,-Xlink=-largeaddressaware:no");
            }
        }

        // ---- PE link flags (common to every module kind) -------------
        let ld_path = format!(
            "--ld-path={}",
            self.repo_root.join("tools/link_guard/ld.lld").display()
        );
        for f in [
            "-nostdlib",
            "-fuse-ld=lld",
            // lld through tools/link_guard.py, which fails the link on any
            // duplicate symbol that --allow-multiple-definition (below) would
            // otherwise resolve silently by link order. Same policy as gcc.cmake.
            ld_path.as_str(),
            "-Wl,--enable-auto-image-base,--disable-auto-import",
            "-Wl,--major-image-version,5",
            "-Wl,--minor-image-version,01",
            "-Wl,--major-os-version,5",
            "-Wl,--minor-os-version,01",
            "-Wl,--allow-multiple-definition",
            // Reproducible builds (deterministic-builds-plan.md rung 2): the gnu
            // ld.lld driver defaults to --insert-timestamp, stamping the PE
            // header with wall-clock time → every rebuild diverges, defeating
            // cross-worktree dedup. Zero it so the kernel/DLL byte-matches across
            // builds. Verify with `vibeos-cli repro report <pe>` (→ 0).
            "-Wl,--no-insert-timestamp",
        ] {
            link_arg!(self, "{f}");
        }
        // Ship/disk strip (release-strip opt-out for `[profile.release] debug=1`):
        // debug=1 (arm64 line-table symbolication) embeds DWARF in every linked PE
        // → fat artifacts that defeat cross-worktree dedup. VIBEOS_STRIP_RELEASE=1
        // (set by ship/release/disk-GC builds) drops the debug sections at link
        // time; dev/smoke leave them in for gdb symbolication. No-op when debug=0
        // (nothing to strip) — so this is inert on the current default build.
        // The EFI/native loaders below strip UNCONDITIONALLY (they don't use in-image
        // DWARF — symbolicated via qemu/sym.sh offset math).
        // rerun-if-env-changed so toggling the flag on an already-built tree relinks
        // (without it cargo wouldn't rerun this build script → silent no-op strip).
        println!("cargo:rerun-if-env-changed=VIBEOS_STRIP_RELEASE");
        if env::var_os("VIBEOS_STRIP_RELEASE").is_some() {
            link_arg!(self, "-Wl,--strip-debug");
        }
        // Layout ground truth (captures(none)/layout-tip debugging): env-gated
        // lld map file — records every archive member pulled and which twin of
        // a duplicated symbol won first-match under --allow-multiple-definition.
        // Diff two maps to see exactly what a link-layout perturbation changed
        // (tools/scan_captures.py is the compile-time half of the same audit).
        println!("cargo:rerun-if-env-changed=VIBEOS_LINK_MAP");
        if env::var("VIBEOS_LINK_MAP").as_deref() == Ok("1") {
            link_arg!(
                self,
                "-Wl,-Map,{}",
                self.build_dir
                    .join(format!("{}.map", self.pkg_name))
                    .display()
            );
        }
        // By default disable lld's stdcall-fixup (a decorated-vs-undecorated
        // symbol mismatch is a real ABI bug, not something to paper over). But
        // winsrv.dll's cmake link relies on the fixup to resolve winsrv-rust's
        // cross-module extern "C"-def vs extern "system"-call mismatches
        // (e.g. HistoryReshapeAllBuffers) — those are a pre-existing
        // translation-quality issue; allow_stdcall_fixup() reproduces cmake's
        // behavior (which passes neither flag → lld default = fixup-with-warning)
        // so the cargo link is byte-equivalent to the working cmake one.
        if !self.allow_stdcall_fixup {
            link_arg!(self, "-Wl,--disable-stdcall-fixup");
        }
        // Custom stack reserve (default 1MB otherwise) — e.g. arm64 csrss.exe's
        // deep LdrpLoadDll chain.
        if let Some(bytes) = self.stack_reserve {
            link_arg!(self, "-Wl,--stack,{bytes}");
        }
        // ---- kind-specific subsystem / alignment / sections ----------
        let (entry, image_base): (&String, Option<u64>) = match &kind {
            ModuleKind::KernelExe { entry, image_base } => {
                // native subsystem; GC unreferenced sections (main.rs anchors
                // the entry to hold the root); no -shared.
                link_arg!(self, "-Wl,--subsystem,native:5.01");
                link_arg!(self, "-Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000");
                link_arg!(self, "-Wl,--gc-sections");
                (entry, Some(*image_base))
            }
            ModuleKind::KernelDll { entry, image_base } => {
                // native subsystem PE DLL, 0x1000 align. rustc's own `--gc-sections`
                // applies unless `keep_all_sections()`, which cmake's links match:
                // the hand-written kernel DLL cells (hal, kdcom, bootvid, the
                // drivers cut before the generator) still gc.
                link_arg!(self, "-Wl,--subsystem,native:5.01");
                link_arg!(self, "-Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000");
                link_arg!(self, "-shared");
                if self.keep_all_sections {
                    // After rustc's --gc-sections, so it wins (the last flag does).
                    link_arg!(self, "-Wl,--no-gc-sections");
                }
                (entry, Some(*image_base))
            }
            ModuleKind::Win32Dll {
                entry,
                image_base,
                subsystem,
            } => {
                // user-mode PE DLL: exports come from the .def so NO
                // --exclude-all-symbols, default PE alignment (file 0x200),
                // -shared, keep all sections. Subsystem is explicit for
                // ntdll.dll (console:5.01) and lld-default (GUI 6.0) for
                // ntdll_vista.dll, which passes no --subsystem.
                if let Some(s) = subsystem {
                    link_arg!(self, "-Wl,--subsystem,{s}");
                }
                link_arg!(self, "-shared");
                // KEEP ALL SECTIONS — match cmake's user-mode DLL link, which
                // does NOT gc-sections ntdll. rustc adds `--gc-sections` by
                // default for bin targets; without this override it strips
                // ~196KB of arm64 ntdll code (incl. the CsrPortHeap init path),
                // leaving CsrAllocateCaptureBuffer reading a null heap →
                // BUGCHECK 0x6A in smss init. i386/amd64 tolerated the strip;
                // arm64 did not. --no-gc-sections must come AFTER rustc's own
                // --gc-sections so it wins (last flag wins in lld).
                link_arg!(self, "-Wl,--no-gc-sections");
                (entry, *image_base)
            }
            ModuleKind::EfiApplication {
                entry,
                image_base,
                alignment,
            } => {
                // UEFI app (uefildr.efi): efi_application subsystem,
                // per-arch file/section alignment (x86 0x200, arm64 0x1000),
                // --gc-sections, no -shared (exe), --exclude-all-symbols
                // (exports nothing). The common flags above already add
                // --allow-multiple-definition (the cargo bin's own
                // #[panic_handler] in main.rs collides with
                // libfreeldr_rust_uefi.a + cnt-rust CRT dups; lld first-match
                // wins) and --enable-auto-image-base. cmake's uefildr omits
                // the --major-*-version flags the common block adds, but those
                // are harmless for an EFI app the firmware loads by FAT path.
                link_arg!(self, "-Wl,--subsystem,efi_application:5.01");
                link_arg!(
                    self,
                    "-Wl,--exclude-all-symbols,--file-alignment,0x{a:x},--section-alignment,0x{a:x}",
                    a = alignment
                );
                link_arg!(self, "-Wl,--gc-sections");
                // Strip DWARF from uefildr unconditionally. With `[profile.release]
                // debug=1` the cargo bin embeds line tables, but the loader is
                // symbolicated via offset math in qemu/sym.sh (not in-image DWARF),
                // so the debug sections are pure bloat in a firmware-loaded image —
                // always drop them. (An earlier theory that EDK2 StartImage *rejects*
                // a DWARF-fat EFI as "Unsupported" was disproven — that line is
                // firmware noise; the real arm64 break was a corrupted efivars.fd.
                // So this is leanness, not a boot requirement.) No-op at debug=0.
                link_arg!(self, "-Wl,--strip-debug");
                (entry, *image_base)
            }
            ModuleKind::NativeExe { entry, image_base } => {
                // rosload.exe: native subsystem exe that EXPORTS via its .def
                // (so NO --exclude-all-symbols, unlike KernelExe), --gc-sections,
                // no -shared, DEFAULT PE alignment (cmake passes no
                // --file/-section-alignment → lld default file 0x200/section
                // 0x1000). exports flow from .spec2def()'s rosload.def.
                link_arg!(self, "-Wl,--subsystem,native:5.01");
                link_arg!(self, "-Wl,--gc-sections");
                // Strip DWARF unconditionally (same as EfiApplication above: rosload
                // is symbolicated via offset math, not in-image debug info, so debug=1
                // line tables are dead weight here). No-op at debug=0.
                link_arg!(self, "-Wl,--strip-debug");
                (entry, Some(*image_base))
            }
            ModuleKind::BiosLoader { entry, image_base } => {
                link_arg!(self, "-Wl,--subsystem,native:5.01");
                link_arg!(self, "-Wl,--file-alignment,0x200,--section-alignment,0x200");
                link_arg!(self, "-Wl,--gc-sections");
                // Same as EfiApplication/NativeExe: symbolicated by offset math,
                // not in-image DWARF. The cmake link also ran `strip --strip-all`.
                link_arg!(self, "-Wl,--strip-debug");
                (entry, Some(*image_base))
            }
            ModuleKind::Win32Exe { entry, subsystem } => {
                // user-mode win32 exe (lsass.exe, set_module_type win32gui):
                // explicit subsystem, --gc-sections, no -shared, no
                // --exclude-all-symbols (nothing exported), no explicit
                // --image-base (common --enable-auto-image-base resolves it,
                // matching cmake). Entry = the CRT startup in libmsvcrtex.a.
                link_arg!(self, "-Wl,--subsystem,{subsystem}");
                link_arg!(self, "-Wl,--gc-sections");
                (entry, None)
            }
        };
        // Force-undefined pulls (-u SYM) — keep GC'd-but-needed archive members.
        for sym in &self.force_undefined {
            link_arg!(self, "-Wl,-u,{sym}");
        }
        for a in &self.extra_link_args {
            link_arg!(self, "{a}");
        }
        link_arg!(self, "-Wl,-e,{entry}");
        // Emit an explicit base only when the module pins one; otherwise the
        // common --enable-auto-image-base resolves it (matches cmake's arm64
        // uefildr, which passes no --image-base).
        if let Some(b) = image_base {
            link_arg!(self, "-Wl,--image-base,0x{:08x}", b);
        }
    }

    /// Build one workspace Rust staticlib crate (`crate-type=["staticlib"]`),
    /// returning a consumer-owned archive when sharing is enabled, or the
    /// private target's archive in legacy mode. `default_features=false` →
    /// `--no-default-features`.
    ///
    /// `kernel=true` (the main module body, e.g. ntoskrnl-rust / ntdll-rust):
    /// builds the custom kernel json with `-Z build-std` on nightly.
    /// `kernel=false` (Tier-B user-mode sibling, e.g. rtl-rust / cnt-rust-lib):
    /// builds the STANDARD user-mode target (`arch.rust_user_target`,
    /// hardfloat, precompiled std) on the default toolchain, NO build-std —
    /// exactly as cmake builds these. Using the kernel json for a user-mode
    /// sibling corrupts cnt-rust-lib's by-value f64 ABI (`x86-softfloat`) and
    /// crashes i386 boot.
    ///
    /// Separate Cargo invocations preserve feature selection. Shared body storage
    /// holds a shard lock until the archive is copied into this consumer's OUT_DIR;
    /// the outer linker never consumes a mutable shared primary archive.
    fn build_rust_crate(
        &self,
        crate_name: &str,
        features: &[&str],
        default_features: bool,
        target_stem: &str,
        kernel: bool,
        rustflags: Option<&str>,
    ) -> PathBuf {
        self.build_rust_crate_as(crate_name, features, default_features, target_stem, kernel, rustflags, false)
    }

    /// `build_rust_crate`, and with `staticlib` the crate is built as one even though
    /// its manifest only says rlib (`cargo rustc --crate-type staticlib`), as cmake
    /// builds concfg-rust's standalone archive.
    #[allow(clippy::too_many_arguments)]
    fn build_rust_crate_as(
        &self,
        crate_name: &str,
        features: &[&str],
        default_features: bool,
        target_stem: &str,
        kernel: bool,
        rustflags: Option<&str>,
        staticlib: bool,
    ) -> PathBuf {
        let key = format!(
            "{crate_name}|{features:?}|{default_features}|{target_stem}|{kernel}|{rustflags:?}|{staticlib}"
        );
        memo(&RUST_LIBS, key, || {
            self.build_rust_crate_once(
                crate_name,
                features,
                default_features,
                target_stem,
                kernel,
                rustflags,
                staticlib,
            )
        })
    }

    fn build_rust_crate_once(
        &self,
        crate_name: &str,
        features: &[&str],
        default_features: bool,
        target_stem: &str,
        kernel: bool,
        rustflags: Option<&str>,
        staticlib: bool,
    ) -> PathBuf {
        let lib_file = format!("lib{}.a", crate_name.replace('-', "_"));
        let workspace = self.repo_root.join("sdk/lib/rust");
        // Track the crate's own sources so an edit re-runs THIS build.rs, which
        // re-invokes the inner `cargo build -p <crate>` below. Without it cargo
        // may skip build.rs after a body-crate edit and relink a STALE .a (the
        // libcntpr.cmake staleness class). rerun-if-changed on a dir recurses.
        let crate_dir = workspace.join(crate_name);
        println!("cargo:rerun-if-env-changed=VIBEOS_EMIT_LLVM_IR");
        println!("cargo:rerun-if-changed={}", crate_dir.join("src").display());
        println!(
            "cargo:rerun-if-changed={}",
            crate_dir.join("Cargo.toml").display()
        );
        println!("cargo:rerun-if-env-changed=VIBEOS_RUST_STORAGE_PILOT");
        let storage_pilot =
            storage::pilot_enabled(env::var("VIBEOS_RUST_STORAGE_PILOT").ok().as_deref());
        // Only three distinct main-body staticlibs, with identical codegen settings.
        // Cargo locks this inner directory; the outer cargo uses another directory,
        // so it never holds the lock its build script needs. No multi-package build
        // (feature unification), shared cross-worktree path, or same-name variant.
        let shared_pilot = storage_pilot
            && storage::pilot_member(&self.pkg_name, crate_name)
            && self.arch.name == "amd64"
            && !self.target.is_json()
            && target_stem == "x86_64-pc-windows-gnu"
            && kernel
            && default_features
            && features.is_empty()
            && rustflags.is_none()
            && env::var("VIBEOS_EMIT_LLVM_IR").as_deref() != Ok("1");
        let tgt_dir = workspace.join(if shared_pilot {
            "target/storage-pilot-v1-amd64-rustlib".to_string()
        } else {
            format!(
                "target/{}-{}-rustlib",
                self.pkg_name,
                crate_name.replace('-', "_")
            )
        });
        // Kernel json targets: the env CARGO (the pinned nightly, set by the outer
        // cargo) drives build-std. Builtin kernel targets and user-mode siblings run
        // the plain `cargo` rustup proxy, matching cmake's precompiled-std path — the
        // env scrub below strips the outer nightly's RUSTUP_TOOLCHAIN/RUSTC, so the
        // proxy resolves the PINNED STABLE from the repo-root rust-toolchain.toml.
        // (It used to say `rustup run stable`, which named the FLOATING `stable`
        // channel and so ignored the pin — a `rustup update` silently swapped the
        // compiler under the user-mode siblings.)
        let kernel_build_std = kernel && self.target.is_json();
        let mut cmd = if kernel_build_std {
            Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        } else {
            Command::new("cargo")
        };
        cmd.arg(if staticlib { "rustc" } else { "build" })
            .arg("--release")
            .arg("--target");
        // out_stem = the subdir cargo writes under.
        let out_stem: String = if kernel {
            cmd.arg(self.target.arg());
            if kernel_build_std {
                cmd.arg("-Z")
                    .arg("build-std=core,alloc,compiler_builtins")
                    .arg("-Z")
                    .arg("build-std-features=compiler-builtins-mem");
            }
            target_stem.to_string()
        } else {
            cmd.arg(self.arch.rust_user_target);
            self.arch.rust_user_target.to_string()
        };
        cmd.arg("-p").arg(crate_name);
        if staticlib {
            cmd.arg("--crate-type").arg("staticlib");
        }
        if !default_features {
            cmd.arg("--no-default-features");
        }
        if !features.is_empty() {
            cmd.arg("--features").arg(features.join(","));
        }
        cmd.arg("--manifest-path")
            .arg(workspace.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", &tgt_dir);
        if kernel {
            if kernel_build_std {
                // build-std needs nightly — the PINNED one (`.rust-nightly`), never a
                // floating `nightly`: upstream has already removed the c_variadic API
                // this tree is written against, so an unpinned `rustup update` breaks
                // the build. Bump the pin there, not here.
                cmd.env("RUSTUP_TOOLCHAIN", pinned_nightly(&self.repo_root));
            } else {
                // The outer cargo may still be nightly/build-std. Scrub its env
                // so the stable builtin-target cargo behaves like cmake's fresh
                // cargo process.
                compiler_env::scrub_outer_compiler(&mut cmd);
            }
            // The CRT-class kernel siblings (rtl-rust + cnt-rust-lib) need the
            // EXACT per-crate RUSTFLAGS cmake gives them, or LLVM's default
            // pre-pass pipeline miscompiles them (the captures(none)/DCE class
            // `-C no-prepopulate-passes` guards — e.g. memset DCE'd to a spin,
            // captures(none) eliding output writes). Without it rtl-rust built
            // 1.3MB instead of cmake's 6.1MB (over-optimized) and hung the kernel
            // right after ACPI init. Mirror cmake EXACTLY:
            //   - rtl-rust    (sdk/lib/rtl/CMakeLists.txt): i386 →
            //       `-sse,-sse2 -C no-prepopulate-passes`; amd64/arm64 → none.
            //   - cnt-rust-lib (sdk/lib/crt/libcntpr.cmake): i386 →
            //       `-sse,-sse2 -C no-prepopulate-passes`; amd64/arm64 →
            //       `-C no-prepopulate-passes`.
            // The kernel json is x86-softfloat (ABI only); i386 codegen still
            // needs SSE OFF, so re-state target-feature=-sse,-sse2 here. An env
            // RUSTFLAGS OVERRIDES `.cargo/config.toml`'s `[target.i686-kernel]`
            // copy of that flag, so it must be repeated; scrub any inherited
            // encoded flags first so this RUSTFLAGS wins. Only these CRT-class
            // children are affected — the main kernel-rust build (which booted
            // fine without it) keeps the config-supplied flags.
            // env RUSTFLAGS OVERRIDES `.cargo/config.toml`'s `[target.i686-kernel]`
            // copy of `-sse,-sse2`, so every kernel crate built here must
            // RE-STATE the i386 no-SSE baseline (i386 kernel runs no-SSE — Phase
            // 0 leaves CR4.OSFXSR unset). amd64/arm64 kernel targets carry no
            // config rustflags, so there's nothing to preserve there.
            let sse = if self.arch.name == "i386" {
                "-C target-feature=-sse,-sse2 "
            } else {
                ""
            };
            // The CRT-class crates additionally need `-C no-prepopulate-passes`
            // or LLVM's default pre-passes miscompile them (captures(none)/DCE —
            // e.g. memset DCE'd to a spin; rtl-rust over-optimized 1.3MB-vs-6.1MB
            // and hung the kernel after ACPI). Mirrors sdk/lib/rtl/CMakeLists.txt
            // + sdk/lib/crt/libcntpr.cmake: rtl-rust on i386 only; cnt-rust-lib
            // on every arch.
            //
            // (#38/#35) The cmlib-rust no-prepop guard was REMOVED 2026-06-18: it was
            // added on the hypothesis that the #35 x86-ungate blocker was a
            // layout-sensitive captures(none)/DCE miscompile on cmlib's Hv* hive-write
            // path. That turned out to be a RED HERRING — the real blocker was an i386
            // stdcall/cdecl ABI mismatch in cmlib cmcheck/cmheal (fixed by declaring
            // the CMAPI fns extern "system"; field manual "#35 SOLVED"). With the ABI
            // fix in place the ungate boots ×3 WITHOUT guarding cmlib, so the guard is
            // dead weight. rtl-rust/i386 + cnt-rust-lib keep their genuine guards.
            let no_prepop = match (crate_name, self.arch.name) {
                ("rtl-rust", "i386") | ("cnt-rust-lib", _) => "-C no-prepopulate-passes ",
                _ => "",
            };
            // Match cmake's lint POSTURE: cmake builds the kernel rust on STABLE
            // (standard target, no build-std), where the kernel translation's
            // lints are warn-level and ship fine. The cargo-drive REQUIRES nightly
            // (for the softfloat kernel json's build-std) and the current nightly
            // ESCALATES several lints to deny on ntoskrnl-rust's (mostly amd64)
            // code — pre-existing translation-cleanup items, NOT cargo-drive
            // regressions (the code is exactly what cmake ships). Allow them so
            // the nightly build keeps cmake parity; real compile errors (type/ABI
            // mismatches) still fail. NOTE: `--cap-lints allow` is NOT usable here
            // — it makes rustc's `--print=file-names` build-std target probe emit
            // nothing ("output of --print=file-names missing"). Per-lint `-A` is
            // probe-safe, so allow them individually. (A whole-kernel lint cleanup
            // would let these be dropped.)
            //   - clashing_extern_declarations: ABI-compatible extern fns
            //     re-declared with divergent pointee types (DefaultQueryInfoBufferCheck).
            //   - static_mut_refs: `&mut STATIC` (Rust 2024 deny) in WMI/driver glue.
            let allow = if kernel_build_std {
                "-A clashing_extern_declarations -A static_mut_refs"
            } else {
                ""
            };
            // Opt-in post-opt LLVM-IR emission for the captures(none) audit
            // (tools/scan_captures.py). Adds per-CGU .ll files next to the
            // normal artifacts in this crate's -rustlib target dir; the emit
            // is additive (rustc unions --emit), so the .a still builds.
            let emit_ir = if env::var("VIBEOS_EMIT_LLVM_IR").as_deref() == Ok("1") {
                "--emit=llvm-ir "
            } else {
                ""
            };
            // BUILTIN-TARGET CODEGEN RESTORE (stable-toolchain Axis B). The custom
            // kernel json set `frame-pointer: always`; the builtin triples do not
            // (x86_64-pc-windows-gnu defaults to may-omit, aarch64-pc-windows-gnullvm
            // to non-leaf). Frame pointers are load-bearing here — the kernel debug
            // toolkit's stack walk (`qemu/inspect.sh bt`, triage-bugcheck) and the
            // bugcheck stack trace both chase the frame chain — so restore them
            // explicitly rather than inherit the target default.
            //
            // The json's other two non-default opts need no action: `panic-strategy:
            // abort` is already the workspace `[profile.release] panic = "abort"`, and
            // `requires-uwtable: true` is set by the builtin triples too (so Rust
            // frames keep their .pdata/.xdata — field manual §9.35).
            //
            // NOT restorable on stable: `function-sections: false` (it is `-Z
            // function-sections`, nightly-only). x86_64-pc-windows-gnu sets it false
            // anyway, so only arm64 flips to per-function sections. That is benign
            // for the KernelDll/Win32Dll kinds (no --gc-sections; Win32Dll forces
            // --no-gc-sections) and is covered by the ×3 gate for the Exe kinds.
            let frame_ptr = if self.target.is_json() {
                "" // the json already says frame-pointer: always
            } else {
                "-C force-frame-pointers=yes "
            };
            // i386 kernel bodies keep debug_assert! but drop rustc's UB checks
            // (`-Z ub-checks`, nightly -- this build-std path is the sanctioned
            // nightly user). Their entry point, core's check_language_ub, is a LEAF
            // function with a 1-byte local, and LLVM keeps the x86 stack aligned only
            // in functions that make calls: it runs with ESP odd. An interrupt there
            // pushes its frame at an odd address (i386 hardware does not realign,
            // amd64's does), and the next checked KTRAP_FRAME deref -- armed
            // hal-rust's KiEnterInterruptTrap -- panics on a "misaligned pointer"
            // the checks themselves produced. amd64 and arm64 keep the UB checks.
            let ub_checks = if kernel_build_std && self.arch.name == "i386" {
                " -Zub-checks=no"
            } else {
                ""
            };
            let rustflags = format!("{sse}{frame_ptr}{no_prepop}{emit_ir}{allow}{ub_checks}");
            cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
            cmd.env_remove("CARGO_BUILD_RUSTFLAGS");
            if !rustflags.is_empty() {
                cmd.env("RUSTFLAGS", rustflags);
            }
        } else {
            // User-mode siblings build on the rustup DEFAULT (stable) via the
            // `rustup run stable cargo` above. But the outer `cargo +nightly`
            // (ntdll-cargo) injects a full cargo/rustc env into build scripts —
            // RUSTC pointing at the NIGHTLY rustc, RUSTUP_TOOLCHAIN=nightly,
            // encoded build-std rustflags — all of which override the stable
            // toolchain and reintroduce the missing-std E0463. Scrub them so
            // the child cargo starts clean, exactly like cmake's fresh process.
            compiler_env::scrub_outer_compiler(&mut cmd);
            // Run from the cmake build dir, exactly like cmake's custom_command.
            // Cargo discovers `.cargo/config.toml` by CWD, not by --manifest-path:
            // from here the workspace `sdk/lib/rust/.cargo/config.toml` is NOT
            // found, so its `[target.i686-pc-windows-gnu] -sse,-sse2` rustflags
            // don't apply — cnt-rust's `float` feature needs SSE, and cmake
            // builds it with SSE for this exact reason.
            // With no cmake build, an empty build-<arch> (gitignored) is the same place for
            // config discovery, and it is the cwd the artifact store audits.
            std::fs::create_dir_all(&self.build_dir).expect("mkdir build-<arch> for nested cargo");
            cmd.current_dir(&self.build_dir);
            // cnt-rust REQUIRES `-C no-prepopulate-passes` on EVERY arch: its
            // per-fn cgu-split CRT shims miscompile under LLVM's default
            // pre-passes (corrupted printf/streamout + mem/string ops). On
            // arm64 that corrupted ntdll's own DbgPrint (status 0xc000003a
            // printed as 0xc) + the heap path → smss BUGCHECK 0x6A. i386
            // additionally disables SSE (cnt is built no-float there). This
            // mirrors sdk/lib/crt/libcntpr.cmake's per-arch RUSTFLAGS exactly;
            // the RUSTFLAGS scrub above dropped the inherited copy and
            // .cargo/config.toml isn't found from build_dir, so set it here.
            // See memory feedback_cnt_rust_no_prepopulate_amd64_arm64.
            //
            // An explicit `rustflags` (the main body's .rust_build_rustflags(),
            // e.g. freeldr-rust i386 `-sse,-sse2`) wins; otherwise restate the
            // i386 no-SSE baseline that `.cargo/config.toml` would have supplied
            // if this child cargo ran from the workspace. User-mode callbacks
            // can execute before the i386 SSE state is usable; ntdll-rust hit
            // STATUS_ILLEGAL_INSTRUCTION on compiler-emitted `xorps`.
            if let Some(rf) = rustflags {
                cmd.env("RUSTFLAGS", rf);
            } else {
                // c-r-p generalized this: every i386 Rust crate builds no-SSE
                // (kernel/loader code runs pre-CR4.OSFXSR), and cnt-rust-lib adds
                // -C no-prepopulate-passes on every arch. This subsumes the old
                // explicit cport-rust/hal-rust/cmlib-rust i386 match — cmlib-rust
                // (the CM hive cohab lib) gets -sse,-sse2 via the generic i386 rule.
                let mut parts: Vec<&str> = Vec::new();
                if self.arch.name == "i386" {
                    parts.push("-C target-feature=-sse,-sse2");
                }
                if crate_name == "cnt-rust-lib" {
                    parts.push("-C no-prepopulate-passes");
                }
                if !parts.is_empty() {
                    cmd.env("RUSTFLAGS", parts.join(" "));
                }
            }
        }
        println!("cargo:rerun-if-env-changed=VIBEOS_RUST_BODY_SHARDS");
        let shards = body_storage::parse_shards(env::var("VIBEOS_RUST_BODY_SHARDS").ok().as_deref());
        // Target JSON basenames are not identities: i686-kernel has distinct ABI
        // variants. Partition by the actual spec and compiler pin. Cargo itself
        // fingerprints features; keep them out of this key to reuse dependencies.
        let target_identity = match (&self.target, kernel) {
            (RustTarget::Json(path), true) => {
                println!("cargo:rerun-if-changed={}", path.display());
                std::fs::read(path).expect("read nested Cargo target spec")
            }
            _ => out_stem.as_bytes().to_vec(),
        };
        let pin_file = self.repo_root.join(if kernel_build_std {
            ".rust-nightly"
        } else {
            "rust-toolchain.toml"
        });
        println!("cargo:rerun-if-changed={}", pin_file.display());
        let pin = std::fs::read(&pin_file).expect("read nested Cargo toolchain pin");
        let flags: Vec<_> = ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS"]
            .iter()
            .map(|name| {
                let value = cmd.get_envs().find(|(key, _)| key == name)
                    .map(|(_, value)| value.map(|v| v.to_os_string()))
                    .unwrap_or_else(|| env::var_os(name));
                (name, value)
            }).collect();
        let effective_flags = flags.iter().find(|(name, _)| **name == "RUSTFLAGS")
            .and_then(|(_, value)| value.as_ref())
            .map(|value| value.to_str().expect("non-UTF8 Rust flags"))
            .unwrap_or("");
        let cargo_cwd = cmd.get_current_dir().map(Path::to_path_buf)
            .unwrap_or_else(|| env::current_dir().expect("nested Cargo working directory"));
        if let Some(archive) = artifact_store::get(
            &self.repo_root, &self.pkg_name, crate_name, self.arch.name, &out_stem,
            features, default_features, kernel, staticlib, effective_flags,
            flags.iter().any(|(name, value)| **name != "RUSTFLAGS"
                && value.as_ref().is_some_and(|value| !value.is_empty())),
            &cargo_cwd,
        ) {
            return archive;
        }
        let configuration = format!("{target_identity:?}|{pin:?}|{flags:?}");
        let emit_ir = env::var("VIBEOS_EMIT_LLVM_IR").as_deref() == Ok("1");
        let shared = body_storage::enabled(shards, emit_ir).then(|| {
            body_storage::Shared::acquire(&workspace, self.arch.name, crate_name,
                configuration.as_bytes(), shards)
                .unwrap_or_else(|e| panic!("shared body storage: {e}"))
        });
        let tgt_dir = shared.as_ref().map(|storage| storage.target_dir().to_path_buf())
            .unwrap_or(tgt_dir);
        cmd.env("CARGO_TARGET_DIR", &tgt_dir);
        let status = cmd
            .traced_status()
            .unwrap_or_else(|e| panic!("cargo build -p {crate_name}: {e}"));
        if !status.success() {
            panic!("cargo build -p {crate_name} failed");
        }
        let a = tgt_dir.join(format!("{out_stem}/release/{lib_file}"));
        if !a.exists() {
            panic!("{crate_name} produced no {lib_file} at {}", a.display());
        }
        // Read the matching depfile while the shard remains locked too.
        cargo_deps::watch(&a.with_extension("d"), &a, &workspace);
        match shared {
            Some(storage) => {
                let request = format!("{crate_name}|{features:?}|{default_features}|{kernel}|{staticlib}|{configuration}");
                let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
                storage.snapshot(&a, &out, request.as_bytes())
                    .unwrap_or_else(|e| panic!("snapshot nested Cargo archive: {e}"))
            }
            None => a,
        }
    }

    /// `lib<target>.a` from every source compile_commands.json records for cmake target
    /// `target`, in the order it records them, which is the order cmake archives them:
    /// a symbol two members define (libuuid has 397) resolves to the first.
    fn replay_target(&self, target: &str, ar: &Path) -> PathBuf {
        let db_path = self.build_dir.join("compile_commands.json");
        if cmake_off_forced() || !db_path.exists() {
            // No cmake build: the committed recipe of that compile (cargo_cell_gen
            // --clib-recipes), with cmake's C and ASM rules rebuilt from their templates.
            return self.compile_recipe(target, ar);
        }
        if !db_path.exists() {
            panic!(
                "engine:{target} replays cmake's compile of it and needs {} — \
                 configure the build dir first",
                db_path.display()
            );
        }
        println!("cargo:rerun-if-changed={}", db_path.display());
        let entries = ccdb_entries(&db_path);
        let marker = format!("CMakeFiles/{target}.dir/");
        let mut sources: Vec<String> = Vec::new();
        for o in entries.iter() {
            if !file_command(o).contains(&marker) {
                continue;
            }
            let file = o.iter().find(|(k, _)| k == "file").map(|(_, v)| v.clone());
            let file = file.unwrap_or_else(|| panic!("compile_commands entry without file"));
            if !sources.contains(&file) {
                sources.push(file);
            }
        }
        if sources.is_empty() {
            panic!("engine:{target}: compile_commands.json has no sources for cmake target {target}");
        }
        let rels: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
        self.replay_c_archive(target, &rels, &entries, ar)
    }

    /// `lib<name>.a` from `sources`, each compiled with the exact command cmake's
    /// compile_commands.json records for it in cmake target `name` (the `.pch` dropped,
    /// `-o` into OUT_DIR): `c_from_compile_db`, and the engine-built C libraries.
    fn replay_c_archive(
        &self,
        name: &str,
        sources: &[&str],
        entries: &[Vec<(String, String)>],
        ar: &Path,
    ) -> PathBuf {
        // Disambiguate sources compiled into multiple cmake targets by
        // the target's object dir. `name` IS the cmake target name.
        let marker = format!("CMakeFiles/{name}.dir");
        let out_sub = self.obj_dir().join(format!("ccdb-{name}"));
        std::fs::create_dir_all(&out_sub).ok();
        let mut objs = Vec::new();
        for rel in sources {
            // Normal sources are repo-relative; GENERATED sources (e.g.
            // the RPC `*_s.c` server stub from add_rpc_files) live under
            // build-<arch>/ — pass them build-dir-relative and fall back
            // to build_dir when the repo path doesn't exist. Keeps the
            // API arch-agnostic (no `build-i386/` hardcoded in rel).
            let repo_abs = self.repo_root.join(rel);
            let abs = if repo_abs.exists() {
                repo_abs
            } else {
                self.build_dir.join(rel)
            };
            let (dir, cmd) = lookup_cc(entries, &abs, &marker).unwrap_or_else(|| {
                panic!(
                    "no compile_commands.json entry for {} in target {name} \
                     (marker {marker}) — is it a source of that cmake target?",
                    abs.display()
                )
            });
            let stem = abs
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_else(|| panic!("bad source path {}", abs.display()));
            let obj = out_sub.join(format!("{stem}.o"));
            if objs.contains(&obj) {
                panic!("{name}: two sources compile to {} — objects are named by stem", obj.display());
            }
            let args = transform_cc(&cmd, obj.to_str().expect("utf8 obj path"));
            // cmake compiles through ccache (RULE_LAUNCH_COMPILE) and compile_commands.json
            // leaves the launcher out. The same command from the same directory hits the
            // entry cmake's own compile left.
            let mut c = if self.cmake_uses_ccache() {
                let mut c = Command::new("ccache");
                c.arg(&args[0]);
                c
            } else {
                Command::new(&args[0])
            };
            let status = c
                .args(&args[1..])
                .current_dir(&dir)
                .native_status(&abs)
                .unwrap_or_else(|e| panic!("replay clang for {rel} (lib {name}): {e}"));
            if !status.success() {
                panic!("c_from_compile_db: compile failed for {rel} (lib {name})");
            }
            objs.push(obj);
            println!("cargo:rerun-if-changed={}", abs.display());
        }
        let lib_a = self.obj_dir().join(format!("lib{name}.a"));
        let _ = std::fs::remove_file(&lib_a); // ar rcs appends; start fresh
        let status = Command::new(ar)
            .arg("rcs")
            .arg(&lib_a)
            .args(&objs)
            .traced_status()
            .unwrap_or_else(|e| panic!("ar ({}): {e}", ar.display()));
        if !status.success() {
            panic!("ar failed for c_from_compile_db {name}");
        }
        lib_a
    }

    fn engine_crate(&self, crate_name: &str, features: &[&str], defaults: bool) -> PathBuf {
        let stem = self.arch.rust_user_target;
        self.build_rust_crate(crate_name, features, defaults, stem, false, None)
    }

    /// Whether the configured cmake build compiles through ccache (`ENABLE_CCACHE`).
    fn cmake_uses_ccache(&self) -> bool {
        self.cmake_cache_bool("ENABLE_CCACHE").unwrap_or(false)
    }

    /// A BOOL from the configured build's CMakeCache.txt; None with no configured build.
    fn cmake_cache_bool(&self, name: &str) -> Option<bool> {
        let cache = self.build_dir.join("CMakeCache.txt");
        println!("cargo:rerun-if-changed={}", cache.display());
        let text = std::fs::read_to_string(cache).ok()?;
        let line = text.lines().find(|l| l.starts_with(&format!("{name}:")))?;
        Some(matches!(
            line.rsplit('=').next().map(str::to_ascii_uppercase).as_deref(),
            Some("1" | "ON" | "TRUE" | "YES" | "Y")
        ))
    }

    /// A `static_libs` entry's archive: `engine:<name>` is built here (`engine_lib`),
    /// anything else is a cmake-built `.a` under the build dir.
    fn static_lib_path(&self, lib: &str, clang: &Path, ar: &Path) -> PathBuf {
        if let Some(name) = lib.strip_prefix("engine:") {
            return self.engine_lib(name, clang, ar);
        }
        let full = self.build_dir.join(lib);
        if !full.exists() {
            panic!("missing static lib: {} — run ninja first", full.display());
        }
        full
    }

    /// The small cmake static libraries the engine builds from their own sources,
    /// with the commands cmake ran, so a cell links them without cmake building
    /// anything first:
    ///
    /// - `oldnames`: the import library of msvcrt's POSIX names (`strdup` ->
    ///   `_strdup`), `dlltool -d sdk/lib/crt/moldname-msvcrt.def -k` (oldnames.cmake).
    /// - `chkstk`: the stack probe, sdk/lib/crt's `chkstk` target (one `.s` per arch).
    /// - `pseh`: sdk/lib/pseh's `pseh` target: `dummy.c` (the two `_SEH2_Volatile*`
    ///   globals `pseh2.h` reads), plus the i386 SEH runtime asm.
    /// - `wine`, `conutils`, `uuid`: those cmake targets' C (`replay_target`). They
    ///   read headers cmake generates (psdk/xdk), and uuid's `*_i.c` are widl output,
    ///   so the cell's CMakeLists orders it after those targets (`uuid_sources`).
    /// - `c:<target>`: any other cmake C target, the same way.
    /// - `conutils_rust`, `delayimp_rust`, ...: the crate cmake's custom command built
    ///   into that archive, built as a sibling (`rust_lib`) with the same flags.
    fn engine_lib(&self, name: &str, clang: &Path, ar: &Path) -> PathBuf {
        let i386 = self.arch.name == "i386";
        match name {
            "wine" | "conutils" | "uuid" => self.replay_target(name, ar),
            // The `rust_staticlib_variant` crates (and the hand-rolled custom commands
            // that do the same): `cargo build -p <crate>` with these features and the
            // user-mode flags, which on i386 are the `-sse,-sse2` .cargo/config.toml
            // gave cmake's build from the workspace.
            "conutils_rust" => self.engine_crate("conutils-rust", &[], true),
            "delayimp_rust" => self.engine_crate("delayimp-rust", &[], true),
            "evtlib_rust" => self.engine_crate("evtlib-rust", &[], true),
            "hid_hidparser_rust" => self.engine_crate("hidparser-rust", &["user"], false),
            // sdk/lib/inflib's custom command: RUSTFLAGS -sse,-sse2 on i386 (what the
            // config gives too), empty elsewhere (no config rustflags there either).
            "inflib_rust" => self.engine_crate("inflib-rust", &[], true),
            "cryptlib_rust" => self.engine_crate("cryptlib-rust", &[], true),
            "sptilib_rust" => self.engine_crate("sptilib-rust", &[], true),
            "ntoskrnl_vista_rust" => self.engine_crate("ntoskrnl_vista-rust", &[], true),
            "ntoskrnl_vista_utf8" => self.engine_crate("utf8", &[], true),
            // libcntpr.cmake: `float` on amd64/arm64 only; the user-target build adds
            // its `-C no-prepopulate-passes` (and i386's -sse,-sse2) for cnt-rust-lib.
            "cntpr_rust" => {
                let feats: &[&str] = if i386 { &[] } else { &["float"] };
                self.engine_crate("cnt-rust-lib", feats, true)
            }
            "skiplist_rust" => self.engine_crate("skiplist-rust", &[], true),
            "tzlib_rust" => self.engine_crate("tzlib-rust", &[], true),
            // concfg-rust is an rlib (winsrv-rust's dependency); cmake builds the
            // console CPL's archive with `cargo rustc --crate-type staticlib
            // --features standalone`, and on i386 RUSTFLAGS `-sse,-sse2
            // -C no-prepopulate-passes` (win32ss/user/winsrv/concfg/CMakeLists.txt).
            "concfg_rust" => {
                let flags = i386.then_some("-C target-feature=-sse,-sse2 -C no-prepopulate-passes");
                let stem = self.arch.rust_user_target;
                self.build_rust_crate_as("concfg-rust", &["standalone"], true, stem, false, flags, true)
            }
            // Any other cmake C target, by name: `engine:c:<target>`.
            o if o.starts_with("c:") => self.replay_target(&o[2..], ar),
            // Any `rust_staticlib_variant(<name> <crate> "<features>" <defaults>)` archive,
            // spelled `engine:crate:<crate>:<features,...>:<TRUE|FALSE>` by
            // tools/cargo_cell_gen.py: that function's `cargo build -p <crate>` on the
            // builtin triple, which `engine_crate` repeats. (A variant that passes ENV keeps
            // a named arm above.)
            o if o.starts_with("crate:") => {
                let mut parts = o["crate:".len()..].splitn(3, ':');
                let krate = parts.next().unwrap_or_default();
                let features: Vec<&str> =
                    parts.next().unwrap_or_default().split(',').filter(|f| !f.is_empty()).collect();
                let defaults = matches!(parts.next(), Some("TRUE" | "ON" | "1" | "YES"));
                self.engine_crate(krate, &features, defaults)
            }
            "oldnames" => {
                let def = self.repo_root.join("sdk/lib/crt/moldname-msvcrt.def");
                println!("cargo:rerun-if-changed={}", def.display());
                let dlltool = self.repo_root.join(format!(
                    ".build-tools/llvm-mingw/bin/{}-dlltool",
                    self.arch.triple
                ));
                let dir = self.obj_dir().join("engine-oldnames");
                std::fs::create_dir_all(&dir).ok();
                let lib_a = dir.join("oldnames.a");
                let _ = std::fs::remove_file(&lib_a);
                let status = Command::new(&dlltool)
                    .arg("-m")
                    .arg(self.arch.dlltool_machine)
                    .arg("-d")
                    .arg(&def)
                    .arg("-k")
                    .arg("-l")
                    .arg("oldnames.a")
                    .arg("-t")
                    .arg("oldnames")
                    .current_dir(&dir)
                    .traced_status()
                    .unwrap_or_else(|e| panic!("dlltool ({}): {e}", dlltool.display()));
                if !status.success() || !lib_a.exists() {
                    panic!("dlltool failed for engine:oldnames");
                }
                lib_a
            }
            "chkstk" => {
                let src = match self.arch.name {
                    "i386" => "sdk/lib/crt/except/i386/chkstk_asm.s",
                    "amd64" => "sdk/lib/crt/except/amd64/chkstk_ms.s",
                    _ => "sdk/lib/crt/except/arm64/chkstk_asm.s",
                };
                self.build_archive("engine_chkstk", &[src], &[], clang, ar)
            }
            "pseh" => {
                let asm: &[&str] = if i386 {
                    &["sdk/lib/pseh/i386/seh.s", "sdk/lib/pseh/i386/seh_prolog.s"]
                } else {
                    &[]
                };
                self.build_archive("engine_pseh", asm, &["sdk/lib/pseh/dummy.c"], clang, ar)
            }
            o => panic!("static_libs: no engine-built library `engine:{o}` (see engine_lib)"),
        }
    }

    /// `lib<name>.a` in OUT_DIR from header-free C (`c_rels`, compiled first, the
    /// order cmake's archives list them) and asm run through the kernel asm include
    /// chain, as an on-demand archive (see `asm_archive`).
    fn build_archive(
        &self,
        name: &str,
        asm_rels: &[&str],
        c_rels: &[&str],
        clang: &Path,
        ar: &Path,
    ) -> PathBuf {
        let mut objs = Vec::new();
        for rel in c_rels {
            let source = self.repo_root.join(rel);
            if !source.exists() {
                panic!("missing C source for archive {name}: {}", source.display());
            }
            println!("cargo:rerun-if-changed={}", source.display());
            let mut b = cc::Build::new();
            apply_base(&mut b, clang, AsmDefineSet::Kernel, &self.arch);
            self.apply_prefix_map(&mut b);
            // cmake compiles one section per kind; cc-rs's default -ffunction-sections
            // -fdata-sections gives every global its own `.data$<sym>`, which the link
            // orders differently, moving the globals and every reference to them.
            b.file(&source)
                .out_dir(self.obj_dir())
                .flag("-std=gnu2x")
                .flag("-fno-function-sections")
                .flag("-fno-data-sections");
            trace_cc(&b, &source);
            objs.extend(native_deps::compile(&b, &source));
        }
        let llvm_mingw = self.repo_root.join(".build-tools/llvm-mingw");
        let mut b = cc::Build::new();
        apply_base(&mut b, clang, self.asm_set, &self.arch);
        self.apply_prefix_map(&mut b);
        b.out_dir(self.obj_dir())
            .define("__ASM__", None)
            .flag("-nostdinc")
            .flag("-isystem")
            .flag(self.repo_root.join("sdk/include/crt").to_str().unwrap())
            .flag("-isystem")
            .flag(llvm_mingw.join("lib/clang/19/include").to_str().unwrap());
        self.add_kernel_asm_includes(&mut b);
        b.flag("-x").flag("assembler-with-cpp");
        for rel in asm_rels {
            let source = self.repo_root.join(rel);
            if !source.exists() {
                panic!("missing asm source for archive {name}: {}", source.display());
            }
            println!("cargo:rerun-if-changed={}", source.display());
            let mut single = b.clone();
            single.file(&source);
            trace_cc(&single, &source);
            objs.extend(native_deps::compile(&single, &source));
        }
        if objs.is_empty() {
            panic!("archive {name}: cc-rs produced no objects");
        }
        let lib_a = self.obj_dir().join(format!("lib{name}.a"));
        let _ = std::fs::remove_file(&lib_a); // ar rcs appends; start fresh
        let status = Command::new(ar)
            .arg("rcs")
            .arg(&lib_a)
            .args(&objs)
            .traced_status()
            .unwrap_or_else(|e| panic!("ar ({}): {e}", ar.display()));
        if !status.success() {
            panic!("ar failed for archive {name}");
        }
        lib_a
    }

    fn obj_dir(&self) -> PathBuf {
        let d = PathBuf::from(env::var("OUT_DIR").unwrap()).join(match &self.bin {
            Some(b) => format!("objs-{b}"),
            None => "objs".to_string(),
        });
        std::fs::create_dir_all(&d).expect("mkdir OUT_DIR/objs");
        d
    }

    /// Returns `(def, stubs)` — the `.def` and the generated `-stub`
    /// bodies TU (`module_stubs.c`). The caller links `def` always and
    /// `stubs` only when `spec_stubs()` opted in.
    fn generate_def(&self, spec_rel: &str, dll_name: &str) -> (PathBuf, PathBuf) {
        let spec2def = self.compile_host_tool("sdk/tools/spec2def/spec2def.c", "spec2def");
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join(match &self.bin {
            Some(b) => format!("spec-gen-{b}"),
            None => "spec-gen".to_string(),
        });
        std::fs::create_dir_all(&out).expect("mkdir spec-gen");
        let spec = self.repo_root.join(spec_rel);
        if !spec.exists() {
            panic!("missing spec {}", spec.display());
        }
        let def = out.join("module.def");
        let stubs = out.join("module_stubs.c");
        let status = Command::new(&spec2def)
            .arg(format!("-n={dll_name}"))
            .arg(format!("-a={}", self.arch.spec2def))
            .arg(format!("-d={}", def.display()))
            .arg(format!("-s={}", stubs.display()))
            .arg("--version=0x502")
            .arg("--dbg")
            .arg(&spec)
            .traced_status()
            .unwrap_or_else(|e| panic!("spec2def: {e}"));
        if !status.success() {
            panic!("spec2def failed for {spec_rel}");
        }
        if !def.exists() {
            panic!("spec2def produced no .def at {}", def.display());
        }
        (def, stubs)
    }

    /// `spec --implib → .def → i686-w64-mingw32-dlltool → libNAME.a`.
    /// Mirrors cmake's `spec2def --implib` + `dlltool -m i386 -d ... -k
    /// -l libNAME.a -t NAME` exactly — produces a PE import lib without
    /// a cmake-built one (the spec carries the export names; the target
    /// DLL need not exist).
    fn generate_import_lib(&self, spec_rel: &str, dll_name: &str, basename: &str) -> PathBuf {
        let (out, implib_def) = self.generate_implib_def(spec_rel, dll_name, basename);
        let dlltool = self.repo_root.join(format!(
            ".build-tools/llvm-mingw/bin/{}-dlltool",
            self.arch.triple
        ));
        let lib_a = out.join(format!("lib{basename}.a"));
        let status = Command::new(&dlltool)
            .arg("-m")
            .arg(self.arch.dlltool_machine)
            .arg("-d")
            .arg(&implib_def)
            .arg("-k")
            .arg("-l")
            .arg(&lib_a)
            .arg("-t")
            .arg(basename)
            .traced_status()
            .unwrap_or_else(|e| panic!("dlltool ({}): {e}", dlltool.display()));
        if !status.success() {
            panic!("dlltool failed for {basename}");
        }
        if !lib_a.exists() {
            panic!("dlltool produced no {}", lib_a.display());
        }
        lib_a
    }

    /// The kernel's `kdcom.dll` import library, from the spec of the KD transport this
    /// build ships, chosen the way drivers/base/CMakeLists.txt chooses it: kdrosdbg
    /// (`kdcom.dll` from kdrosdbg.spec) unless `_WINKD_`, then kdcom. The two specs are
    /// identical today, which is why importing the wrong one went unnoticed; an edit to
    /// the shipped one must reach the kernel. With no configured build, config.cmake's
    /// default (`_WINKD_` FALSE) applies. `_WINKD_` + `GDB` ships kdgdb.dll, which no
    /// kernel import names, so that refuses.
    pub fn kd_import_lib(self) -> Self {
        let winkd = self.cmake_cache_bool("_WINKD_").unwrap_or(false);
        let spec = if !winkd {
            "drivers/base/kdrosdbg/kdrosdbg.spec"
        } else if self.cmake_cache_bool("GDB").unwrap_or(false) {
            panic!("kd_import_lib: _WINKD_ with GDB ships kdgdb.dll, not kdcom.dll (not modelled)")
        } else {
            "drivers/base/kdcom/kdcom.spec"
        };
        self.import_lib(spec, "kdcom.dll", "kdcom")
    }

    /// `spec --implib → .def → gen-delaylib.py → lib<NAME>_delayed.a`, the command
    /// cmake's `generate_import_lib` runs for a delay-import lib.
    fn generate_delay_import_lib(&self, spec_rel: &str, dll_name: &str, basename: &str) -> PathBuf {
        let (out, implib_def) = self.generate_implib_def(spec_rel, dll_name, basename);
        let bin = self.repo_root.join(".build-tools/llvm-mingw/bin");
        let script = self.repo_root.join("sdk/tools/gen-delaylib.py");
        let lib_a = out.join(format!("lib{basename}_delayed.a"));
        let _ = std::fs::remove_file(&lib_a);
        let status = Command::new("python3")
            .arg(&script)
            .arg("--def")
            .arg(&implib_def)
            .arg("--arch")
            .arg(self.arch.name)
            .arg("--out")
            .arg(&lib_a)
            .arg("--clang")
            .arg(bin.join(format!("{}-clang", self.arch.triple)))
            .arg("--ar")
            .arg(bin.join(format!("{}-ar", self.arch.triple)))
            .current_dir(&out)
            .traced_status()
            .unwrap_or_else(|e| panic!("gen-delaylib.py: {e}"));
        if !status.success() {
            panic!("gen-delaylib.py failed for {basename}");
        }
        if !lib_a.exists() {
            panic!("gen-delaylib.py produced no {}", lib_a.display());
        }
        println!("cargo:rerun-if-changed={}", script.display());
        lib_a
    }

    /// `spec2def --implib` → `<OUT_DIR>/implib-gen/<basename>_implib.def`.
    fn generate_implib_def(&self, spec_rel: &str, dll_name: &str, basename: &str) -> (PathBuf, PathBuf) {
        let spec2def = self.compile_host_tool("sdk/tools/spec2def/spec2def.c", "spec2def");
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("implib-gen");
        std::fs::create_dir_all(&out).expect("mkdir implib-gen");
        let spec = self.repo_root.join(spec_rel);
        if !spec.exists() {
            panic!("missing spec {}", spec.display());
        }
        let implib_def = out.join(format!("{basename}_implib.def"));
        let status = Command::new(&spec2def)
            .arg("--version=0x502")
            .arg("--dbg")
            .arg(format!("-n={dll_name}"))
            .arg(format!("-a={}", self.arch.spec2def))
            .arg("--implib")
            .arg(format!("-d={}", implib_def.display()))
            .arg(&spec)
            .traced_status()
            .unwrap_or_else(|e| panic!("spec2def --implib: {e}"));
        if !status.success() {
            panic!("spec2def --implib failed for {spec_rel}");
        }
        (out, implib_def)
    }

    /// cmake-off (WS1 DEEP): regenerate the xdk-derived SDK headers (`winnt.h`,
    /// `ntdef.h`, the DDK headers, …) that the boot-RTL C floor's `#include
    /// <rtl.h>` closure pulls. cmake produces these into `build-<arch>/sdk/
    /// include/{psdk,ddk}/` via `add_xdk_header` (native-hpp over the
    /// `sdk/include/xdk/*.template.h` sources, byte-stable + arch-independent);
    /// with no cmake build we run the same transform ourselves. native-hpp
    /// (`sdk/tools/hpp/hpp.c`) is self-hosted like spec2def. Returns the
    /// generated-include base dir (with `psdk/` + `ddk/` subdirs) for the caller
    /// to add as `-I` (appended AFTER the source-tree `-I`, matching cmake's
    /// build-dir-after-source-dir order). Output proven byte-identical to cmake's.
    fn generate_xdk_headers(&self) -> PathBuf {
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("xdk-gen");
        self.generate_xdk_headers_into(&out);
        out
    }

    /// `generate_xdk_headers` into `out` (which gets `psdk/` and `ddk/`).
    fn generate_xdk_headers_into(&self, out: &Path) {
        let hpp = self.compile_host_tool("sdk/tools/hpp/hpp.c", "native-hpp");
        let xdk_src = self.repo_root.join("sdk/include/xdk");
        // (template, out-relative) — mirrors sdk/include/xdk/CMakeLists.txt.
        const HEADERS: &[(&str, &str)] = &[
            ("winnt.template.h", "psdk/winnt.h"),
            ("ntdef.template.h", "psdk/ntdef.h"),
            ("devioctl.template.h", "psdk/devioctl.h"),
            ("wdm.template.h", "ddk/wdm.h"),
            ("ntddk.template.h", "ddk/ntddk.h"),
            ("ntifs.template.h", "ddk/ntifs.h"),
        ];
        for (tmpl, rel) in HEADERS {
            let dst = out.join(rel);
            std::fs::create_dir_all(dst.parent().unwrap()).expect("mkdir xdk-gen");
            // Write to a temp path then atomically rename into place: a later
            // compile (or a concurrent cell's build script) never sees a
            // half-written header. (A partial `winnt.h` shows up as a spurious
            // "unterminated conditional directive".) native-hpp resolves the
            // template name against its WORKING_DIR (the xdk source dir), exactly
            // like the cmake custom_command.
            let tmp = dst.with_extension("h.tmp");
            let mut command = Command::new(&hpp);
            command
                .arg(tmpl)
                .arg(&tmp)
                .current_dir(&xdk_src);
            let status = native_deps::run_hpp(&mut command, &xdk_src.join(tmpl), |cmd| {
                cmd.traced_status()
            })
            .unwrap_or_else(|e| panic!("native-hpp {tmpl}: {e}"));
            if !status.success() {
                panic!("native-hpp failed for {tmpl}");
            }
            std::fs::rename(&tmp, &dst)
                .unwrap_or_else(|e| panic!("rename {} -> {}: {e}", tmp.display(), dst.display()));
            println!("cargo:rerun-if-changed={}", xdk_src.join(tmpl).display());
        }
    }

    /// cmake-off (WS1 DEEP): compile every `c_from_compile_db` lib from its
    /// committed per-arch flag template instead of replaying cmake's recorded
    /// command. Each `cflags/<name>.<arch>.flags` (repo-relative) is cmake's
    /// exact blrtl flag set minus the (inert) PCH + build-dir `-I`, with the
    /// repo root abstracted to `{REPO}`; we substitute it back, append the
    /// native-hpp xdk include dirs, and run clang per source. Objects are
    /// instruction/symbol/reloc identical to cmake's (only `.debug_*` comp_dir
    /// strings differ). Same archive + link semantics as the replay branch.
    fn compile_c_db_committed(&self, clang: &Path, ar: &Path) {
        let xdk = self.generate_xdk_headers();
        for (name, sources) in &self.c_db_libs {
            let flags_rel =
                format!("sdk/lib/rust/reactos-link/cflags/{name}.{}.flags", self.arch.name);
            let flags_path = self.repo_root.join(&flags_rel);
            // No flag template, but a recipe (cargo_cell_gen --clib-recipes): opengl32's
            // i386 asm shims. The recipe is the newer form of the same thing.
            let recipe = self.repo_root.join(format!(
                "sdk/lib/rust/reactos-link/clibs/{}/{name}.recipe",
                self.arch.name
            ));
            if !flags_path.exists() && recipe.exists() {
                let lib_a = self.compile_recipe(name, ar);
                link_arg!(self, "{}", lib_a.display());
                continue;
            }
            let raw = std::fs::read_to_string(&flags_path).unwrap_or_else(|e| {
                panic!(
                    "cmake-off c_from_compile_db({name}) needs committed flags {} \
                     (no compile_commands.json) — regen via tools/gen_cflags.py: {e}",
                    flags_path.display()
                )
            });
            println!("cargo:rerun-if-changed={}", flags_path.display());
            let repo = self.repo_root.to_str().expect("utf8 repo root");
            let flags: Vec<String> = raw
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.replace("{REPO}", repo))
                .collect();
            let out_sub = self.obj_dir().join(format!("ccdb-{name}"));
            std::fs::create_dir_all(&out_sub).ok();
            // gendib libs (win32k residue): produce the 3 generated DIB tables
            // in-build (no compile_commands), keyed by basename for resolution.
            let gendib = if self.gendib_libs.contains(name) {
                self.generate_gendib()
            } else {
                std::collections::HashMap::new()
            };
            // bugcodes.h (windmc-generated, arch-identical) is committed next to
            // the engine; win32k.h's closure needs it. Harmless for other libs.
            let committed_hdrs = self
                .repo_root
                .join("sdk/lib/rust/reactos-link/committed-headers");
            let mut objs = Vec::new();
            for rel in sources {
                // A GENERATED gendib source (dib*gen.c) resolves to its native-
                // gendib OUT_DIR copy; everything else is repo-relative.
                let base = Path::new(rel).file_name().and_then(|s| s.to_str()).unwrap_or("");
                let abs = match gendib.get(base) {
                    Some(p) => p.clone(),
                    None => {
                        let p = self.repo_root.join(rel);
                        if !p.exists() {
                            panic!("missing c_from_compile_db source {}", p.display());
                        }
                        p
                    }
                };
                let stem = abs
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_else(|| panic!("bad source path {}", abs.display()));
                let obj = out_sub.join(format!("{stem}.o"));
                let status = Command::new(clang)
                    // Generated-header dirs go FIRST (cmake interleaves its
                    // `build-<arch>/` generated `-I` BEFORE the source `sdk/include/
                    // wine` dir, so the generated `winnt.h`/`ddk/{wdm,ntddk}.h` win
                    // over wine's competing copies — the loaders pull wine/winnt.h,
                    // which else redefines `_EXCEPTION_REGISTRATION_RECORD` vs
                    // ndk/rtltypes.h). The 6 native-hpp headers have no other source
                    // `.h` (only `.template.h`), so front-placing them matches
                    // cmake's resolution and leaves blrtl/win32k byte-identical.
                    // `-I xdk-gen` (base) resolves the `<psdk/winnt.h>` prefixed form;
                    // `-I xdk-gen/{psdk,ddk}` the bare `<winnt.h>`/`<wdm.h>` form.
                    .arg("-I")
                    .arg(&xdk)
                    .arg("-I")
                    .arg(xdk.join("psdk"))
                    .arg("-I")
                    .arg(xdk.join("ddk"))
                    .arg("-I")
                    .arg(&committed_hdrs)
                    .args(&flags)
                    .arg(&abs)
                    .arg("-o")
                    .arg(&obj)
                    .current_dir(&self.repo_root)
                    .native_status(&abs)
                    .unwrap_or_else(|e| {
                        panic!("committed-flags clang for {rel} (lib {name}): {e}")
                    });
                if !status.success() {
                    panic!("cmake-off c_from_compile_db: compile failed for {rel} (lib {name})");
                }
                objs.push(obj);
                println!("cargo:rerun-if-changed={}", abs.display());
            }
            let lib_a = self.obj_dir().join(format!("lib{name}.a"));
            let _ = std::fs::remove_file(&lib_a);
            let status = Command::new(ar)
                .arg("rcs")
                .arg(&lib_a)
                .args(&objs)
                .traced_status()
                .unwrap_or_else(|e| panic!("ar ({}): {e}", ar.display()));
            if !status.success() {
                panic!("ar failed for c_from_compile_db {name}");
            }
            link_arg!(self, "{}", lib_a.display());
        }
    }

    /// cmake-off: self-host `geninc`. Cross-compile `genincdata.c` for this arch
    /// (committed kernel flags + native-hpp xdk headers + committed `bugcodes.h`),
    /// extract its `.asmdef` COFF section, and emit the per-arch ks*.inc to
    /// `OUT_DIR/asm/`. Returns that dir for the kernel asm `#include` path. The
    /// `.asmdef` table is pure compile-time constants, so the `.obj` section bytes
    /// (and hence the emitted text) are byte-identical to cmake's `native-geninc`.
    pub(crate) fn generate_ks_inc(&self) -> PathBuf {
        self.ks_includes.get_or_generate(|| self.generate_ks_inc_uncached())
    }

    fn generate_ks_inc_uncached(&self) -> PathBuf {
        let out_asm = PathBuf::from(env::var("OUT_DIR").unwrap()).join("asm");
        std::fs::create_dir_all(&out_asm).expect("mkdir asm");
        let dst = out_asm.join(self.ks_inc_name());
        let clang = self.clang_path();
        let xdk = self.generate_xdk_headers();
        let committed_hdrs = self
            .repo_root
            .join("sdk/lib/rust/reactos-link/committed-headers");
        let flags_rel =
            format!("sdk/lib/rust/reactos-link/cflags/genincdata.{}.flags", self.arch.name);
        let flags_path = self.repo_root.join(&flags_rel);
        let raw = std::fs::read_to_string(&flags_path).unwrap_or_else(|e| {
            panic!(
                "cmake-off geninc needs committed flags {} \
                 (no compile_commands.json) — regen via tools/gen_cflags.py: {e}",
                flags_path.display()
            )
        });
        println!("cargo:rerun-if-changed={}", flags_path.display());
        let repo = self.repo_root.to_str().expect("utf8 repo root");
        let flags: Vec<String> = raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.replace("{REPO}", repo))
            .collect();
        let src = self.repo_root.join("sdk/include/asm/genincdata.c");
        let obj = out_asm.join("genincdata.o");
        // Same xdk/bugcodes include layering as compile_c_db_committed: xdk-gen
        // base (psdk/-prefixed forms) + psdk + ddk (bare forms) + bugcodes.h.
        let status = Command::new(&clang)
            // Generated-header dirs first (see compile_c_db_committed).
            .arg("-I")
            .arg(&xdk)
            .arg("-I")
            .arg(xdk.join("psdk"))
            .arg("-I")
            .arg(xdk.join("ddk"))
            .arg("-I")
            .arg(&committed_hdrs)
            .args(&flags)
            .arg(&src)
            .arg("-o")
            .arg(&obj)
            .current_dir(&self.repo_root)
            .native_status(&src)
            .unwrap_or_else(|e| panic!("cmake-off geninc clang for genincdata.c: {e}"));
        if !status.success() {
            panic!("cmake-off geninc: genincdata.c compile failed");
        }
        let asmdef = read_coff_section(&obj, ".asmdef");
        std::fs::write(&dst, emit_ks_inc(&asmdef))
            .unwrap_or_else(|e| panic!("write {}: {e}", dst.display()));
        println!("cargo:rerun-if-changed={}", src.display());
        out_asm
    }

    /// Compile a single-.c host tool with the host cc, return its path.
    fn compile_host_tool(&self, src_rel: &str, name: &str) -> PathBuf {
        memo(&HOST_TOOLS, format!("{src_rel}|{name}"), || self.compile_host_tool_once(src_rel, name))
    }

    fn compile_host_tool_once(&self, src_rel: &str, name: &str) -> PathBuf {
        let host = env::var("HOST").expect("HOST");
        let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("host-tools");
        std::fs::create_dir_all(&out).expect("mkdir host-tools");
        let exe = out.join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        });
        let cc_tool = cc::Build::new()
            .target(&host)
            .host(&host)
            .opt_level(2)
            .get_compiler();
        let src = self.repo_root.join(src_rel);
        let mut cmd = Command::new(cc_tool.path());
        cmd.args(cc_tool.args());
        cmd.arg("-O2").arg("-o").arg(&exe).arg(&src);
        let status = cmd
            .native_status(&src)
            .unwrap_or_else(|e| panic!("host cc ({}): {e}", cc_tool.path().display()));
        if !status.success() {
            panic!("host cc failed to compile {src_rel}");
        }
        exe
    }
}

/// Common kernel-mode clang flags + the chosen DEFINES (cmake's
/// `FLAGS =`/`DEFINES =` lines for any ntoskrnl/HAL TU). Arch-gated: the
/// `-m`/`-march` x86 codegen flags only apply where valid (clang rejects
/// `-march=pentium` on aarch64, etc.).
fn apply_base(build: &mut cc::Build, clang: &Path, set: AsmDefineSet, arch: &ArchCfg) {
    build
        .compiler(clang)
        .warnings(false)
        .extra_warnings(false)
        .flag(format!("--target={}", arch.triple))
        .flag("-pipe")
        .flag("-fms-extensions")
        .flag("-fno-strict-aliasing")
        .flag("-fno-common")
        .flag("-Wno-microsoft")
        .flag("-Wno-pragma-pack")
        .flag("-fno-associative-math")
        .flag("-fno-builtin-stpcpy")
        .flag("-gdwarf-2")
        .flag("-gstrict-dwarf")
        .flag("-O1")
        .flag("-fno-optimize-sibling-calls")
        .flag("-fno-omit-frame-pointer")
        .flag("-Wno-unknown-warning-option");
    if arch.is_x86 {
        // x87 long double on both i386 + amd64 (matches cmake's kernel CRT).
        build.flag("-mlong-double-64");
    }
    if arch.is_i386 {
        // i386-only perf/ABI tuning (cmake's ntoskrnl/HAL i386 FLAGS line).
        build
            .flag("-march=pentium")
            .flag("-mtune=generic")
            .flag("-mstackrealign");
    }
    for (n, v) in defines(set, arch) {
        build.define(n, v);
    }
}


// ---- geninc self-host (ks*.inc / ksarm64.h from genincdata's .asmdef) -----
// The kernel asm offset constants (`ks386.inc`/`ksamd64.inc`/`ksarm64.h`) are
// produced by compiling `sdk/include/asm/genincdata.c` to a PE object whose
// `.asmdef` section is a flat `ASMGENDATA[]` table of compile-time constants
// (FIELD_OFFSET/sizeof/enum values), then walking that table into assembler
// text. cmake builds + runs the `geninc` host tool to do the walk; cmake-off we
// extract + emit it directly (the `.asmdef` values are pure constants with no
// relocations, so the `.obj`'s raw section bytes equal the linked module's).
// This is the in-engine analogue of `tools/gen_ks386_inc.py`, generalized to
// all three arches, and is byte-identical to cmake's `native-geninc` output.

/// Read a named section's raw bytes out of a COFF object file. The COFF File
/// Header is at offset 0 (no DOS stub for a `.obj`); section table follows the
/// (here empty) optional header. Mirrors `read_coff_section` in gen_ks386_inc.py.
fn read_coff_section(obj: &Path, section_name: &str) -> Vec<u8> {
    let raw = std::fs::read(obj).unwrap_or_else(|e| panic!("read {}: {e}", obj.display()));
    if raw.len() < 20 {
        panic!("{} too small to be a COFF object", obj.display());
    }
    let u16le = |o: usize| u16::from_le_bytes([raw[o], raw[o + 1]]) as usize;
    let u32le = |o: usize| {
        u32::from_le_bytes([raw[o], raw[o + 1], raw[o + 2], raw[o + 3]]) as usize
    };
    let num_sections = u16le(2);
    let opt_header_size = u16le(16);
    let sec_table = 20 + opt_header_size;
    for i in 0..num_sections {
        let off = sec_table + i * 40;
        let nb = &raw[off..off + 8];
        let nul = nb.iter().position(|&c| c == 0).unwrap_or(8);
        if &nb[..nul] == section_name.as_bytes() {
            let raw_size = u32le(off + 16);
            let raw_addr = u32le(off + 20);
            if raw_addr == 0 || raw_size == 0 {
                panic!("section {section_name:?} has no raw data in {}", obj.display());
            }
            return raw[raw_addr..raw_addr + raw_size].to_vec();
        }
    }
    panic!("section {section_name:?} not found in {}", obj.display());
}

/// Walk an `ASMGENDATA[]` `.asmdef` blob into ks*.inc text. Each record is 64
/// bytes: `char Type; char Name[55]; u64 Value`. GNU (non-`-ms`) format — the
/// only one cmake uses with the llvm-mingw (non-MSVC) toolchain. Mirrors
/// `emit_ks386_inc` in gen_ks386_inc.py, identical for all arches.
fn emit_ks_inc(asmdef: &[u8]) -> String {
    let mut out = String::new();
    let mut off = 0usize;
    while off + 64 <= asmdef.len() {
        let chunk = &asmdef[off..off + 64];
        let ty = chunk[0];
        let nul = chunk[1..56].iter().position(|&c| c == 0).unwrap_or(55);
        let name = std::str::from_utf8(&chunk[1..1 + nul])
            .unwrap_or_else(|_| panic!("non-utf8 ASMGENDATA name at offset {off}"));
        let value = u64::from_le_bytes(chunk[56..64].try_into().unwrap());
        match ty {
            0 => break,                                          // TYPE_END
            1 => out.push_str(&format!("{name}\n")),             // TYPE_RAW
            2 => out.push_str(&format!("{name} = 0x{value:x}\n")), // TYPE_CONSTANT
            3 => out.push_str(&format!("\n/* {name} */\n")),     // TYPE_HEADER
            other => panic!("unknown ASMGENDATA type {other} at offset {off}"),
        }
        off += 64;
    }
    out
}

// ---- compile_commands.json replay (c_from_compile_db) --------------------
// Minimal, dependency-free reader + command transform. cmake/Ninja emits a
// flat JSON array of `{directory, command, file[, output]}` objects whose
// values are all strings — we don't need a general JSON parser, only the three
// string fields per object.

/// Decode one JSON string starting at the opening quote `b[start] == '"'`.
/// Returns (decoded, index-after-closing-quote). Handles the escapes cmake
/// actually emits (`\"`, `\\`, `\/`, `\n`, `\t`, `\r`); paths/commands here are
/// ASCII so bytes pass through directly.
fn read_json_string(b: &[u8], start: usize) -> (String, usize) {
    let mut i = start + 1;
    let mut s = String::new();
    while i < b.len() {
        match b[i] {
            b'\\' if i + 1 < b.len() => {
                let c = match b[i + 1] {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'/' => '/',
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    other => other as char,
                };
                s.push(c);
                i += 2;
            }
            b'"' => return (s, i + 1),
            c => {
                s.push(c as char);
                i += 1;
            }
        }
    }
    (s, i)
}

/// Generate a Rust RPC **server** stub for `idl_rel` and return the path to the
/// emitted `.rs` (for a stub crate's `build.rs` to hand to `include!`). This is
/// the Rust-output counterpart of the `rpc_server_stub` link method: instead of
/// compiling widl's generated C and linking it, we transliterate it to Rust
/// (via `tools/widl_to_rust.py`, against the `midl-rt` crate) so the stub is
/// pure Rust with no C in the tree.
///
/// Steps (all in `OUT_DIR/rpc-<name>/`):
///   1. Replay cmake's recorded `widl` command (from `build.ninja`) → `<name>.c`
///      + `<name>.h`.
///   2. Compile `<name>.c` → `<name>.o` (compile_commands.json replay) PURELY to
///      read the exact per-arch external symbol decorations (i386 stdcall `@N`,
///      incl. by-value struct params) — the `.o` is never linked.
///   3. Run `widl_to_rust.py` (`--symobj <name>.o --arch <a> --nm <llvm-nm>`)
///      → `<name>.rs`.
///
/// `name` is the stub base (e.g. `svcctl_s`); `c_marker` the cmake target whose
/// compile_commands entry compiles `<name>.c` (e.g. `services_c`). Designed to
/// be called from a `build.rs`; reads `CARGO_CFG_TARGET_ARCH` / `CARGO_MANIFEST_DIR`
/// / `OUT_DIR` from the environment exactly like `Linker::new`.
pub fn generate_rpc_stub_rust(idl_rel: &str, name: &str, c_marker: &str) -> PathBuf {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");
    let arch = arch_cfg(&target_arch);
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.ancestors().nth(4).expect("repo root").to_path_buf();
    let build_dir = repo_root.join(format!("build-{}", arch.name));
    let out_sub = PathBuf::from(env::var("OUT_DIR").unwrap()).join(format!("rpc-{name}"));
    std::fs::create_dir_all(&out_sub).ok();

    // cmake-off (WS1 DEEP, RPC): with no configured cmake build there is no
    // host-tools `widl`, no `build.ninja`, and no `compile_commands.json` — the
    // three inputs the regenerate path below needs (widl codegen + the widl
    // command + the decoration compile). Fall back to the committed per-arch stub
    // `.rs` (the deterministic widl→widl_to_rust.py output), exactly the WS2/WS3
    // pattern: the cmake build regenerates it below (so it can never go stale),
    // the committed copy is used ONLY when there is no cmake build at all.
    // Regen: build services-cargo / winlogon-cargo with a configured build-<arch>
    // and copy `OUT_DIR/rpc-<name>/<name>.rs` → `<stub-crate>/data/<name>.<arch>.rs`.
    let widl = build_dir.join("host-tools/bin/widl");
    if !widl.exists()
        || !build_dir.join("build.ninja").exists()
        || !build_dir.join("compile_commands.json").exists()
    {
        let committed = manifest_dir.join(format!("data/{name}.{}.rs", arch.name));
        if !committed.exists() {
            panic!(
                "cmake-off generate_rpc_stub_rust({name}) needs committed stub {} \
                 (no configured cmake build) — regen by building this cell with a \
                 configured build-{} and copying OUT_DIR/rpc-{name}/{name}.rs there",
                committed.display(),
                arch.name
            );
        }
        println!("cargo:rerun-if-changed={}", committed.display());
        println!("cargo:rerun-if-changed={}", repo_root.join(idl_rel).display());
        return committed;
    }
    let ninja_path = build_dir.join("build.ninja");
    let ninja = std::fs::read_to_string(&ninja_path)
        .unwrap_or_else(|e| panic!("need {}: {e}", ninja_path.display()));
    let db_path = build_dir.join("compile_commands.json");
    let db_text = std::fs::read_to_string(&db_path)
        .unwrap_or_else(|e| panic!("need {} (CMAKE_EXPORT_COMPILE_COMMANDS): {e}", db_path.display()));
    let entries = parse_ccdb(&db_text);

    let gen_c = out_sub.join(format!("{name}.c"));
    let gen_h = out_sub.join(format!("{name}.h"));
    let gen_o = out_sub.join(format!("{name}.o"));
    let gen_rs = out_sub.join(format!("{name}.rs"));

    // 1. widl replay → .c + .h
    let raw = widl_command_from_ninja(&ninja, name)
        .unwrap_or_else(|| panic!("no widl CUSTOM_COMMAND for {name}.c in build.ninja"));
    let cmd = rewrite_widl_outputs(
        &raw, name, gen_c.to_str().unwrap(), gen_h.to_str().unwrap(),
    );
    let st = Command::new("sh").arg("-c").arg(&cmd).traced_status()
        .unwrap_or_else(|e| panic!("run widl for {name}: {e}"));
    assert!(st.success(), "widl codegen failed for {name}");

    // 2. compile .c → .o for symbol decorations (replay; never linked)
    let marker = format!("CMakeFiles/{c_marker}.dir");
    let suffix = format!("/{name}.c");
    let (dir, ccmd, cmake_file) = lookup_cc_suffix(&entries, &suffix, &marker)
        .unwrap_or_else(|| panic!("no compile_commands entry for *{suffix} in {c_marker}"));
    let mut args = transform_cc(&ccmd, gen_o.to_str().unwrap());
    for a in args.iter_mut() {
        if *a == cmake_file {
            *a = gen_c.to_str().unwrap().to_string();
        }
    }
    let st = Command::new(&args[0]).args(&args[1..]).current_dir(&dir).traced_status()
        .unwrap_or_else(|e| panic!("compile {name}.c for decorations: {e}"));
    assert!(st.success(), "decoration compile failed for {name}");

    // 3. transliterate to Rust
    let nm = repo_root.join(".build-tools/llvm-mingw/bin/llvm-nm");
    let script = repo_root.join("tools/widl_to_rust.py");
    let st = Command::new("python3")
        .arg(&script)
        .arg(&gen_c)
        .arg("--symobj").arg(&gen_o)
        .arg("--arch").arg(arch.name)
        .arg("--nm").arg(&nm)
        .arg("-o").arg(&gen_rs)
        .traced_status()
        .unwrap_or_else(|e| panic!("run widl_to_rust.py for {name}: {e}"));
    assert!(st.success(), "widl_to_rust.py failed for {name}");

    println!("cargo:rerun-if-changed={}", repo_root.join(idl_rel).display());
    println!("cargo:rerun-if-changed={}", script.display());
    println!("cargo:rerun-if-changed={}", ninja_path.display());
    gen_rs
}

/// Parse compile_commands.json into one `(key, value)` list per object. All
/// values are strings; alternating string tokens inside an object are
/// key/value (cmake never nests objects or uses non-string values here).
fn parse_ccdb(text: &str) -> Vec<Vec<(String, String)>> {
    let b = text.as_bytes();
    let mut i = 0;
    let mut objects = Vec::new();
    let mut cur: Vec<(String, String)> = Vec::new();
    let mut pending_key: Option<String> = None;
    let mut in_object = false;
    while i < b.len() {
        match b[i] {
            b'{' => {
                in_object = true;
                cur = Vec::new();
                pending_key = None;
                i += 1;
            }
            b'}' => {
                if in_object {
                    objects.push(std::mem::take(&mut cur));
                    in_object = false;
                }
                i += 1;
            }
            b'"' if in_object => {
                let (s, ni) = read_json_string(b, i);
                i = ni;
                match pending_key.take() {
                    None => pending_key = Some(s),
                    Some(k) => cur.push((k, s)),
                }
            }
            _ => i += 1,
        }
    }
    objects
}

/// Find the `(directory, command)` for the entry whose `file` is `abs` AND
/// whose command targets `marker` (the `CMakeFiles/<target>.dir` object dir).
/// The marker disambiguates a source compiled into multiple targets with
/// different flags (e.g. `locale.c` in both `rtl` and `rtl_vista`). Matches the
/// raw cmake-written path first, then a canonicalized fallback (symlinks /
/// `..`), so a repo_root that differs cosmetically from cmake's still resolves.
fn lookup_cc(
    entries: &[Vec<(String, String)>],
    abs: &Path,
    marker: &str,
) -> Option<(String, String)> {
    let field = |o: &[(String, String)], key: &str| -> Option<String> {
        o.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    };
    let abs_s = abs.to_str();
    let abs_canon = std::fs::canonicalize(abs).ok();
    for o in entries {
        let file = match field(o, "file") {
            Some(f) => f,
            None => continue,
        };
        let file_hit = Some(file.as_str()) == abs_s
            || abs_canon
                .as_ref()
                .zip(std::fs::canonicalize(&file).ok())
                .is_some_and(|(a, b)| *a == b);
        if !file_hit {
            continue;
        }
        let cmd = file_command(o);
        if cmd.contains(marker) {
            return Some((field(o, "directory").unwrap_or_default(), cmd));
        }
    }
    None
}

fn file_command(o: &[(String, String)]) -> String {
    o.iter()
        .find(|(k, _)| k == "command")
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

/// Like `lookup_cc` but matches the compile_commands entry by file-path SUFFIX
/// (for generated sources whose abs path the caller doesn't reconstruct).
/// Returns (directory, command, matched-file-path).
fn lookup_cc_suffix(
    entries: &[Vec<(String, String)>],
    suffix: &str,
    marker: &str,
) -> Option<(String, String, String)> {
    let field = |o: &[(String, String)], key: &str| -> Option<String> {
        o.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    };
    for o in entries {
        let file = match field(o, "file") {
            Some(f) => f,
            None => continue,
        };
        if !file.ends_with(suffix) {
            continue;
        }
        let cmd = file_command(o);
        if cmd.contains(marker) {
            return Some((field(o, "directory").unwrap_or_default(), cmd, file));
        }
    }
    None
}

/// Extract cmake's recorded `widl` CUSTOM_COMMAND for `<name>.c` from build.ninja
/// (the `build … <name>.c <name>.h … : CUSTOM_COMMAND` statement's `COMMAND =`).
/// Returns the raw shell command (`cd <dir> && widl … && widl …`).
fn widl_command_from_ninja(ninja: &str, name: &str) -> Option<String> {
    let out_c = format!("/{name}.c ");
    let lines: Vec<&str> = ninja.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if l.starts_with("build ") && l.contains(": CUSTOM_COMMAND") && l.contains(&out_c) {
            let mut j = i + 1;
            while j < lines.len() && !lines[j].starts_with("build ") {
                if let Some(cmd) = lines[j].trim_start().strip_prefix("COMMAND = ") {
                    return Some(cmd.to_string());
                }
                j += 1;
            }
        }
        i += 1;
    }
    None
}

/// Rewrite the two `-o <path>.{c,h}` outputs of a replayed `widl` command to our
/// OUT_DIR paths (leaving the `-H <name>.h` include-name hint + all `-I`/`-D`/
/// flags untouched). Whitespace-split is safe: ReactOS build paths have no spaces.
fn rewrite_widl_outputs(cmd: &str, name: &str, gen_c: &str, gen_h: &str) -> String {
    let dot_c = format!("/{name}.c");
    let dot_h = format!("/{name}.h");
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let mut out: Vec<String> = Vec::with_capacity(parts.len());
    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "-o" && i + 1 < parts.len() {
            let t = parts[i + 1];
            out.push("-o".to_string());
            if t.ends_with(&dot_c) {
                out.push(gen_c.to_string());
            } else if t.ends_with(&dot_h) {
                out.push(gen_h.to_string());
            } else {
                out.push(t.to_string());
            }
            i += 2;
            continue;
        }
        out.push(parts[i].to_string());
        i += 1;
    }
    out.join(" ")
}

/// Transform a replayed clang command: drop the ninja-built `.pch` (the
/// `-Winvalid-pch` and the `-Xclang -include-pch -Xclang <pch>` quad) while
/// keeping the configure-time `-Xclang -include -Xclang <cmake_pch.h>`
/// force-include, and redirect `-o <obj>` to our output. Whitespace-split is
/// safe: ReactOS build paths contain no spaces.
fn transform_cc(cmd: &str, obj: &str) -> Vec<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let mut out: Vec<String> = Vec::with_capacity(parts.len());
    let mut j = 0;
    while j < parts.len() {
        let p = parts[j];
        if p == "-Winvalid-pch" {
            j += 1;
            continue;
        }
        // -Xclang -include-pch -Xclang <pch>  (4 tokens) → drop
        if p == "-Xclang" && j + 1 < parts.len() && parts[j + 1] == "-include-pch" {
            j += 4;
            continue;
        }
        if p == "-o" && j + 1 < parts.len() {
            out.push("-o".to_string());
            out.push(obj.to_string());
            j += 2;
            continue;
        }
        out.push(p.to_string());
        j += 1;
    }
    out
}
