# shellcheck shell=bash
# Sourced by the tools/optiplex scripts: the OptiPlex 9020 bench is the Pi 4
# bench's machinery (tools/pi4) pointed at the x64 box that took over its plug,
# Pico and capture card (docs/engineering/optiplex-bench.md). Each value can be
# overridden from the environment.
export VIBEOS_BENCH_BOARD=optiplex
export PI4_BENCH=${OPTIPLEX_BENCH:-/mnt/d/home/vibeos-optiplex}
export PI4_PICO_HOST=${OPTIPLEX_PICO_HOST:-picousb5}
export PI4_HDMI_VID=${OPTIPLEX_HDMI_VID:-345f}
# The plug that used to power the Pi 4.
export PI4_PLUG=${OPTIPLEX_PLUG:-shellyplugusg4-e8f60a7df904}
[ -n "${OPTIPLEX_PLUG_PASS:-}" ] && export PI4_PLUG_PASS=$OPTIPLEX_PLUG_PASS
# The 9020's COM1 through a Prolific PL2303GT on the host (Windows COM7), 115200 8N1.
export OPTIPLEX_COM=${OPTIPLEX_COM:-COM7}
# Where the bench HTTP server (serve_http.py on the Windows host, :8069) serves
# from; the 9020's iPXE sanboots vibeos-amd64.iso out of it (netboot.sh).
export OPTIPLEX_PAYLOAD=${OPTIPLEX_PAYLOAD:-/mnt/d/home/vibeos-rpi4-setup/payload}
