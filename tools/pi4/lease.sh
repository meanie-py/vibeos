# Sourced by the bench scripts before anything that changes the board's state:
# refuses (exit 75) while another lane holds the Pi 4 bench lease, otherwise
# takes or renews it for this worktree. Mechanism: tests/qemu/cli/pi4lease.py;
# front door: `vibeos-cli pi4 lease`.
pi4_lease() {
    local repo
    repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
    (cd "$repo" && python3 tests/qemu/cli/pi4lease.py touch) || exit 75
}
