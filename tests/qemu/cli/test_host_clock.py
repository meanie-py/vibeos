from . import host_clock


def test_measured_steps_distinguish_slew_from_backwards_jump():
    samples = [
        {"elapsed_seconds": 0, "offset_ns": 0, "read_latency_ns": 100},
        {"elapsed_seconds": 1, "offset_ns": 1000, "read_latency_ns": 100},
        {"elapsed_seconds": 2, "offset_ns": -1_200_000_000, "read_latency_ns": 100},
    ]
    changes = host_clock.steps(samples)
    assert changes[0]["step_ms"] == 0.001
    assert changes[1]["step_ms"] == -1200.001


def test_descheduled_clock_pair_cannot_manufacture_a_step():
    samples = [
        {"elapsed_seconds": 0, "offset_ns": 0, "read_latency_ns": 100},
        {"elapsed_seconds": 1, "offset_ns": 100_000_000, "read_latency_ns": 200_000_000},
        {"elapsed_seconds": 2, "offset_ns": 0, "read_latency_ns": 100},
    ]
    assert host_clock.steps(samples) == [{"at_seconds": 2, "step_ms": 0}]
