//! Native architecture/media/guest-device plans shared by every VM launcher.
use crate::{backend, media, vm::VmConfig};
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    time::Duration,
};
fn text<'a>(env: &'a BTreeMap<String, String>, key: &str) -> &'a str {
    env.get(key).map(|s| s.trim()).unwrap_or("")
}
fn enabled(env: &BTreeMap<String, String>, key: &str) -> bool {
    matches!(text(env, key), "1" | "true" | "yes")
}
fn number(env: &BTreeMap<String, String>, key: &str, default: u64) -> Result<u64> {
    let raw = text(env, key);
    if raw.is_empty() {
        return Ok(default);
    }
    raw.parse()
        .map_err(|_| Error::new(format!("{key} must be a nonnegative integer (got {raw:?})")))
}
fn float(env: &BTreeMap<String, String>, key: &str, default: f64) -> Result<f64> {
    let raw = text(env, key);
    let value = if raw.is_empty() {
        default
    } else {
        raw.parse::<f64>()
            .map_err(|_| Error::new(format!("{key} must be a number")))?
    };
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::new(format!("{key} must be finite and positive")));
    }
    Ok(value)
}
pub fn smp_count(env: &BTreeMap<String, String>) -> Result<Option<u64>> {
    let raw = text(env, "VIBEOS_QEMU_SMP");
    if raw.is_empty() {
        return Ok(None);
    }
    let n = number(env, "VIBEOS_QEMU_SMP", 0)?;
    if n == 0 {
        return Err("VIBEOS_QEMU_SMP must be a positive integer".into());
    }
    Ok(Some(n))
}
pub fn hostile_hazards(env: &BTreeMap<String, String>) -> Result<BTreeSet<String>> {
    let all = [
        "q35", "cpu", "nops2", "novga", "mem", "serial", "a76", "el2",
    ];
    let raw = text(env, "VIBEOS_QEMU_HOSTILE").to_ascii_lowercase();
    if raw == "all" {
        return Ok(all.iter().map(|s| s.to_string()).collect());
    }
    let values = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let bad = values
        .iter()
        .filter(|v| !all.contains(&v.as_str()))
        .collect::<Vec<_>>();
    if !bad.is_empty() {
        return Err(Error::new(format!(
            "VIBEOS_QEMU_HOSTILE: unknown hazards {bad:?}"
        )));
    }
    Ok(values)
}
pub fn tripwire_args(env: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let value = text(env, "VIBEOS_TRIPWIRE");
    if value.contains(',') {
        return Err("VIBEOS_TRIPWIRE must not contain commas; use spaces or semicolons".into());
    }
    Ok(if value.is_empty() {
        Vec::new()
    } else {
        vec![
            "-fw_cfg".into(),
            format!("name=opt/org.vibeos.tripwire,string={value}"),
        ]
    })
}
pub fn nvme_bridge(env: &BTreeMap<String, String>) -> Result<&'static str> {
    match text(env, "VIBEOS_QEMU_NVME_BRIDGE") {
        "" | "0" => Ok(""),
        "1" | "true" | "yes" | "port" => Ok("port"),
        "segment" => Ok("segment"),
        x => Err(Error::new(format!(
            "VIBEOS_QEMU_NVME_BRIDGE={x:?}: expected port or segment"
        ))),
    }
}
pub fn xhci_interrupts(env: &BTreeMap<String, String>) -> Result<&'static str> {
    match text(env, "VIBEOS_QEMU_XHCI_MSI") {
        "" | "msix" => Ok("msix"),
        "msi" => Ok("msi"),
        "line" => Ok("line"),
        x => Err(Error::new(format!(
            "VIBEOS_QEMU_XHCI_MSI={x:?}: expected msix, msi or line"
        ))),
    }
}
pub fn boot_media_device_args(
    iso: &Path,
    ide_bus: Option<&str>,
    default: &str,
    env: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let selected = text(env, "VIBEOS_QEMU_BOOT_TRANSPORT");
    let transport = if selected.is_empty() {
        default
    } else {
        selected
    };
    let mut args = vec![
        "-drive".into(),
        format!(
            "id=cd0,if=none,format=raw,file={},readonly=on",
            iso.display()
        ),
    ];
    match transport{"ide-cd"=>args.extend(["-device".into(),format!("ide-cd,drive=cd0,bootindex=0{}",ide_bus.map(|s|format!(",bus={s}")).unwrap_or_default())]),"virtio-blk"=>args.extend(["-device".into(),"virtio-blk-pci,drive=cd0,bootindex=0".into()]),"usb-storage"|"usb-ehci"=>{let (controller,id)=if transport=="usb-storage"{("qemu-xhci","xhci")}else{("usb-ehci","ehci")};args.extend(["-device".into(),format!("{controller},id={id}"),"-device".into(),format!("usb-storage,bus={id}.0,drive=cd0,bootindex=0,removable=on")]);},_=>return Err(Error::new(format!("VIBEOS_QEMU_BOOT_TRANSPORT={transport:?}: expected ide-cd, virtio-blk, usb-storage or usb-ehci")))}
    Ok(args)
}
pub fn make_spcr(out: &Path, port: u64, irq: u32) -> Result<()> {
    let mut b = [0u8; 80];
    b[..4].copy_from_slice(b"SPCR");
    b[4..8].copy_from_slice(&80u32.to_le_bytes());
    b[8] = 2;
    b[10..16].copy_from_slice(b"VIBEOS");
    b[16..24].copy_from_slice(b"HOSTILE ");
    b[24..28].copy_from_slice(&1u32.to_le_bytes());
    b[28..32].copy_from_slice(b"VIBE");
    b[32..36].copy_from_slice(&1u32.to_le_bytes());
    b[40..44].copy_from_slice(&[1, 8, 0, 1]);
    b[44..52].copy_from_slice(&port.to_le_bytes());
    b[52] = 1;
    b[53] = u8::try_from(irq).map_err(|_| Error::new("SPCR IRQ exceeds byte"))?;
    b[54..58].copy_from_slice(&irq.to_le_bytes());
    b[58] = 7;
    b[60] = 1;
    b[64..68].fill(255);
    b[9] = 0u8.wrapping_sub(b.iter().copied().fold(0u8, u8::wrapping_add));
    fs::write(out, b)?;
    Ok(())
}
pub fn console_uart_args(
    arch: &str,
    channel: &str,
    env: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    if arch == "amd64-uefi" && hostile_hazards(env)?.contains("serial") {
        Ok(vec![
            "-device".into(),
            format!("isa-serial,chardev={channel},iobase=0x5000,irq=5"),
        ])
    } else {
        Ok(vec!["-serial".into(), format!("chardev:{channel}")])
    }
}
pub fn apply_stress_knobs(
    args: &[String],
    arch: &str,
    env: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let raw = text(env, "VIBEOS_QEMU_ICOUNT_SHIFT");
    let shift = if raw.is_empty() {
        None
    } else {
        let n = number(env, "VIBEOS_QEMU_ICOUNT_SHIFT", 0)?;
        if n > 10 {
            return Err("VIBEOS_QEMU_ICOUNT_SHIFT: QEMU accepts 0..10".into());
        }
        Some(n)
    };
    let raw_thread = text(env, "VIBEOS_TCG_THREAD");
    if !matches!(raw_thread, "" | "single" | "multi") {
        return Err("VIBEOS_TCG_THREAD must be single or multi".into());
    }
    if shift.is_none() && raw_thread.is_empty() {
        return Ok(args.into());
    }
    if shift.is_some() && args.iter().any(|x| x == "-icount") {
        return Err("stress icount and record/replay do not stack".into());
    }
    let source = if shift.is_some() && !arch.starts_with("arm64") {
        backend::rewrite_accel(args, "tcg", arch, env)
    } else {
        args.into()
    };
    let selected = if shift.is_some() {
        "single"
    } else {
        raw_thread
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < source.len() {
        if source[i] == "-accel"
            && source
                .get(i + 1)
                .is_some_and(|x| x.split(',').next() == Some("tcg"))
        {
            let props = source[i + 1]
                .split(',')
                .skip(1)
                .filter(|x| !x.starts_with("thread="))
                .collect::<Vec<_>>();
            let suffix = if props.is_empty() {
                String::new()
            } else {
                format!(",{}", props.join(","))
            };
            out.extend(["-accel".into(), format!("tcg,thread={selected}{suffix}")]);
            i += 2;
        } else {
            out.push(source[i].clone());
            i += 1;
        }
    }
    if let Some(n) = shift {
        out.extend(["-icount".into(), format!("shift={n},sleep=off")]);
    }
    Ok(out)
}
pub fn acpitable_relative(args: &[String]) -> Vec<String> {
    let mut out = args.to_vec();
    for i in 1..out.len() {
        if out[i - 1] == "-acpitable" && out[i].starts_with("file=") {
            let name = out[i].rsplit(['\\', '/']).next().unwrap();
            out[i] = format!("file={name}");
        }
    }
    out
}
fn media_for(
    root: &Path,
    arch: &str,
    work: &Path,
    env: &BTreeMap<String, String>,
) -> Result<PathBuf> {
    let supplied = text(env, "VIBEOS_BOOT_MEDIA");
    let mut media = if supplied.is_empty() {
        if arch.starts_with("arm64") {
            root.join("qemu/boot.img")
        } else {
            root.join(format!(
                "build-{}/livecd.iso",
                arch.split('-').next().unwrap()
            ))
        }
    } else {
        PathBuf::from(supplied)
    };
    if enabled(env, "VIBEOS_WRITABLE_ROOT") && !arch.starts_with("arm64") {
        if !supplied.is_empty() {
            return Err("VIBEOS_BOOT_MEDIA cannot boot with VIBEOS_WRITABLE_ROOT=1".into());
        }
        let script = root.join("tools/conformance/mk_writable_livecd.py");
        let base = root.join(format!("build-{}", arch.split('-').next().unwrap()));
        let python = env
            .get("VIBEOS_TOOLING_PYTHON")
            .map(String::as_str)
            .unwrap_or("python3");
        let (code, out, err) = backend::run_capture(
            python,
            &[
                script.to_string_lossy().into_owned(),
                base.to_string_lossy().into_owned(),
            ],
            Duration::from_secs(600),
        )?;
        if code != 0 {
            return Err(Error::new(format!(
                "writable livecd preparation failed: {out}\n{err}"
            )));
        }
        let candidate = base.join("livecd-rw.iso");
        if !candidate.is_file() {
            return Err(Error::new(format!(
                "writable media producer did not emit {}",
                candidate.display()
            )));
        }
        media = candidate;
    }
    if arch == "arm64-test" {
        media = media::testkernel_boot_img(&media, work)?;
    }
    if smp_count(env)?.is_some() {
        media = media::smp_selected_iso(&media, work)?;
    }
    if enabled(env, "VIBEOS_QEMU_STORTEST") {
        media = media::stortest_selected_iso(&media, work)?;
    }
    Ok(media)
}
fn firmware(src: &Path, out: &Path) -> Result<()> {
    if !src.is_file() {
        return Err(Error::new(format!(
            "firmware is missing: {}",
            src.display()
        )));
    }
    media::qcow2_from_raw(src, out)
}
pub fn gic_version(env: &BTreeMap<String, String>) -> Result<u64> {
    match number(env, "VIBEOS_QEMU_GIC", 2)? {
        v @ (2 | 3) => Ok(v),
        _ => Err("VIBEOS_QEMU_GIC must be 2 or 3".into()),
    }
}
pub const ECHO_PORTS: &[(&str, u16)] = &[
    ("netstack-outbound", 9999),
    ("tdi-selftest", 9998),
    ("usermode-winsock", 9997),
];
pub fn guestfwd_echo_opts(addr: &str) -> String {
    let mut ports = ECHO_PORTS.iter().map(|(_, p)| *p).collect::<Vec<_>>();
    ports.sort_unstable();
    ports
        .iter()
        .map(|p| format!(",guestfwd=tcp:{addr}:{p}-cmd:cat"))
        .collect()
}
pub fn machine_args(arch: &str, env: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let arm = arch.starts_with("arm64");
    let hazards = hostile_hazards(env)?;
    let mut machine: Vec<String> = if arm {
        vec![
            "-M".into(),
            "virt".into(),
            "-cpu".into(),
            "cortex-a72".into(),
            "-accel".into(),
            "tcg,thread=multi".into(),
            "-device".into(),
            "virtio-keyboard-device,id=arm64kbd".into(),
            "-device".into(),
            "virtio-mouse-device,id=arm64mouse".into(),
            "-global".into(),
            "virtio-mmio.force-legacy=false".into(),
        ]
    } else {
        vec![
            "-M".into(),
            "pc".into(),
            "-accel".into(),
            "kvm".into(),
            "-cpu".into(),
            "host".into(),
        ]
    };
    if arm {
        if gic_version(env)? == 3 {
            machine[1].push_str(",gic-version=3,its=on");
        }
        if enabled(env, "VIBEOS_QEMU_SMMU") {
            machine[1].push_str(",iommu=smmuv3");
        }
        if hazards.contains("el2") {
            machine[1].push_str(",virtualization=on");
        }
        if hazards.contains("a76") {
            machine[3] = "cortex-a76".into();
        }
        if !text(env, "VIBEOS_QEMU_CPU").is_empty() {
            machine[3] = text(env, "VIBEOS_QEMU_CPU").into();
        }
    } else if arch.starts_with("amd64-uefi") {
        if hazards.contains("q35") || hazards.contains("nops2") {
            machine[1] = format!(
                "{}{}",
                if hazards.contains("q35") { "q35" } else { "pc" },
                if hazards.contains("nops2") {
                    ",i8042=off"
                } else {
                    ""
                }
            );
        }
        if hazards.contains("cpu") {
            machine[5] = "max".into();
        }
    }
    Ok(machine)
}
fn boot_args(
    root: &Path,
    arch: &str,
    work: &Path,
    memory: &str,
    env: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let arm = arch.starts_with("arm64");
    let hazards = hostile_hazards(env)?;
    let mut machine = machine_args(arch, env)?;
    let image = media_for(root, arch, work, env)?;
    let mem = if hazards.contains("mem") && (arm || arch.starts_with("amd64-uefi")) {
        "6G".into()
    } else if enabled(env, "VIBEOS_WRITABLE_ROOT")
        && memory.ends_with('M')
        && memory[..memory.len() - 1].parse::<u64>().unwrap_or(2048) < 2048
    {
        "2G".into()
    } else {
        memory.to_owned()
    };
    if arm {
        let vars = work.join("efivars.qcow2");
        firmware(&root.join("qemu/efivars.fd"), &vars)?;
        machine.extend([
            "-m".into(),
            mem,
            "-drive".into(),
            format!(
                "if=pflash,format=raw,file={},readonly=on",
                root.join("qemu/QEMU_EFI.fd").display()
            ),
            "-drive".into(),
            format!("if=pflash,format=qcow2,file={}", vars.display()),
        ]);
        if enabled(env, "VIBEOS_QEMU_BOOT_NVME") {
            machine.extend([
                "-drive".into(),
                format!(
                    "file={},format=raw,if=none,id=bootnvme,snapshot=on,file.locking=off",
                    image.display()
                ),
                "-device".into(),
                "nvme,serial=VIBEOSBOOT0,drive=bootnvme,addr=0x2,bootindex=0".into(),
            ]);
        } else if nvme_bridge(env)? == "segment" {
            machine.extend([
                "-drive".into(),
                format!(
                    "file={},format=raw,if=none,id=bootvirtio,readonly=on,file.locking=off",
                    image.display()
                ),
                "-device".into(),
                "virtio-blk-pci,drive=bootvirtio,bus=pcie.0,addr=0x2".into(),
            ]);
        } else {
            machine.extend([
                "-drive".into(),
                format!(
                    "file={},format=raw,if=virtio,readonly=on,file.locking=off",
                    image.display()
                ),
            ]);
        }
        if !enabled(env, "VIBEOS_QEMU_HEADLESS") {
            machine.extend(["-device".into(), "ramfb".into()]);
        }
        machine.extend(["-display".into(), "none".into()]);
        return Ok(machine);
    }
    if arch != "amd64-bios" {
        let (code, vars_src, vars_name) = if arch.starts_with("i386") {
            (
                root.join("qemu/OVMF32_CODE.fd"),
                root.join("qemu/OVMF32_VARS.fd"),
                "ovmf32_vars.qcow2",
            )
        } else {
            (
                PathBuf::from("/usr/share/OVMF/OVMF_CODE_4M.fd"),
                PathBuf::from("/usr/share/OVMF/OVMF_VARS_4M.fd"),
                "ovmf_vars.qcow2",
            )
        };
        let vars = work.join(vars_name);
        firmware(&vars_src, &vars)?;
        machine.extend([
            "-m".into(),
            mem,
            "-drive".into(),
            format!("if=pflash,format=raw,file={},readonly=on", code.display()),
            "-drive".into(),
            format!("if=pflash,format=qcow2,file={}", vars.display()),
        ]);
        machine.extend(boot_media_device_args(
            &image,
            if hazards.contains("q35") {
                Some("ide.2")
            } else {
                None
            },
            "ide-cd",
            env,
        )?);
    } else {
        machine.extend(["-m".into(), mem]);
        machine.extend(boot_media_device_args(&image, None, "usb-storage", env)?);
    }
    if hazards.contains("novga") {
        machine.extend([
            "-vga".into(),
            "none".into(),
            "-device".into(),
            "ramfb".into(),
        ]);
    } else {
        machine.extend(["-vga".into(), "std".into()]);
    }
    if arch == "amd64-uefi" && hazards.contains("serial") {
        let spcr = work.join("spcr.bin");
        make_spcr(&spcr, 0x5000, 5)?;
        machine.extend(["-acpitable".into(), format!("file={}", spcr.display())]);
    }
    machine.extend(["-display".into(), "none".into(), "-no-reboot".into()]);
    Ok(machine)
}
fn stick(path: &Path, fat: u32, mib: u64, stortest: bool) -> Result<()> {
    if !path.exists() {
        let files = if stortest {
            vec![("MARKER.TXT".into(), b"vibeos usb e2e marker\r\n".to_vec())]
        } else {
            vec![(
                "VIBEOS.TXT".into(),
                b"hello from the usb stick\r\n".to_vec(),
            )]
        };
        media::make_usb_stick_image(
            path,
            fat,
            mib,
            if stortest { "VIBEUSB" } else { "VIBESTICK" },
            &files,
        )?;
    }
    Ok(())
}
pub fn temp_disk_args(
    arch: &str,
    work: &Path,
    env: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let arm = arch.starts_with("arm64");
    let mut disks = Vec::new();
    if !arm
        && arch != "amd64-bios"
        && !matches!(text(env, "VIBEOS_QEMU_TEMP_DISK"), "0" | "false" | "no")
    {
        let d = work.join("tempdisk.qcow2");
        media::make_temp_disk(&d)?;
        disks.extend([
            "-drive".into(),
            format!("file={},format=qcow2,if=none,id=tempdisk", d.display()),
            "-device".into(),
            "ide-hd,drive=tempdisk".into(),
        ]);
    }
    Ok(disks)
}
pub fn devices(
    arch: &str,
    work: &Path,
    env: &BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>)> {
    let arm = arch.starts_with("arm64");
    let i386 = arch.starts_with("i386");
    let scratch = work.join("scratch.qcow2");
    media::empty_qcow2(&scratch)?;
    let mut disks = vec![
        "-drive".into(),
        format!("file={},format=qcow2,if=none,id=scratch", scratch.display()),
    ];
    if !i386 {
        disks.extend([
            "-device".into(),
            if arm {
                "virtio-blk-device,drive=scratch"
            } else {
                "virtio-blk-pci,drive=scratch"
            }
            .into(),
        ]);
    }
    let mut common = Vec::new();
    let xhci = enabled(env, "VIBEOS_QEMU_XHCI")
        || (arch == "amd64-uefi" && hostile_hazards(env)?.contains("nops2"));
    let stortest = enabled(env, "VIBEOS_QEMU_STORTEST");
    let usb = stortest || enabled(env, "VIBEOS_QEMU_USB_DISK");
    if usb {
        let disk = work.join("usbdisk.qcow2");
        stick(&disk, 16, 64, stortest)?;
        disks.extend([
            "-drive".into(),
            format!("file={},format=qcow2,if=none,id=usbstor0", disk.display()),
        ]);
        if xhci {
            let ss = work.join("usbdisk-ss.qcow2");
            stick(
                &ss,
                32,
                number(env, "VIBEOS_QEMU_USB_STICK_SS_MIB", 64)?,
                false,
            )?;
            disks.extend([
                "-drive".into(),
                format!("file={},format=qcow2,if=none,id=usbstor1", ss.display()),
            ]);
        } else {
            common.extend([
                "-device".into(),
                "usb-ehci,id=xehci,addr=0xb".into(),
                "-device".into(),
                "usb-storage,bus=xehci.0,drive=usbstor0".into(),
            ]);
        }
    }
    if xhci {
        let suffix = match xhci_interrupts(env)? {
            "msix" => "",
            "msi" => ",msix=off,msi=on",
            _ => ",msix=off,msi=off",
        };
        common.extend([
            "-device".into(),
            format!("qemu-xhci,id=xhci,addr=0x8{suffix}"),
            "-device".into(),
            "usb-kbd,bus=xhci.0,id=ukbd".into(),
            "-device".into(),
            "usb-mouse,bus=xhci.0,id=umouse".into(),
        ]);
        if usb {
            common.extend([
                "-device".into(),
                "usb-hub,bus=xhci.0,port=3,id=uhub".into(),
                "-device".into(),
                "usb-storage,bus=xhci.0,port=3.1,drive=usbstor0,id=ustick".into(),
                "-device".into(),
                "usb-storage,bus=xhci.0,drive=usbstor1,id=ustick_ss".into(),
            ]);
        }
    }
    if enabled(env, "VIBEOS_QEMU_NVME") {
        let d = work.join("nvmedisk.qcow2");
        stick(&d, 16, 64, false)?;
        disks.extend([
            "-drive".into(),
            format!("file={},format=qcow2,if=none,id=nvme0", d.display()),
        ]);
        match nvme_bridge(env)? {
            "segment" => common.extend([
                "-device".into(),
                "pxb-pcie,id=pxb0,bus_nr=0x80,bus=pcie.0,addr=0x9".into(),
                "-device".into(),
                "pcie-root-port,id=nvmeport,chassis=1,slot=1,bus=pxb0,addr=0x0".into(),
                "-device".into(),
                "nvme,serial=VIBEOSNVME0,drive=nvme0,bus=nvmeport".into(),
            ]),
            "port" => common.extend([
                "-device".into(),
                "pcie-root-port,id=nvmeport,chassis=1,slot=1,addr=0x9".into(),
                "-device".into(),
                "nvme,serial=VIBEOSNVME0,drive=nvme0,bus=nvmeport".into(),
            ]),
            _ => common.extend([
                "-device".into(),
                "nvme,serial=VIBEOSNVME0,drive=nvme0,addr=0x9".into(),
            ]),
        }
    }
    if arm && enabled(env, "VIBEOS_QEMU_VIRTIO_MMIO_DISK") {
        let d = work.join("viommiodisk.qcow2");
        stick(&d, 16, 64, false)?;
        disks.extend([
            "-drive".into(),
            format!("file={},format=qcow2,if=none,id=viommio0", d.display()),
            "-device".into(),
            "virtio-blk-device,drive=viommio0".into(),
        ]);
    }
    if text(env, "VIBEOS_AHCI_CD") == "1" {
        for event in ["ahci_cmd_done", "ahci_irq_raise", "ahci_irq_lower"] {
            common.extend(["-trace".into(), event.into()]);
        }
        if arm {
            let d = work.join("ahcidisk.qcow2");
            stick(&d, 16, 64, false)?;
            disks.extend([
                "-drive".into(),
                format!("file={},format=qcow2,if=none,id=ahcihd", d.display()),
            ]);
            common.extend([
                "-device".into(),
                "ich9-ahci,id=ahci0,addr=0xa".into(),
                "-device".into(),
                "ide-hd,bus=ahci0.0,drive=ahcihd".into(),
                "-device".into(),
                "ide-cd,bus=ahci0.1".into(),
            ]);
        }
    }
    if i386 && enabled(env, "VIBEOS_QEMU_ISA_IDE") {
        let d = work.join("isaidedisk.qcow2");
        stick(&d, 16, 64, false)?;
        disks.extend([
            "-drive".into(),
            format!("file={},format=qcow2,if=none,id=isaidehd", d.display()),
        ]);
        common.extend([
            "-device".into(),
            "isa-ide,iobase=0x1e8,iobase2=0x3ee,irq=11,id=isaide".into(),
            "-device".into(),
            "ide-hd,bus=isaide.0,drive=isaidehd".into(),
        ]);
    }
    disks.extend(temp_disk_args(arch, work, env)?);
    Ok((disks, common))
}

pub fn timeout_policy(
    base: f64,
    silence_base: f64,
    env: &BTreeMap<String, String>,
) -> Result<(f64, f64)> {
    let scale = float(env, "VIBEOS_BOOT_TIMEOUT_SCALE", 1.0)?;
    let mut boot_timeout = base * smp_count(env)?.unwrap_or(1).max(1) as f64 * scale;
    let mut silence: f64 = silence_base * scale;
    if !text(env, "VIBEOS_QEMU_ICOUNT_SHIFT").is_empty() {
        boot_timeout = boot_timeout.max(900.0);
        silence = silence.max(600.0);
    }
    if smp_count(env)?.unwrap_or(1)
        > std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1) as u64
    {
        silence = silence.max(900.0);
    }
    Ok((boot_timeout, silence))
}
fn resolve_inner(root: &Path, request: &Value, prepare: bool) -> Result<VmConfig> {
    let arch = request["arch"]
        .as_str()
        .ok_or("VM requires explicit arch")?;
    let arch = match arch {
        "i386" => "i386-uefi",
        "amd64" => "amd64-uefi",
        v => v,
    };
    if ![
        "arm64",
        "arm64-test",
        "arm64-net",
        "i386-uefi",
        "i386-uefi-net",
        "amd64-uefi",
        "amd64-uefi-net",
        "amd64-bios",
    ]
    .contains(&arch)
    {
        return Err(Error::new(format!("unknown VM architecture {arch:?}")));
    }
    let mut env = std::env::vars().collect::<BTreeMap<String, String>>();
    if let Some(value) = request.get("env") {
        env.extend(serde_json::from_value::<BTreeMap<String, String>>(
            value.clone(),
        )?);
    }
    if let Some(memory) = request["memory"].as_str() {
        env.insert("VIBEOS_QEMU_MEMORY".into(), memory.into());
    }
    if request["temp_disk"].as_bool() == Some(false) {
        env.insert("VIBEOS_QEMU_TEMP_DISK".into(), "0".into());
    }
    let mut work = PathBuf::from(request["work_dir"].as_str().ok_or("VM work_dir missing")?);
    if work.is_relative() {
        work = root.join(work);
    }
    fs::create_dir_all(&work)?;
    let arm = arch.starts_with("arm64");
    let net = arch.ends_with("-net");
    let memory: String = if text(&env, "VIBEOS_QEMU_MEMORY").is_empty() {
        if arm { "1G" } else { "512M" }.into()
    } else {
        let m = text(&env, "VIBEOS_QEMU_MEMORY");
        if !regex::Regex::new(r"^[1-9][0-9]*[MG]$")?.is_match(m) {
            return Err("VIBEOS_QEMU_MEMORY must look like 2G or 1536M".into());
        }
        m.into()
    };
    let (mut args, disk_args, mut common) = if prepare {
        let args = boot_args(root, arch, &work, &memory, &env)?;
        let (disks, common) = devices(arch, &work, &env)?;
        (args, disks, common)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };
    let xhci = enabled(&env, "VIBEOS_QEMU_XHCI")
        || (arch == "amd64-uefi" && hostile_hazards(&env)?.contains("nops2"));
    if xhci {
        let mut out = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if args[i] == "-device"
                && args.get(i + 1).is_some_and(|v| {
                    v.starts_with("virtio-keyboard-device") || v.starts_with("virtio-mouse-device")
                })
            {
                i += 2;
            } else {
                out.push(args[i].clone());
                i += 1;
            }
        }
        args = out;
    }
    common.extend(tripwire_args(&env)?);
    let smp = smp_count(&env)?;
    if let Some(n) = smp {
        common.extend(["-smp".into(), n.to_string()]);
    }
    let extra = text(&env, "VIBEOS_QEMU_EXTRA_ARGS");
    if !extra.is_empty() {
        common
            .extend(shlex::split(extra).ok_or("VIBEOS_QEMU_EXTRA_ARGS has invalid shell quoting")?);
    }
    let (boot_timeout, silence) = timeout_policy(if arm { 180.0 } else { 360.0 }, 120.0, &env)?;
    let live = request
        .get("serial_log")
        .or_else(|| request.get("live_log"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env.get("VIBEOS_ARTIFACT_DIR")
                .filter(|s| !s.trim().is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| work.clone())
                .join(format!("{arch}_serial.log"))
        });
    let marker = if arch == "arm64-test" {
        "TESTKERNEL READY"
    } else {
        "[EXPLORER] _tWinMain entered"
    };
    let settle = if arch == "arm64-test" {
        0.0
    } else if net && arm {
        30.0
    } else if net {
        2.0
    } else {
        15.0
    };
    let config = VmConfig {
        arch: arch.into(),
        qemu_bin: if arm {
            "qemu-system-aarch64"
        } else if arch.starts_with("i386") {
            "qemu-system-i386"
        } else {
            "qemu-system-x86_64"
        }
        .into(),
        arch_args: args,
        common_args: common,
        disk_args,
        network_args: Vec::new(),
        serial_args: console_uart_args(arch, "ser0", &env)?,
        backend: request["backend"].as_str().unwrap_or("native").into(),
        windows: request.get("windows").cloned().unwrap_or_else(|| json!({})),
        work_dir: work,
        live_log: live,
        boot_marker: marker.into(),
        boot_timeout,
        qmp_timeout: if arm { 60.0 } else { 10.0 },
        min_boot_bytes: if arm {
            5000
        } else if arch.starts_with("i386") {
            500
        } else {
            1000
        },
        desktop_settle_seconds: settle,
        hang_silence_seconds: silence,
        max_lifetime: number(&env, "VIBEOS_QEMU_MAX_LIFETIME", 1800)? as f64,
        env: env.clone(),
        owner_pid: request["owner_pid"].as_u64().unwrap_or(0) as u32,
        gdb: request["gdb"].as_bool().unwrap_or(false)
            || matches!(text(&env, "VIBEOS_QEMU_GDB"), "1" | "true" | "yes" | "wait"),
        gdb_wait: request["gdb_wait"].as_bool().unwrap_or(false)
            || text(&env, "VIBEOS_QEMU_GDB") == "wait",
        gdb_port: request["gdb_port"].as_u64().unwrap_or(0) as u16,
        serial_file: request["serial_file"].as_bool().unwrap_or(false),
        memory,
        required_milestones: if arm {
            vec![
                "[KERNEL] KiSystemStartup entered".into(),
                "[PS] PspUserThreadStartup: returning".into(),
                "[SMSS] _main entered".into(),
                "[CSRSS] _main entered".into(),
                "[WINLOGON] WinMain entered".into(),
                "[EXPLORER] _tWinMain entered".into(),
                "shell ready".into(),
            ]
        } else {
            vec![
                "[SMSS] _main entered".into(),
                "[CSRSS] _main entered".into(),
                "[WINLOGON] WinMain entered".into(),
                "[EXPLORER] _tWinMain entered".into(),
                "shell ready".into(),
            ]
        },
    };
    Ok(config)
}

pub fn resolve(root: &Path, request: &Value) -> Result<VmConfig> {
    resolve_inner(root, request, false)
}
pub fn prepare(root: &Path, config: &VmConfig) -> Result<VmConfig> {
    resolve_inner(root, &serde_json::to_value(config)?, true)
}

#[derive(Default)]
pub struct LaunchPorts {
    used: BTreeSet<u16>,
}
impl LaunchPorts {
    pub fn reserve(&mut self, port: u16) -> bool {
        port != 0 && self.used.insert(port)
    }
    pub fn ports(&self) -> Vec<u16> {
        self.used.iter().copied().collect()
    }
    pub fn choose(&mut self, mut picker: impl FnMut() -> Result<u16>) -> Result<u16> {
        for _ in 0..64 {
            let port = picker()?;
            if self.reserve(port) {
                return Ok(port);
            }
        }
        Err("could not allocate a unique launch port after 64 candidates".into())
    }
}
pub fn network_args(arch: &str, windows: bool) -> Result<(Vec<String>, Value)> {
    network_args_with(arch, windows, &mut LaunchPorts::default())
}
pub fn network_args_with(
    arch: &str,
    windows: bool,
    ports: &mut LaunchPorts,
) -> Result<(Vec<String>, Value)> {
    if !arch.ends_with("-net") {
        return Ok((Vec::new(), json!({})));
    }
    let mut pick = || {
        ports.choose(|| {
            if windows {
                backend::windows_free_tcp_port()
            } else {
                Ok(TcpListener::bind(("127.0.0.1", 0))?.local_addr()?.port())
            }
        })
    };
    let echo = pick()?;
    let tdi = pick()?;
    let payload = pick()?;
    Ok((vec!["-device".into(),"virtio-net-pci,disable-legacy=on,netdev=net0,id=vnet0".into(),"-netdev".into(),format!("user,id=net0,hostfwd=tcp:127.0.0.1:{echo}-:7777,hostfwd=tcp:127.0.0.1:{tdi}-:8888,hostfwd=tcp:127.0.0.1:{payload}-:8080{}",guestfwd_echo_opts("10.0.2.100"))],json!({"echo_hostfwd_port":echo,"tdi_srv_hostfwd_port":tdi,"payload_hostfwd_port":payload,"ECHO_GUEST_PORT":7777,"TDI_SRV_GUEST_PORT":8888,"PAYLOAD_GUEST_PORT":8080,"GUESTFWD_ADDR":"10.0.2.100"})))
}

pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    let env: BTreeMap<String, String> = a
        .get("env")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_else(|| std::env::vars().collect());
    let args = || -> Result<Vec<String>> { Ok(serde_json::from_value(a["args"].clone())?) };
    Ok(match request["op"].as_str().unwrap_or("") {
        "vm_config" | "vm_arch_config" => {
            let mut v = serde_json::to_value(resolve(root, a)?)?;
            if request["op"] == "vm_arch_config" {
                v["boot_timeout"] =
                    json!(if a["arch"].as_str().unwrap_or("").starts_with("arm64") {
                        180.0
                    } else {
                        360.0
                    });
                v["hang_silence_seconds"] = json!(120.0);
            }
            v["desktop_milestone"] = json!("shell ready");
            v["machine_args"] = json!(machine_args(
                a["arch"].as_str().ok_or("missing arch")?,
                &BTreeMap::new()
            )?);
            v
        }
        "vm_prepare_config" => serde_json::to_value(resolve_inner(root, a, true)?)?,
        "vm_smp_selected_iso" => json!(media::smp_selected_iso(
            Path::new(a["iso"].as_str().ok_or("missing iso")?),
            Path::new(a["work_dir"].as_str().ok_or("missing work_dir")?)
        )?),
        "vm_stortest_selected_iso" => json!(media::stortest_selected_iso(
            Path::new(a["iso"].as_str().ok_or("missing iso")?),
            Path::new(a["work_dir"].as_str().ok_or("missing work_dir")?)
        )?),
        "vm_testkernel_boot_img" => json!(media::testkernel_boot_img(
            Path::new(a["src"].as_str().ok_or("missing src")?),
            Path::new(a["work_dir"].as_str().ok_or("missing work_dir")?)
        )?),
        "vm_temp_disk_args" => json!(temp_disk_args(
            a["arch"].as_str().ok_or("missing arch")?,
            Path::new(a["work_dir"].as_str().ok_or("missing work_dir")?),
            &env
        )?),
        "vm_sparse_image" => {
            let bytes: Vec<u8> = serde_json::from_value(a["data"].clone())?;
            media::sparse_image(Path::new(a["path"].as_str().ok_or("missing path")?), &bytes)?;
            Value::Null
        }
        "vm_make_temp_disk" => {
            media::make_temp_disk(Path::new(a["path"].as_str().ok_or("missing path")?))?;
            Value::Null
        }
        "vm_make_usb_stick_image" => {
            let files: Vec<(String, Vec<u8>)> = serde_json::from_value(a["files"].clone())?;
            media::make_usb_stick_image(
                Path::new(a["path"].as_str().ok_or("missing path")?),
                a["fat"].as_u64().unwrap_or(16) as u32,
                a["mib"].as_u64().unwrap_or(64),
                a["label"].as_str().unwrap_or("VIBEOSUSB"),
                &files,
            )?;
            Value::Null
        }
        "vm_echo_ports" => json!(ECHO_PORTS
            .iter()
            .map(|(name, port)| (*name, *port))
            .collect::<BTreeMap<_, _>>()),
        "vm_guestfwd_echo_opts" => json!(guestfwd_echo_opts(
            a["addr"].as_str().ok_or("missing guestfwd address")?
        )),
        "vm_env_options" => {
            json!({"smp":smp_count(&env)?,"gic":gic_version(&env)?,"smmu":enabled(&env,"VIBEOS_QEMU_SMMU"),"cpu":env.get("VIBEOS_QEMU_CPU"),"boot_nvme":enabled(&env,"VIBEOS_QEMU_BOOT_NVME"),"headless":enabled(&env,"VIBEOS_QEMU_HEADLESS"),"stortest":enabled(&env,"VIBEOS_QEMU_STORTEST"),"writable_root":enabled(&env,"VIBEOS_WRITABLE_ROOT"),"icount_shift":if text(&env,"VIBEOS_QEMU_ICOUNT_SHIFT").is_empty(){None}else{let v=number(&env,"VIBEOS_QEMU_ICOUNT_SHIFT",0)?;if v>10{return Err("icount shift must be 0..10".into());}Some(v)},"tcg_thread":match text(&env,"VIBEOS_TCG_THREAD"){""=>None,"single"=>Some("single"),"multi"=>Some("multi"),_=>return Err("VIBEOS_TCG_THREAD must be single or multi".into())},"timeout_scale":float(&env,"VIBEOS_BOOT_TIMEOUT_SCALE",1.0)?})
        }
        "vm_tripwire_args" => json!(tripwire_args(&env)?),
        "vm_make_spcr" => {
            make_spcr(
                Path::new(a["path"].as_str().ok_or("missing path")?),
                a["port"].as_u64().ok_or("missing port")?,
                a["irq"].as_u64().ok_or("missing irq")? as u32,
            )?;
            Value::Null
        }
        "vm_nvme_bridge" => json!(nvme_bridge(&env)?),
        "vm_machine_args" => json!(machine_args(
            a["arch"].as_str().ok_or("missing arch")?,
            &env
        )?),
        "vm_xhci_interrupts" => json!(xhci_interrupts(&env)?),
        "vm_unique_ports" => {
            let candidates: Vec<u16> = serde_json::from_value(a["candidates"].clone())?;
            if candidates.is_empty() {
                return Err("empty port candidates".into());
            }
            let mut ports = LaunchPorts::default();
            let mut at = 0;
            let mut result = Vec::new();
            for _ in 0..a["count"].as_u64().unwrap_or(1) {
                result.push(ports.choose(|| {
                    let value = candidates[at.min(candidates.len() - 1)];
                    at += 1;
                    Ok(value)
                })?);
            }
            json!(result)
        }
        "vm_smp_boot_timeout" => json!(
            a["base"].as_f64().ok_or("missing base timeout")?
                * a["count"].as_u64().unwrap_or(1).max(1) as f64
        ),
        "vm_smp_count" => json!(smp_count(&env)?),
        "vm_hostile_hazards" => json!(hostile_hazards(&env)?),
        "vm_console_uart_args" => json!(console_uart_args(
            a["arch"].as_str().ok_or("missing arch")?,
            a["chardev"].as_str().unwrap_or("ser0"),
            &env
        )?),
        "vm_boot_media_device_args" => json!(boot_media_device_args(
            Path::new(a["iso"].as_str().ok_or("missing iso")?),
            a["ide_bus"].as_str(),
            a["default_transport"].as_str().unwrap_or("ide-cd"),
            &env
        )?),
        "vm_apply_stress_knobs" => json!(apply_stress_knobs(
            &args()?,
            a["arch"].as_str().ok_or("missing arch")?,
            &env
        )?),
        "vm_acpitable_relative" => json!(acpitable_relative(&args()?)),
        _ => return Err("unknown VM configuration operation".into()),
    })
}
