# ============================================================================
# nvme.sys — NVM Express driver (i386, amd64, arm64), cargo-driven link
# ============================================================================
# pci.sys enumerates the controller (class 01/08/02) and the critical device
# database (boot/bootdata/hivesys.inf) binds this driver to PCI\CC_010802.
# The driver is Rust with no C original: sdk/lib/rust/nvme-rust (body, on the
# shared scsidisk-rust SCSI layer) + nvme-cargo (link).
add_cargo_module(nvme nvme-cargo nvme.sys KIND kerneldll
    BODY_CRATE nvme-rust DESTINATION reactos/system32/drivers)
