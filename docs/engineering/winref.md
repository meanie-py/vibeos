# winref: real Windows 11 as the conformance oracle

ReactOS's rostests encode what someone believed Server 2003 did. For the Windows 10/11
surface (`modern-windows-compat-roadmap.md`, Rung 0) the expected result comes from Windows
itself: build a test with our toolchain, run it on a real Windows 11, and grade the vibeos
guest against that. `vibeos-cli winref` (`tests/qemu/cli/winref.py`, guest scripts in
`qemu/winref/`) is the VM; commands are in `tests/qemu/cli/README.md`.

## Shape

- **Host:** the Windows host's QEMU (`C:\Program Files\qemu`), images on `D:\vibeos-winref`.
  No network device: no updates, no account, no drift.
- **Base image:** one unattended install (`autounattend.xml` + first-logon `setup.ps1`):
  test signing on, Windows Update off, autologon as `winref`, and an elevated logon task
  running `C:\winref\boot.ps1`.
- **Run:** a qcow2 overlay of the base boots with the payload directory as a CD. The
  runner copies it to `C:\winref\payload` (tests write beside their own .exe), runs
  `run.cmd` elevated, relays every line to COM1 (`OUT: ...`), prints `WINREF: exit=<n>`,
  powers off. The overlay is discarded. ~1.5 min on WHPX.
- **Maintenance:** `winref maint <script.ps1>` boots the base itself with the script as a
  payload; `--keys` types it into Win+R over QMP when the runner itself is broken.
  Snapshot first. `post-install` is the pristine result of the install.
- **Evidence:** `winref logs` reads `C:\Windows\Panther\UnattendGC\setupact.log`, the
  runner task's XML and `C:\winref\` off a stopped disk (7-Zip + `qemu-img dd`; no nbd,
  no admin). Every run keeps a downscaled screenshot timeline in `shots/`.

Reference so far: Windows 11 Pro 26H2, build 26300.9457, from Microsoft's public
multi-edition ISO, unactivated (the generic Pro install key selects the edition).

## What it cost to get here (each is fixed in the tool)

| Symptom | Cause |
|---|---|
| `WHPX: Unexpected VP exit code 4` mid-Setup, after `Ignoring request for interrupt vector 0` (AMD Ryzen 9 7950X3D) | QEMU's WHPX on this host; not the CPU model, USB or irqchip. Install under TCG (~2 h); installed systems run fine on WHPX |
| Setup's account `winref` never became an admin (`0x8007056b`), so FirstLogonCommands never ran | computer name == account name: the group add resolved the MACHINE. Computer is `WINREF-VM` |
| FirstLogonCommands skipped | deprecated `SkipMachineOOBE`/`SkipUserOOBE` (removed) |
| Setup hung forever | `schtasks /create /ru` without `/rp` prompts for a password |
| VM sat at the desktop after setup | `Stop-Computer` from FirstLogonCommands came back as a RESTART; the launcher presses the ACPI power button once the guest prints its finish line |
| Runner never fired | task XML in `System32\Tasks` but unknown to Task Scheduler; setup and maint now verify `Get-ScheduledTask`. Not a battery issue: Windows reports line=Online |
| 20 GB snapshot copy: `ENOMEM` | DrvFS `sendfile`; copies go through Windows `Copy-Item` |

## What the first payload found

kernel32 `sync`/`version` and ntdll `rtl` winetests, built by our toolchain, run green on
Windows 11 only after fixing three things on OUR side, none of them Windows behaviour:

- **59 `sync` failures, error 161:** our build writes `__FILE__` as
  `\modules\rostests\...` (clang + `-ffile-prefix-map` on a Windows target), and object
  names may not contain `\`. The test's MSVC-only guard now covers every ReactOS build.
- **31 `sync` failures in `NtWaitForMultipleObjects`:** the test declared the 4-byte
  `WAIT_TYPE` as `BOOLEAN`; on x64 the kernel read undefined upper bits of r8.
- **14 `version` failures:** the test writes a manifest beside its .exe; the payload CD is
  read-only, so it measured an unmanifested process. The runner now copies payloads.

Each would have failed the same way on vibeos and been filed as an OS bug.
