# Host clock stability on WSL

Use the front door to measure the clock rather than trusting a time service's
"synchronized" flag:

```sh
./vibeos-cli host-clock --seconds 65
./vibeos-cli host-clock --fix-wsl --seconds 65
```

The probe records paired `CLOCK_REALTIME`/monotonic readings in `samples.json` and
reports changes in their offset. A step over 50 ms fails the check. Pairs stalled
more than 2 ms between reads remain in the evidence but are excluded from step
classification. This measures discontinuities, **not accuracy against UTC**.

`--fix-wsl` refuses to change a native host or a WSL instance without active
Hyper-V implicit synchronization. It disables and stops `systemd-timesyncd`,
leaving Hyper-V as the WSL clock authority, then records ten seconds of settling
and the requested observation. The prior service state and actual command result
are retained in `summary.json`. It does not change Windows time configuration.

Ubuntu documents why running a separate NTP clock authority inside modern WSL
can conflict with Hyper-V's forced host samples, and recommends disabling
`systemd-timesyncd` for this setup:
[Ubuntu WSL time synchronization](https://ubuntu.com/wsl/docs/stable/explanation/time-sync/).

## This host, 2026-09-19

Observed before the change:

- WSL kernel `6.18.33.2-microsoft-standard-WSL2`, `timesync_implicit=Y`.
- Linux `systemd-timesyncd` active/enabled, polling every 32 seconds.
- Independent probe observed **-1226.672 ms** and **-1225.949 ms** steps at
  24.635 and 56.812 seconds, before stopping the service.
- Windows Time was stopped/manual, configured for `time.windows.com,0x9`.
  `w32tm /stripchart` measured Windows about **0.707 seconds ahead** of that
  server; this is separate evidence from Linux's NTP status.

At 03:05:53, `systemctl disable --now systemd-timesyncd.service` succeeded;
`journalctl -u systemd-timesyncd` independently records the stop. Hyper-V remained
enabled. After ten seconds of settling, the next 65 seconds contained no steps
over 50 ms; the largest measured step was **0.004789 ms**.

Evidence under `/tmp/vibeos-cli/`:

- `20260919-030453-uy9h2uuh-host-clock-host/samples.json` (the two pre-change steps;
  its final few seconds overlap the service change).
- `20260919-030553-0xupg_q1-host-clock-host/{summary,samples,settling}.json`
  (command, before/after service state and independent post-change measurement).

At 10:44, an approved Windows UAC operation set Windows Time to Automatic and
successfully synchronized its existing `time.windows.com,0x9` source. Windows
Time-Service events 37 and 35 independently confirm valid NTP data and source
selection. Subsequent `w32tm /stripchart` samples were within 4.2–7.1 ms of that
server. The operation log is
`C:/Users/Lambda/AppData/Local/VibeOS/host-clock/windows-time-20260919-uac.json`.

The service initially stopped again despite Automatic startup. At 10:57, an
approved UAC retry applied
[Microsoft's workgroup startup fix](https://learn.microsoft.com/en-us/troubleshoot/windows-client/active-directory/w32time-not-start-on-workgroup):
confirmed this machine is not domain joined, backed up and removed the service
triggers, set Automatic startup, started the service, and synchronized again.
The existing delayed-start setting remains enabled. Independent `sc` queries
after the observation below confirmed Running, Automatic (Delayed), and no
registered service triggers. Startup after reboot has not been tested.

Windows-side evidence under `C:/Users/Lambda/AppData/Local/VibeOS/host-clock/`:

- `windows-time-9520e7452560.json`: successful elevated repair and synchronization
  at 10:57:06 against the unchanged `time.windows.com,0x9` source.
- `trigger-info-9520e7452560.reg`: original service-trigger backup.

Independent `w32tm /stripchart` samples at 10:57:46 and 10:57:51 measured Windows
22.6–22.8 ms ahead of the server; the intervening request timed out.
The post-repair WSL probe passed 65 seconds with a largest measured step of
**0.00528 ms**, while Linux timesyncd remained inactive/disabled and Hyper-V
implicit synchronization remained enabled. Evidence:
`/tmp/vibeos-cli/20260919-105746-cr7e0ymi-host-clock-host/{summary,samples}.json`.

The repair is persistent across normal systemd starts. If intentionally returning
to a Linux-controlled clock, first arrange that Hyper-V no longer forces host
samples; blindly re-enabling Linux NTP recreates this conflict. The prior local
service state was enabled/active.

Guest snapshot clocks are a different layer. Checkpoint verification separately
proves guest InterruptTime rollback and exclusion of host pause time; existing
record/replay clock-event problems are not fixed by this host change.
