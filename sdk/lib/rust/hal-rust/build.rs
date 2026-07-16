//! hal-rust build script — emits the HAL *interrupt-flavor* cfgs the source
//! gates on, so no module hard-codes `target_arch` for the PIC-vs-APIC split.
//!
//! Two cfgs, mutually exclusive on x86:
//!   `hal_apic` — LAPIC/IOAPIC/TPR interrupt model (apic*, madt, rtctimer, tsc).
//!   `hal_pic`  — 8259 PIC interrupt model (pic_pic, pic_halinit, timer, profil).
//!
//! Resolution:
//!   amd64 (x86_64)     → `hal_apic` ALWAYS (amd64 has no PIC flavor).
//!   i386  (x86) + apic → `hal_apic` (the i386-apic-hal lane's opt-in flavor).
//!   i386  (x86)        → `hal_pic`  (default; also every FreeLDR mini-HAL,
//!                                    which never enables `apic`).
//!   aarch64            → neither (the x86 module isn't compiled there).
//!
//! The `apic` cargo feature is set by hal-cargo's build.rs from VIBEOS_APIC_HAL
//! (cmake -DVIBEOS_APIC_HAL=ON). Gating on these cfgs rather than on the feature
//! keeps amd64 APIC-by-default with no feature and lets one `#[cfg(hal_apic)]`
//! cover both x86_64 and i386-opt-in.

use std::env;

fn main() {
    // Declare the cfgs so pinned-stable/nightly rustc's unexpected_cfgs lint
    // (check-cfg) stays quiet — these names are unknown to cargo otherwise.
    println!("cargo:rustc-check-cfg=cfg(hal_apic)");
    println!("cargo:rustc-check-cfg=cfg(hal_pic)");

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    // Cargo sets CARGO_FEATURE_<NAME> for each enabled feature at build-script time.
    let apic_feature = env::var("CARGO_FEATURE_APIC").is_ok();

    match arch.as_str() {
        // amd64 is the APIC flavor unconditionally — no i386-style PIC option.
        "x86_64" => println!("cargo:rustc-cfg=hal_apic"),
        // i386: opt-in APIC, else PIC (default + all mini-HAL builds).
        "x86" => {
            if apic_feature {
                println!("cargo:rustc-cfg=hal_apic");
            } else {
                println!("cargo:rustc-cfg=hal_pic");
            }
        }
        // aarch64 / others: the x86 backend module is #[cfg]'d out entirely;
        // neither flavor cfg applies.
        _ => {}
    }
}
