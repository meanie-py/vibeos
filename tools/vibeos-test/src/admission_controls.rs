//! Native file-lock controls replacing Python VM private admission monkeypatches.
use crate::admission::Admission;
use std::{collections::BTreeMap, path::Path};
fn environment(root: &Path) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "VIBEOS_QEMU_LOCK_DIR".into(),
            root.join("locks").to_string_lossy().into_owned(),
        ),
        (
            "VIBEOS_HOSTLOCK_DIR".into(),
            root.join("host").to_string_lossy().into_owned(),
        ),
        (
            "VIBEOS_BUILD_POLICY_DIR".into(),
            root.join("policy").to_string_lossy().into_owned(),
        ),
        ("VIBEOS_QEMU_QUEUE_TIMEOUT".into(), "0".into()),
        ("VIBEOS_HOST_LOCK_TIMEOUT".into(), "0".into()),
        ("VIBEOS_QEMU_MAX_CONCURRENT".into(), "4".into()),
        ("VIBEOS_HOST_CPU_LEASES".into(), "1".into()),
        ("VIBEOS_QEMU_CAP_AMD64".into(), "1".into()),
    ])
}
#[test]
fn whpx_same_arch_serializes_while_default_windows_tcg_does_not() {
    let root = tempfile::tempdir().unwrap();
    let mut env = environment(root.path());
    env.insert("VIBEOS_HOST_BROKER".into(), "0".into());
    let first = Admission::acquire(
        root.path(),
        "amd64-uefi",
        "windows-host",
        "whpx",
        root.path(),
        &env,
    )
    .unwrap();
    let second = Admission::acquire(
        root.path(),
        "amd64-uefi",
        "windows-host",
        "whpx",
        root.path(),
        &env,
    );
    assert!(matches!(second,Err(ref e)if e.message.contains("same-arch")));
    drop(first);
    let first = Admission::acquire(
        root.path(),
        "amd64-uefi",
        "windows-host",
        "tcg",
        root.path(),
        &env,
    )
    .unwrap();
    let second = Admission::acquire(
        root.path(),
        "amd64-uefi",
        "windows-host",
        "tcg",
        root.path(),
        &env,
    )
    .unwrap();
    assert!(first.receipt()["arch_slot"].is_null() && second.receipt()["arch_slot"].is_null());
}
#[test]
fn native_tcg_retains_cpu_lease_and_timeout_refuses_unadmitted_boot() {
    let root = tempfile::tempdir().unwrap();
    let env = environment(root.path());
    let first =
        Admission::acquire(root.path(), "arm64", "native", "tcg", root.path(), &env).unwrap();
    assert!(first.receipt()["cpu_lease"].as_str().is_some());
    let second = Admission::acquire(root.path(), "arm64", "native", "tcg", root.path(), &env);
    assert!(
        second.is_err(),
        "a full CPU pool must never launch an unlocked VM"
    );
    drop(first);
    // A sibling test forking (any `pre_exec` spawn) while `first` was held gives its child
    // a copy of the lease's open file description, and the flock lives until that child
    // execs and CLOEXEC closes it - milliseconds later. Real callers wait up to an hour;
    // a 0s re-acquire lost that race 3 times in 8 full-suite runs. Allow a short wait.
    let mut env = env;
    env.insert("VIBEOS_HOST_LOCK_TIMEOUT".into(), "5".into());
    if let Err(e) = Admission::acquire(root.path(), "arm64", "native", "tcg", root.path(), &env) {
        panic!("re-acquire after release failed: {} ({:?})", e.message, e.kind);
    }
}
#[test]
fn kvm_and_explicit_broker_off_do_not_consume_tcg_leases() {
    let root = tempfile::tempdir().unwrap();
    let mut env = environment(root.path());
    let kvm = Admission::acquire(
        root.path(),
        "amd64-uefi",
        "native",
        "kvm",
        root.path(),
        &env,
    )
    .unwrap();
    assert!(kvm.receipt()["cpu_lease"].is_null());
    drop(kvm);
    env.insert("VIBEOS_HOST_BROKER".into(), "0".into());
    let tcg = Admission::acquire(root.path(), "arm64", "native", "tcg", root.path(), &env).unwrap();
    assert!(tcg.receipt()["cpu_lease"].is_null());
}
