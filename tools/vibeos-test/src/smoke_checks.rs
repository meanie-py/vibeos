//! Post-boot evidence checks. Requested rig state is an explicit environment DTO.
use crate::verdict::{args, kernel_asserts, py_lines, re, string};
use crate::{Error, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
pub type Env = BTreeMap<String, String>;
fn env<'a>(e: &'a Env, name: &str) -> &'a str {
    e.get(name).map(String::as_str).unwrap_or("")
}
fn yes(e: &Env, name: &str) -> bool {
    env(e, name).trim() == "1"
}
fn tail<F: Fn(&str) -> bool>(text: &str, n: usize, predicate: F) -> Vec<String> {
    let lines: Vec<_> = py_lines(text)
        .into_iter()
        .filter(|l| predicate(l))
        .map(|l| l.trim().into())
        .collect();
    lines[lines.len().saturating_sub(n)..].to_vec()
}
fn fail(class: &str, salient: Vec<String>) -> Value {
    json!({"classification":class,"salient":salient})
}
fn with_tail(first: impl Into<String>, extra: Vec<String>) -> Vec<String> {
    let mut out = vec![first.into()];
    out.extend(extra);
    out
}
pub fn booted_kernel(text: &str) -> Option<String> {
    re(r"(?i)\[FLDR\] load \S*[\\/](ntoskrnl\.exe|ntkrnlmp\.exe)")
        .captures(text)
        .map(|c| c[1].to_lowercase())
}
pub fn booted_hal(text: &str) -> Option<String> {
    re(r"(?i)\[FLDR\] load \S*[\\/](halmp\.dll|hal\.dll)")
        .captures(text)
        .map(|c| c[1].to_lowercase())
}
fn smp(e: &Env) -> Result<u64> {
    let raw = env(e, "VIBEOS_QEMU_SMP").trim();
    if !raw.is_empty() && raw.chars().all(|c| c.is_ascii_digit()) {
        let n = raw
            .parse()
            .map_err(|_| Error::new("VIBEOS_QEMU_SMP exceeds supported integer range"))?;
        Ok(n)
    } else if raw.chars().any(|c| c.is_numeric() && !c.is_ascii()) {
        Err("VIBEOS_QEMU_SMP requires ASCII decimal digits".into())
    } else {
        Ok(0)
    }
}
pub fn smp_check(text: &str, arch: &str, e: &Env) -> Result<Value> {
    let n = smp(e)?;
    let how = if n > 0 {
        format!("--smp {n}")
    } else {
        "the default boot".into()
    };
    if let Some(booted) = booted_kernel(text).filter(|k| k != "ntkrnlmp.exe") {
        let entry = if arch.starts_with("arm64") {
            "ReactOS_Arm64_MP"
        } else {
            "LiveCD_SMP"
        };
        return Ok(fail("UP_KERNEL_ON_SMP_RUN",vec![format!("{how} booted {booted}, not ntkrnlmp.exe"),format!("the media's DefaultOS= was patched to {entry} but freeldr booted the UP entry anyway — check the [{entry}] Options line")]));
    }
    let hal = booted_hal(text);
    if let Some(h) = hal.as_ref().filter(|h| h.as_str() != "halmp.dll") {
        return Ok(fail("SMP_UP_HAL",vec![format!("{how} loaded {h}, not halmp.dll — the MP kernel is running on the UP HAL, so HalStartNextProcessor is the FALSE stub and this boot is silently 1 CPU"),"the boot entry is missing /HAL=halmp.dll (livecd.ini [LiveCD_SMP], or qemu/freeldr.ini [ReactOS_Arm64_MP] on arm64)".into()]));
    }
    if text.contains("UP HAL stub called") {
        return Ok(fail("SMP_UP_HAL",vec!["MP kernel is running on a UP HAL — HalStartNextProcessor is the FALSE stub, so this boot is silently 1 CPU".into(),format!("loaded HAL was {}, so the UP body is wearing an MP name — check hal-rust's `smp` feature reached halmp-cargo",hal.as_deref().unwrap_or("not seen on serial"))]));
    }
    let missing: Vec<_> = (1..n)
        .filter(|c| !text.contains(&format!("CPU{c} scheduling live")))
        .map(|c| c.to_string())
        .collect();
    if !missing.is_empty() {
        return Ok(fail(
            "SMP_AP_NOT_SCHEDULING",
            vec![
                format!(
                    "-smp {n}: no `[SMP] CPU<n> scheduling live` for CPU {}",
                    missing.join(", ")
                ),
                "the BSP reached the desktop but that AP never ran a non-idle thread".into(),
            ],
        ));
    }
    Ok(Value::Null)
}
pub fn mem_mb(size: &str) -> Result<i64> {
    if size.is_empty() {
        return Ok(0);
    }
    let mut chars = size.char_indices();
    let (at, last) = chars.next_back().unwrap();
    let number = size[..at]
        .parse::<i64>()
        .map_err(|_| Error::new("invalid QEMU memory size"))?;
    number
        .checked_mul(if matches!(last, 'G' | 'g') { 1024 } else { 1 })
        .ok_or_else(|| Error::new("QEMU memory size exceeds integer range"))
}
pub fn payload_vm_lifetime(boot_budget: f64, drain: f64, e: &Env) -> Result<i64> {
    let raw = e
        .get("VIBEOS_QEMU_MAX_LIFETIME")
        .map(String::as_str)
        .unwrap_or("1800");
    let floor = raw
        .parse::<i64>()
        .map_err(|_| Error::new("invalid VIBEOS_QEMU_MAX_LIFETIME"))?;
    let total = boot_budget + drain;
    if !total.is_finite() || total >= i64::MAX as f64 || total < i64::MIN as f64 {
        return Err("payload lifetime exceeds integer range".into());
    }
    let requested = (total as i64)
        .checked_add(300)
        .ok_or("payload lifetime exceeds integer range")?;
    Ok(floor.max(requested))
}
pub fn ahci_requested(e: &Env) -> bool {
    yes(e, "VIBEOS_AHCI_CD")
}
pub fn nvme_requested(e: &Env) -> bool {
    yes(e, "VIBEOS_QEMU_NVME")
}
pub fn ahci_check(text: &str, e: &Env) -> Value {
    if !ahci_requested(e) || re(r"\[UNIATA\] AhciDetect: exit found=1\b").is_match(text) {
        return Value::Null;
    }
    let requirements = [
        r"\[DA\] AddDevice PCI\\VEN_8086&DEV_2922\S* \(drv \\Driver\\pciide\) -> 0x0",
        r"\[DA\] START PCI\\VEN_8086&DEV_2922\S* -> 0x0",
        r"\[CDDB\] check IDE\\CdRom",
    ];
    if requirements.iter().all(|p| re(p).is_match(text)) {
        return Value::Null;
    }
    fail("AHCI_NOT_ENGAGED",vec!["--ahci boot reached the desktop but neither uniata reported `AhciDetect: exit found=1` nor pciide started ich9-ahci (8086:2922) with a CdRom under it".into(),"the CD was not on ich9-ahci (backend.py rewrite) or no driver claimed it in AHCI mode (PCI#CC_0106: hivesys.inf, or ata_stack.inf on --ata-stack media)".into()])
}
pub fn ata_health_check(text: &str) -> Value {
    let specs = [
        (
            "ATA_ASSERT",
            r"\*\*\* ASSERT FAILED: .* at (?:atapi|pciidex|legide)-rust/",
        ),
        (
            "ATA_TIMEOUT",
            r"\((?:atapi|pciidex)-rust/[^)]*\) CH \d+: (?:Slot \d+ \(\w+\) timed out|Internal request timed out)",
        ),
        (
            "ATA_NO_INTERRUPT",
            r"\(pciidex-rust/[^)]*\) CH \d+: Could not connect to interrupt",
        ),
    ];
    for (class, pattern) in specs {
        let pattern = re(pattern);
        let lines: Vec<_> = py_lines(text)
            .into_iter()
            .filter(|l| pattern.is_match(l))
            .map(|l| l.trim().to_owned())
            .collect();
        if !lines.is_empty() {
            let first = match class {
                "ATA_ASSERT" => format!("{} ATA stack assertion(s) failed", lines.len()),
                "ATA_TIMEOUT" => format!(
                    "{} ATA request(s) timed out (a lost interrupt)",
                    lines.len()
                ),
                _ => "an ATA channel could not connect its interrupt and was not started".into(),
            };
            return fail(class, with_tail(first, lines.into_iter().take(4).collect()));
        }
    }
    Value::Null
}
pub fn kernel_assert_check(text: &str) -> Value {
    let assertions = kernel_asserts(text);
    if assertions.is_empty() {
        Value::Null
    } else {
        fail(
            "KERNEL_ASSERT",
            with_tail(
                format!("{} kernel assertion(s) failed", assertions.len()),
                assertions.into_iter().take(4).collect(),
            ),
        )
    }
}
pub fn legide_check(text: &str, e: &Env) -> Value {
    if !yes(e, "VIBEOS_QEMU_ISA_IDE") {
        return Value::Null;
    }
    let problem = if !text.contains("LEGIDE_DETECT: probing") {
        "legide never forced its probe: LEGIDE_DETECT did not reach the tripwire, or legide did not load"
    } else if !re(r"\[DA\] PiEnumerateDevice Root\\legide\\\d+: [1-9]\d* children").is_match(text) {
        "legide probed but reported no channel with a device (Root\\legide\\NNNN)"
    } else {
        return Value::Null;
    };
    fail(
        "LEGIDE_NOT_ENGAGED",
        with_tail(
            problem,
            tail(text, 4, |l| l.to_lowercase().contains("legide")),
        ),
    )
}
pub fn ahci_irq_check(text: &str, e: &Env) -> Value {
    if !ahci_requested(e) {
        return Value::Null;
    }
    let events: Vec<_> = py_lines(text)
        .iter()
        .filter(|l| l.starts_with("ahci_"))
        .filter_map(|l| l.split_whitespace().next().map(str::to_owned))
        .collect();
    if !events.iter().any(|e| e == "ahci_cmd_done") {
        return Value::Null;
    }
    let Some(first) = events.iter().position(|e| e == "ahci_irq_raise") else {
        return fail("AHCI_POLLING",vec![format!("{} AHCI commands completed and the controller never raised its interrupt: the guest driver never enabled it (GHC.IE/PxIE)",events.iter().filter(|e|e.as_str()=="ahci_cmd_done").count())]);
    };
    let after = &events[first..];
    let done = after
        .iter()
        .filter(|e| e.as_str() == "ahci_cmd_done")
        .count();
    let raised = after
        .iter()
        .filter(|e| e.as_str() == "ahci_irq_raise")
        .count();
    let (mut run, mut longest) = (0, 0);
    for ev in after {
        if ev == "ahci_cmd_done" {
            run += 1;
            longest = longest.max(run);
        } else if ev == "ahci_irq_raise" {
            run = 0;
        }
    }
    if longest > 8 {
        return fail("AHCI_POLLING",vec![format!("after the driver's first interrupt, {longest} AHCI commands completed in a row with no interrupt ({done} commands, {raised} raises): it is polling")]);
    }
    if !after.iter().any(|e| e == "ahci_irq_lower") {
        return fail("AHCI_IRQ_STUCK",vec![format!("the AHCI interrupt was raised and never lowered: the guest's ISR never cleared PxIS/IS ({done} commands, {raised} raises)")]);
    }
    Value::Null
}
pub fn net_suite_env(e: &Env, artifact_dir: &str) -> Env {
    let mut result = e.clone();
    result.remove("VIBEOS_QEMU_NVME");
    result.insert("VIBEOS_ARTIFACT_DIR".into(), artifact_dir.into());
    result
}
const NVME_PUBLISHED: &str = "[NVME] controller 0: namespace 1 published as a disk";
const NVME_MSIX: &str = "[NVME] controller 0: MSI-X: ";
const NVME_INTERRUPTS: &str = "[NVME] controller 0: interrupts working";
const NVME_PASSIVE: &str = "message(s) connected at passive level";
const ROOT_PORT: &str = "[DA] START PCI\\VEN_1B36&DEV_000C";
const SYSFB_HEADLESS: &str = "[sysfb] no firmware framebuffer: headless";
pub fn nvme_default_on(nvme: bool, arch: &str, e: &Env) -> bool {
    nvme || env(e, "VIBEOS_QEMU_NVME").trim() != "0" && arch == "amd64-bios"
}
pub fn headless_check(text: &str, e: &Env) -> Value {
    if !yes(e, "VIBEOS_QEMU_HEADLESS") || text.contains(SYSFB_HEADLESS) {
        Value::Null
    } else {
        fail(
            "NOT_HEADLESS",
            with_tail(
                format!("--headless boot, but sysfb never printed `{SYSFB_HEADLESS}`"),
                tail(text, 3, |l| l.contains("[sysfb]")),
            ),
        )
    }
}
pub fn boot_nvme_check(text: &str, e: &Env) -> Value {
    if !yes(e, "VIBEOS_QEMU_BOOT_NVME") || text.contains(NVME_PUBLISHED) {
        Value::Null
    } else {
        fail("NVME_BOOT_DISK_NOT_SEEN",with_tail(format!("booted from NVMe, but nvme.sys never published the boot namespace (`{NVME_PUBLISHED}`)"),tail(text,4,|l|l.contains("[NVME]"))))
    }
}
pub fn xhci_check(text: &str, e: &Env) -> Result<Value> {
    let mode = env(e, "VIBEOS_QEMU_XHCI_MSI").trim();
    if mode.is_empty() {
        return Ok(Value::Null);
    }
    let lines = tail(text, 4, |l| {
        l.contains("MSI") || l.contains("[USBPORT] interrupt")
    });
    if !text.contains("[XHCI] controller running") {
        return Ok(fail(
            "XHCI_NOT_ENGAGED",
            with_tail(
                "--xhci boot, but usbxhci never printed `[XHCI] controller running`",
                lines,
            ),
        ));
    }
    if !text.contains("[DA] START USB\\Vid_0627&Pid_0001") {
        return Ok(fail("XHCI_NO_TRANSFERS",with_tail(format!("the USB keyboard never started: its enumeration's control transfers never completed, so the controller's {mode} interrupts are not arriving"),lines)));
    }
    let message = text.contains("[USBPORT] interrupt: message vector");
    if mode == "line" {
        return Ok(if message {
            fail(
                "XHCI_NOT_LINE",
                with_tail("--xhci line, but usbport connected a message", lines),
            )
        } else {
            Value::Null
        });
    }
    let block = re(r"MSI enabled for 1b36:000d: (\d+) message\(s\)")
        .captures(text)
        .map(|c| c[1].to_owned());
    let enabled = if mode == "msix" {
        text.contains("MSI-X enabled for 1b36:000d")
    } else {
        block.is_some()
    };
    if !message || !enabled {
        return Ok(fail(
            "XHCI_NOT_MSI",
            with_tail(
                format!("--xhci {mode}, but the controller did not interrupt by {mode}"),
                lines,
            ),
        ));
    }
    let wanted = re(r"XHCI_MSI_MESSAGES=(?:0x)?([0-9a-fA-F]+)")
        .captures(env(e, "VIBEOS_TRIPWIRE"))
        .map(|c| c[1].to_owned());
    if mode == "msi" {
        if let (Some(block), Some(wanted)) = (block, wanted) {
            let want = u64::from_str_radix(&wanted, 16)
                .map_err(|_| Error::new("XHCI_MSI_MESSAGES exceeds u64 range"))?;
            let got = block
                .parse::<u64>()
                .map_err(|_| Error::new("XHCI MSI count exceeds u64 range"))?;
            if got != want {
                return Ok(fail("XHCI_MSI_BLOCK_SHORT",with_tail(format!("XHCI_MSI_MESSAGES asked for an MSI block of {want}, but pci.sys enabled {block}"),lines)));
            }
        }
    }
    Ok(Value::Null)
}
pub fn nvme_release_check(text: &str) -> Value {
    let lines = tail(text, 8, |l| {
        l.contains("[NVME]")
            || l.contains("MSI-X")
            || l.contains("[PNP] message")
            || l.contains("released")
    });
    for (needle, what) in [
        (
            "[NVME] controller 0: tripwire: failing START",
            "nvme.sys never failed its START for the tripwire",
        ),
        (
            "[NVME] controller 0: interrupts disconnected",
            "nvme.sys never disconnected its interrupts",
        ),
        (
            "MSI-X disabled for 1b36:0010",
            "pci.sys never turned the controller's MSI-X off at REMOVE",
        ),
    ] {
        if !text.contains(needle) {
            return fail("NVME_NOT_RELEASED", with_tail(what, lines));
        }
    }
    if !re(r"\[PNP\] message interrupts: ([1-9]\d*) back to the HAL, 0 refused").is_match(text) {
        return fail(
            "NVME_NOT_RELEASED",
            with_tail(
                "the PnP manager never gave the device's message vectors back to the HAL",
                lines,
            ),
        );
    }
    Value::Null
}
fn in_page(line: &str, page: u64) -> bool {
    re(r"address ([0-9a-fA-F]+)$")
        .captures(line)
        .and_then(|c| u64::from_str_radix(&c[1], 16).ok())
        .is_some_and(|n| n >= page && (n as u128) < page as u128 + 0x1000)
}
pub fn smmu_translate_check(text: &str, e: &Env) -> Result<Value> {
    let lines = tail(text, 6, |l| l.contains("SMMUv3"));
    if !re(r"\[HAL\] SMMUv3 at \S+: TRANSLATING").is_match(text) {
        return Ok(fail(
            "SMMU_NOT_TRANSLATING",
            with_tail(
                "SMMU=TRANSLATE, but the HAL never put the SMMUv3 in translate mode",
                lines,
            ),
        ));
    }
    if text.contains("NOT ATTACHED")
        || !re(r"\[HAL\] SMMUv3 at \S+: requester \S+ is StreamID \S+ - translating").is_match(text)
    {
        return Ok(fail(
            "SMMU_NOT_ATTACHED",
            with_tail(
                "a device's stream was never attached to the SMMU's identity map",
                lines,
            ),
        ));
    }
    if nvme_requested(e) && !text.contains("translating in a domain of its own") {
        return Ok(fail(
            "SMMU_NO_DOMAIN",
            with_tail(
                "nvme.sys is DMA-remapping compatible, but no device got a domain of its own",
                lines,
            ),
        ));
    }
    if re(r"\[VIOBLK\] [0-9a-f]{2}:[0-9a-f]{2}\.\d dev ").is_match(text)
        && !text.contains("claimed by its driver)")
    {
        let mut sal=with_tail("vioblk drove a PCI function, but no driver claimed a domain of its own for one (vioblk-rust / netstack-rust dma.rs)",tail(text,2,|l|l.contains("VIOBLK] 00:")));
        sal.extend(lines);
        return Ok(fail("SMMU_UNCLAIMED", sal));
    }
    let mut faults: Vec<String> = py_lines(text)
        .into_iter()
        .filter(|l| l.contains("SMMUv3") && (l.contains("FAULT") || l.contains("global error")))
        .map(|l| l.trim().into())
        .collect();
    let tripwire = env(e, "VIBEOS_TRIPWIRE");
    if tripwire.contains("NVME_STRAY_DMA") {
        let stray = re(r"(?m)tripwire: a stray Identify to RAM at 0x([0-9a-fA-F]+) \w+; (.*)$")
            .captures(text)
            .map(|c| (c[0].to_owned(), c[1].to_owned(), c[2].to_owned()));
        if stray.as_ref().is_none_or(|s| !s.2.contains("untouched")) {
            let mut sal = vec![if stray.is_some() {
                "a stray Identify to a page nvme.sys never mapped was not stopped".into()
            } else {
                "nvme.sys never reported its stray Identify".into()
            }];
            if let Some(s) = stray {
                sal.push(s.0);
            }
            return Ok(fail("SMMU_STRAY_DMA_LANDED", sal));
        }
        let stray = stray.unwrap();
        let page = u64::from_str_radix(&stray.1, 16)
            .map_err(|_| Error::new("stray DMA address exceeds u64 range"))?;
        if !faults.iter().any(|l| in_page(l, page)) {
            return Ok(fail("SMMU_MISSED_STRAY_DMA",vec![format!("the stray Identify to {page:#x} left its page untouched, but the SMMU reported no fault there"),stray.0]));
        }
        faults.retain(|l| !in_page(l, page));
    }
    if let Some(c) = re(r"NVME_BAD_DMA=(?:0x)?([0-9a-fA-F]+)").captures(tripwire) {
        let target = u64::from_str_radix(&c[1], 16)
            .map_err(|_| Error::new("NVME_BAD_DMA exceeds u64 range"))?;
        if !faults.iter().any(|l| in_page(l, target)) {
            let mut sal = with_tail(
                format!(
                    "nvme.sys aimed a DMA at {target:016x}, and the SMMU reported no fault there"
                ),
                faults.iter().take(2).cloned().collect(),
            );
            sal.extend(tail(text, 1, |l| l.contains("tripwire: an Identify")));
            return Ok(fail("SMMU_MISSED_BAD_DMA", sal));
        }
        faults.retain(|l| !in_page(l, target));
    }
    if !faults.is_empty() {
        return Ok(fail(
            "SMMU_FAULT",
            with_tail(
                format!(
                    "{} SMMU fault line(s): a device DMA'd outside the identity map",
                    faults.len()
                ),
                faults.into_iter().take(4).collect(),
            ),
        ));
    }
    Ok(Value::Null)
}
pub fn gic_check(text: &str, e: &Env) -> Result<Value> {
    let smmu = yes(e, "VIBEOS_QEMU_SMMU");
    let tripwire = env(e, "VIBEOS_TRIPWIRE");
    if smmu && tripwire.contains("SMMU=TRANSLATE") {
        return smmu_translate_check(text, e);
    }
    if smmu && !re(r"\[HAL\] IORT: SMMUv3 at \S+ was .*; now bypassing").is_match(text) {
        return Ok(fail(
            "SMMU_NOT_BYPASSED",
            with_tail(
                "--smmu boot, but the HAL never reported the SMMUv3 bypassing DMA",
                tail(text, 6, |l| l.contains("IORT")),
            ),
        ));
    }
    if env(e, "VIBEOS_QEMU_GIC").trim() != "3" {
        return Ok(Value::Null);
    }
    let lines = tail(text, 6, |l| l.contains("[GIC]") || l.contains("GICv3"));
    if !text.contains("[GIC] GICv3: distributor") {
        return Ok(fail("GIC_NOT_V3",with_tail("--gic 3 boot reached the desktop but the HAL never printed `[GIC] GICv3: distributor`",lines)));
    }
    if nvme_requested(e) && !re(r"\[HAL\] GICv3 ITS: [1-9]\d* of \d+ up").is_match(text) {
        return Ok(fail(
            "GIC_NO_ITS",
            with_tail(
                "--gic 3 --nvme boot, but the HAL never reported an ITS up",
                lines,
            ),
        ));
    }
    if tripwire.contains("ITS_MISSTEER")
        && !re(r"\[HAL\] GICv3 ITS \d+: LPI \d+ moved from CPU\d+ to CPU\d+").is_match(text)
    {
        return Ok(fail(
            "GIC_LPI_NOT_MOVED",
            with_tail(
                "ITS_MISSTEER bound every LPI one CPU over, and no MOVI ever moved one back",
                tail(text, 6, |l| l.contains("LPI") || l.contains("MISSTEER")),
            ),
        ));
    }
    if nvme_requested(e)
        && smp(e)?.max(1) > 1
        && !re(r"\[GIC\] CPU[1-9]\d*: first LPI delivered").is_match(text)
    {
        return Ok(fail("GIC_LPIS_ONE_CPU",with_tail("--gic 3 --nvme --smp boot, but no CPU but CPU0 ever took an LPI: the ITS did not spread NVMe's messages",tail(text,6,|l|l.contains("LPI")))));
    }
    Ok(Value::Null)
}
pub fn nvme_check(text: &str, e: &Env) -> Result<Value> {
    if !nvme_requested(e) {
        return Ok(Value::Null);
    }
    let tripwire = env(e, "VIBEOS_TRIPWIRE");
    if tripwire.contains("NVME_FAIL_START") {
        return Ok(nvme_release_check(text));
    }
    let lines = tail(text, 6, |l| l.contains("[NVME]"));
    if !text.contains(NVME_PUBLISHED) {
        return Ok(fail(
            "NVME_NOT_ENGAGED",
            with_tail(
                format!(
                    "--nvme boot reached the desktop but nvme.sys never printed `{NVME_PUBLISHED}`"
                ),
                if lines.is_empty() {
                    vec!["no [NVME] line at all: nvme.sys never loaded (PCI#CC_010802 in hivesys.inf?)".into()]
                } else {
                    lines
                },
            ),
        ));
    }
    if !text.contains(NVME_MSIX) {
        let msix = tail(text, 4, |l| l.contains("MSI-X"));
        let mut sal=with_tail(format!("the namespace was published but nvme.sys never printed `{NVME_MSIX}`: no MSI-X messages were assigned, so it fell back to its pin"),if msix.is_empty(){vec!["pci.sys printed no MSI-X line: the request never reached it".into()]}else{msix});
        sal.extend(lines);
        return Ok(fail("NVME_NOT_MSIX", sal));
    }
    if !text.contains(NVME_INTERRUPTS) {
        return Ok(fail("NVME_POLLING",with_tail(format!("the namespace was published but nvme.sys never printed `{NVME_INTERRUPTS}`: no completion arrived by interrupt, so the driver ran on its poll"),lines)));
    }
    if tripwire.contains("NVME_PASSIVE_ISR") && !text.contains(NVME_PASSIVE) {
        return Ok(fail("NVME_NOT_PASSIVE",with_tail(format!("NVME_PASSIVE_ISR boot, but nvme.sys never printed `{NVME_PASSIVE}`: its messages were not connected passive (iomgr irq.rs)"),lines)));
    }
    let test=re(r"\[NVME\] controller 0: self-test: read \d+ KiB .*?up to (\d+) commands in flight.*?(\d+) commands by DMA to the caller's pages").captures(text).map(|c|(c[1].to_owned(),c[2].to_owned()));
    let queued = test
        .as_ref()
        .map(|c| {
            c.0.parse::<u64>()
                .map_err(|_| Error::new("NVMe queued command count exceeds u64 range"))
        })
        .transpose()?;
    if queued.is_none_or(|n| n < 2) {
        return Ok(fail(
            "NVME_NOT_QUEUED",
            with_tail(
                if test.is_some() {
                    "nvme.sys's self-test never had two commands in flight at once"
                } else {
                    "nvme.sys printed no self-test read line"
                },
                lines,
            ),
        ));
    }
    let test = test.unwrap();
    let dma = test
        .1
        .parse::<u64>()
        .map_err(|_| Error::new("NVMe DMA command count exceeds u64 range"))?;
    if dma == 0 {
        return Ok(fail("NVME_BOUNCED",with_tail("nvme.sys's self-test read went through the bounce buffer, not by DMA to the reader's pages (nvme.rs `dma_ready` refused its MDL)",lines)));
    }
    if tripwire.contains("NVME_LOSE_COMPLETION=")
        && !re(r"\[NVME\] controller 0: self-test: [1-9]\d* reset\(s\)").is_match(text)
    {
        return Ok(fail("NVME_NOT_RESET",with_tail("the NVME_LOSE_COMPLETION tripwire threw a completion away, but nvme.sys never reported resetting the controller for it",lines)));
    }
    let bridge = env(e, "VIBEOS_QEMU_NVME_BRIDGE").trim();
    if bridge == "segment"
        && !re(r"\[HAL\] PCI segment 1: root bus 128-255 published").is_match(text)
    {
        return Ok(fail(
            "NVME_SEGMENT_NOT_PUBLISHED",
            with_tail(
                "--nvme-segment boot, but the HAL never published PCI segment 1's root bus",
                tail(text, 4, |l| l.contains("[HAL] PCI")),
            ),
        ));
    }
    if !bridge.is_empty() && !text.contains(ROOT_PORT) {
        return Ok(fail("NVME_NOT_BEHIND_BRIDGE",with_tail(format!("--nvme-bridge boot, but the PnP manager never started the root port (`{ROOT_PORT}`): the controller cannot have been behind it"),tail(text,4,|l|l.contains("[HAL] PCI")))));
    }
    Ok(Value::Null)
}
pub fn virtio_mmio_check(text: &str, arch: &str) -> Value {
    if arch != "arm64" {
        return Value::Null;
    }
    let lines = tail(text, 6, |l| l.contains("virtio-mmio"));
    if py_lines(text)
        .iter()
        .any(|l| l.contains("[VIOBLK] virtio-mmio") && l.contains(": interrupts working"))
    {
        return Value::Null;
    }
    fail("VIRTIO_MMIO_DISK_NOT_SERVED",with_tail("arm64 boot reached the desktop but vioblk never printed `[VIOBLK] virtio-mmio <base>: interrupts working` for the scratch disk",if lines.is_empty(){vec!["no virtio-mmio line at all: the HAL published no LNRO0005 block slot (hal-rust arm64/pnpdd.rs) or vioblk never bound it (ACPI#LNRO0005&DEV_0002 in hivesys.inf)".into()]}else{lines}))
}
pub fn dispatch(_root: &std::path::Path, request: &Value) -> Result<Value> {
    let a = args(request);
    let op = string(request, "op")?;
    let e: Env = a
        .get("env")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let text = || {
        a.get("text")
            .or_else(|| a.get("serial"))
            .or_else(|| a.get("stderr_text"))
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new("missing smoke check text"))
    };
    Ok(match op {
        "smoke_check_booted_kernel" => json!(booted_kernel(text()?)),
        "smoke_check_booted_hal" => json!(booted_hal(text()?)),
        "smoke_check_smp" => smp_check(text()?, string(a, "arch")?, &e)?,
        "smoke_check_mem_mb" => json!(mem_mb(string(a, "size")?)?),
        "smoke_check_payload_vm_lifetime" => json!(payload_vm_lifetime(
            a["boot_budget"].as_f64().ok_or("missing boot budget")?,
            a["drain"].as_f64().ok_or("missing drain")?,
            &e
        )?),
        "smoke_check_ahci_requested" => json!(ahci_requested(&e)),
        "smoke_check_nvme_requested" => json!(nvme_requested(&e)),
        "smoke_check_nvme_default_on" => json!(nvme_default_on(
            a["nvme"].as_bool().unwrap_or(false),
            a["arch"].as_str().unwrap_or(""),
            &e
        )),
        "smoke_check_ahci" => ahci_check(text()?, &e),
        "smoke_check_ata_health" => ata_health_check(text()?),
        "smoke_check_kernel_assert" => kernel_assert_check(text()?),
        "smoke_check_legide" => legide_check(text()?, &e),
        "smoke_check_ahci_irq" => ahci_irq_check(text()?, &e),
        "smoke_check_net_suite_env" => json!(net_suite_env(&e, string(a, "artifact_dir")?)),
        "smoke_check_headless" => headless_check(text()?, &e),
        "smoke_check_boot_nvme" => boot_nvme_check(text()?, &e),
        "smoke_check_xhci" => xhci_check(text()?, &e)?,
        "smoke_check_nvme_release" => nvme_release_check(text()?),
        "smoke_check_smmu_translate" => smmu_translate_check(text()?, &e)?,
        "smoke_check_gic" => gic_check(text()?, &e)?,
        "smoke_check_nvme" => nvme_check(text()?, &e)?,
        "smoke_check_virtio_mmio" => virtio_mmio_check(text()?, string(a, "arch")?),
        _ => return Err(Error::new(format!("unknown smoke check operation {op}"))),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_mp_pair_and_ap_liveness_are_required() {
        let mut env = Env::new();
        env.insert("VIBEOS_QEMU_SMP".into(), "2".into());
        assert_eq!(
            smp_check("[FLDR] load x\\ntoskrnl.exe\n", "amd64", &env).unwrap()["classification"],
            "UP_KERNEL_ON_SMP_RUN"
        );
        assert_eq!(
            smp_check(
                "[FLDR] load x\\ntkrnlmp.exe\n[FLDR] load x\\halmp.dll\n",
                "amd64",
                &env
            )
            .unwrap()["classification"],
            "SMP_AP_NOT_SCHEDULING"
        );
    }
    #[test]
    fn ahci_proves_interrupts_not_just_driver_reports() {
        let env = [("VIBEOS_AHCI_CD".into(), "1".into())]
            .into_iter()
            .collect();
        assert_eq!(
            ahci_irq_check("ahci_cmd_done a\n", &env)["classification"],
            "AHCI_POLLING"
        );
        assert!(ahci_irq_check("ahci_irq_raise\nahci_cmd_done\nahci_irq_lower\n", &env).is_null());
    }
    #[test]
    fn dma_faults_are_never_hidden_by_the_desktop() {
        let env = [("VIBEOS_QEMU_NVME".into(), "1".into())]
            .into_iter()
            .collect();
        let text="[HAL] SMMUv3 at f000: TRANSLATING\n[HAL] SMMUv3 at f000: requester 00:00 is StreamID 001 - translating\ntranslating in a domain of its own\n[HAL] SMMUv3 at f000: FAULT address deadbeef\n";
        assert_eq!(
            smmu_translate_check(text, &env).unwrap()["classification"],
            "SMMU_FAULT"
        );
    }
}
