"""Busy/idle state machine, including the PID-liveness auto-flip."""

import subprocess

from agent_swarm import state


def test_set_and_read_busy_idle(env):
    state.set_state("agent", "busy")
    assert state.is_busy("agent")
    state.set_state("agent", "idle")
    assert not state.is_busy("agent")


def test_unknown_identity_is_not_busy(env):
    assert not state.is_busy("never-seen")


def test_dead_pid_auto_flips_to_idle(env):
    # a reaped child is a guaranteed-dead pid
    proc = subprocess.Popen(["true"])
    proc.wait()
    dead_pid = proc.pid
    state.set_state("agent", "busy", pid=dead_pid)
    st = state.get_state("agent")
    assert st["alive"] is False
    assert st["state"] == "idle"  # dead process → idle regardless of last write
    assert not state.is_busy("agent")


def test_live_pid_stays_busy(env):
    import os

    state.set_state("agent", "busy", pid=os.getpid())  # this test process is alive
    assert state.is_busy("agent")
