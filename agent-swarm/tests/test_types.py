from agent_swarm.types import (
    Agent,
    AgentStatus,
    DrainResult,
    HighMessage,
    Priority,
    VikunjaPriority,
)


def test_vikunja_priority_label_and_parse():
    assert VikunjaPriority.DO_NOW.label == "DO NOW"
    assert VikunjaPriority.HIGH.label == "High"
    assert VikunjaPriority.parse(5) is VikunjaPriority.DO_NOW
    assert VikunjaPriority.parse("3") is VikunjaPriority.HIGH  # numeric string
    assert VikunjaPriority.parse(None) is VikunjaPriority.UNSET  # safe on missing
    assert VikunjaPriority.parse(99) is VikunjaPriority.UNSET  # out of range


def test_priority_serializes_as_str():
    assert Priority.HIGH.value == "high"
    assert Priority.LOW.value == "low"


def test_drain_result_to_dict_uses_from_key():
    r = DrainResult(high=[HighMessage("alice", "hi", "p1", "ch")], low_count=2)
    assert r.to_dict() == {
        "high": [{"from": "alice", "message": "hi", "post_id": "p1", "channel": "ch"}],
        "low_count": 2,
    }


def test_empty_drain_result():
    assert DrainResult().to_dict() == {"high": [], "low_count": 0}


def test_agent_to_dict_status_is_str():
    a = Agent(identity="win32k", status=AgentStatus.ONLINE, project="user32")
    d = a.to_dict()
    assert d["status"] == "online"
    assert d["identity"] == "win32k" and d["project"] == "user32"
