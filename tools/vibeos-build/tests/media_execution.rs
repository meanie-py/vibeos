use serde_json::{json, Value};
use vibeos_build::media_manifest as m;
#[test]
fn assemble_arm64_with_fake_ramdisk_and_stamp() {
    let base = std::env::temp_dir().join(format!("manifest-assemble-test-{}", std::process::id()));
    if base.exists() {
        std::fs::remove_dir_all(&base).unwrap();
    }
    let root = base.join("root");
    let out = base.join("out");
    for dir in [
        root.join("boot/media"),
        root.join("build-arm64"),
        root.join("qemu"),
    ] {
        std::fs::create_dir_all(dir).unwrap();
    }
    assert!(std::process::Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(&root)
        .status()
        .unwrap()
        .success());
    std::fs::write(root.join("uefildr.efi"), b"efi").unwrap();
    std::fs::write(root.join("boot/media/arm64.json"),r#"{"config":"Debug","cds":{"livecd":{"command":"","entries":[["efi/boot/bootaa64.efi","repo:uefildr.efi"]]}}}"#).unwrap();
    std::fs::write(root.join("qemu/mkramdisk.sh"),"while [ $# -gt 0 ]; do if [ \"$1\" = --out-dir ]; then shift; out=$1; fi; shift; done\nprintf fat > \"$out/livecd.img\"\n").unwrap();
    let image = m::assemble("arm64", "livecd", &out, None, &root).unwrap();
    assert_eq!(std::fs::read(&image).unwrap(), b"fat");
    let stamp: Value =
        serde_json::from_slice(&std::fs::read(out.join("livecd.provenance.json")).unwrap())
            .unwrap();
    assert_eq!(stamp["arch"], "arm64");
    assert_eq!(stamp["source"], m::source_hash(&root).unwrap());
    assert_eq!(
        std::fs::read_to_string(out.join("livecd.lst")).unwrap(),
        format!(
            "efi/boot/bootaa64.efi={}\n",
            root.join("uefildr.efi").display()
        )
    );
    std::fs::remove_dir_all(base).unwrap();
}
#[test]
fn recab_uses_cells_and_both_commands() {
    let base = std::env::temp_dir().join(format!("manifest-recab-test-{}", std::process::id()));
    if base.exists() {
        std::fs::remove_dir_all(&base).unwrap();
    }
    let root = base.join("root");
    let build = root.join("build-amd64");
    let cells = base.join("cells");
    let out = base.join("out");
    for d in [
        root.join("boot"),
        build.join("host-tools/bin"),
        cells.join("base/m"),
        out.clone(),
    ] {
        std::fs::create_dir_all(d).unwrap();
    }
    std::fs::write(root.join("boot/head.in"), ".Set X=1\n").unwrap();
    std::fs::write(cells.join("base/m/m.exe"), "m").unwrap();
    let tool = build.join("host-tools/bin/cabman");
    std::fs::write(
        &tool,
        "#!/bin/sh\nprintf x > \"$1/reactos.inf\"\nprintf x > \"$1/reactos.cab\"\n",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).unwrap();
    let man = json!({"cab":{"dff":"repo:boot/head.in","inf_command":"{BUILD}/host-tools/bin/cabman {BUILD}/boot/bootdata/packages","cab_command":"{BUILD}/host-tools/bin/cabman {BUILD}/boot/bootdata/packages","files":[["cell:base/m/m.exe","2"]]}});
    let made = m::recab(&man, &out, &build, &root, &cells).unwrap();
    assert_eq!(made[m::CAB], out.join("cab/reactos.cab"));
    assert_eq!(
        std::fs::read_to_string(out.join("cab/reactos.dff")).unwrap(),
        format!(".Set X=1\n\"{}\" 2\n", cells.join("base/m/m.exe").display())
    );
    std::fs::remove_dir_all(base).unwrap();
}
