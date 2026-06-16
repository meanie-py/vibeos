"""Bridge routing: which agent(s) own a task, by assignee or by `agent:<name>` label."""

from agent_swarm.bridge import VikunjaBridge


class FakeMMChannels:
    def __init__(self, channels):
        self._channels = set(channels)

    def find_channel(self, name):
        return "chan-id" if name in self._channels else None


def _bridge(known_agents):
    """A bridge with routing wired but no network (bypasses __init__)."""
    b = VikunjaBridge.__new__(VikunjaBridge)
    b.mm = FakeMMChannels({f"agent-{a}" for a in known_agents})
    b._agent_cache = {}
    return b


def test_route_by_assignee():
    b = _bridge({"win32k"})
    task = {"assignees": [{"username": "win32k"}], "labels": []}
    assert b._agents_for(task) == {"win32k"}


def test_route_by_agent_label():
    b = _bridge({"win32k"})
    assert b._agents_for({"labels": [{"title": "agent:win32k"}]}) == {"win32k"}
    assert b._agents_for({"labels": [{"title": "win32k"}]}) == {"win32k"}  # bare name


def test_unknown_assignee_is_not_routed():
    b = _bridge({"win32k"})
    assert b._agents_for({"assignees": [{"username": "ghost"}]}) == set()


def test_multiple_agents_on_one_task():
    b = _bridge({"win32k", "ntdll"})
    task = {"assignees": [{"username": "win32k"}], "labels": [{"title": "agent:ntdll"}]}
    assert b._agents_for(task) == {"win32k", "ntdll"}
