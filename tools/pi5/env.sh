# Sourced by the tools/pi5 scripts: the Pi 5 bench is the Pi 4 bench's machinery
# (tools/pi4) pointed at the other board's plug, Pico, capture card and lease.
# Each value can be overridden from the environment.
export VIBEOS_BENCH_BOARD=pi5
export PI4_BENCH=${PI5_BENCH:-/mnt/d/home/vibeos-rpi5-setup}
# 2026-10-18: the Pi 5's Pico and plug moved to the Pi 4 (tools/pi4's defaults now). The
# Pi 5 has neither until they are set here again. Unset, they name a host that cannot
# resolve: what needs them fails, and nothing falls through to tools/pi4's defaults,
# which would reach into the Pi 4's bench. (Scripts that need neither, like stage.sh's
# freeldr.ini, still run.)
PI5_NO_BENCH=none.invalid
export PI4_PICO_HOST=${PI5_PICO_HOST:-$PI5_NO_BENCH}
export PI4_HDMI_VID=${PI5_HDMI_VID:-534d}
# Its plug was shellyplugusg4-acebe6f18e28, the Pi 4's since 2026-10-18.
export PI4_PLUG=${PI5_PLUG:-$PI5_NO_BENCH}
[ -n "${PI5_PLUG_PASS:-}" ] && export PI4_PLUG_PASS=$PI5_PLUG_PASS
PI5_HOST=${PI5_HOST:-vibeos-pi5}
PI5_SSH_TARGET=${PI5_SSH_USER:-bench}@$PI5_HOST

# The stager's host key changes whenever a card is rebuilt (cloud-init makes a
# new one), so it lives in the bench's own known_hosts, not ~/.ssh's.
pi5_ssh() {
    mkdir -p "$PI4_BENCH"
    ssh -o BatchMode=yes -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new \
        -o UserKnownHostsFile="$PI4_BENCH/known_hosts" "$PI5_SSH_TARGET" "$@"
}
