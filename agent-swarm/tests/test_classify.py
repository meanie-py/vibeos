from agent_swarm.queue import classify, is_board_high, is_mention
from agent_swarm.types import Priority


def test_direct_mention_is_high():
    assert classify("hey @win32k look at this", "win32k") is Priority.HIGH
    assert classify("nothing for you here", "win32k") is Priority.LOW


def test_mention_is_case_insensitive():
    assert is_mention("ping @WIN32K", "win32k")
    assert not is_mention("win32k without an at-sign", "win32k")


def test_here_channel_all_are_high_for_everyone():
    assert classify("@here heads up", None) is Priority.HIGH
    assert classify("@channel ping", "anyone") is Priority.HIGH
    assert classify("@all ping", None) is Priority.HIGH


def test_board_priority_detection():
    assert is_board_high("manual.test priority DO NOW")
    assert is_board_high("#5 priority: High needs review")
    assert is_board_high("task priority Urgent")
    assert not is_board_high("task priority Low someday")
    assert not is_board_high("just normal chatter")


def test_board_high_is_high_even_without_mention():
    assert classify("#3 fix the heap corruptor · priority DO NOW", None) is Priority.HIGH
    assert classify("#9 priority Low someday", None) is Priority.LOW
