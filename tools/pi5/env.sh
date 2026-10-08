# Sourced by the tools/pi5 scripts: the Pi 5 bench is the Pi 4 bench's machinery
# (tools/pi4) pointed at the other board's plug, Pico, capture card and lease.
# Each value can be overridden from the environment.
export VIBEOS_BENCH_BOARD=pi5
export PI4_BENCH=${PI5_BENCH:-/mnt/d/home/vibeos-rpi5-setup}
export PI4_PICO_HOST=${PI5_PICO_HOST:-picousb}
export PI4_HDMI_VID=${PI5_HDMI_VID:-345f}
# The second Shelly plug (a Plug US Gen4, like the Pi 4's; 192.168.1.87 on 2026-10-05).
export PI4_PLUG=${PI5_PLUG:-shellyplugusg4-acebe6f18e28}
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
