# docs/engineering/archive — closed & obsolete engineering docs

Docs here are **kept for the record, not for active guidance.** They fall into three
buckets:

- **Obsolete** — superseded by a different approach, abandoned, or describing a retired
  mechanism (e.g. the `continue-rust-port` workflow, a concluded/negative PoC, a plan whose
  work is all built): `beyond-ntoskrnl-targets`, `translation-banking-roadmap`,
  `rtl-flip-playbook`, `cargo-drive-unification-plan` (BLOCKED negative PoC),
  `qemu-cli-port-plan` (the CLI shipped), `win32k-pretranslation-audit` (concluded PoC).
- **Closed** — the work is done + landed on `master`, no open items:
  `advapi32/secur32/gdi32-translation-audit`, `amd64-boot-stall-investigation`,
  `arm64-clock-and-lpcexitthread-findings`, `classpnp-cohab-link-gap`, `rustconv-hal-target`,
  `windows-host-qemu-whpx-spike` (shelved), `leaf-driver-translation-plan`,
  `kernel32-shared-vista-rust-plan`, `acpica-ffi-plan`.
- **Harvested** — closed audits whose reusable technique was **lifted into the durable
  reference docs** before archiving. Their transferable content now lives in
  `../translation-debugging-field-manual.md` (rtl bug-catalog + byte-diff method; freeldr
  UEFI-boot recipe) and `../rust-in-kernel.md` (user32 cohab link-trap catalog, hal ABI
  contract, ntdll kernel↔user boundary, RPC-server cohab + difftest, cmake→cargo link
  cutover): `rtl/freeldr/user32/hal/ntdll-translation-audit`, `lsa-sam-server-translation-plan`,
  `ntdll-cargo-drive-audit`, `shell32-translation-audit`.

**For current state** see `../rust-port-status.md` (live as-built), the active
`*-translation-audit.md` set (ntoskrnl/win32k/driver/static-libs/userland/smss-csrss/arm64),
and the evergreen reference docs (field-manual, rust-in-kernel, rust-kernel-patterns,
kernel-debugging-infra). Everything here is recoverable via git history regardless.
