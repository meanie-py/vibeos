"""Local conftest for the vibeos-cli unit tests.

The parent tests/qemu/conftest.py has an autouse `restore_vm` fixture that pulls
in the session `vm` fixture — which boots QEMU. These CLI unit tests are pure
(no hypervisor), so shadow `restore_vm` with a no-op autouse fixture (nearer
scope wins) to keep them fast and hermetic.
"""

import pytest


@pytest.fixture(autouse=True)
def restore_vm():
    yield
