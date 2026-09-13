# Registry security survives key-cache eviction

The clean gate at `/tmp/vibeos-cli/20260912-212300-vr4z3jhp-gate-i386+amd64+arm64`
reported an intermittent i386 `CmSecurity` regression. Its first conformance run,
`/tmp/vibeos-cli/20260912-212758-pfnvvpbl-conformance-i386`, returned World owner/group,
a single World GENERIC_ALL ACE and SACL-present for `\REGISTRY\USER`. Those values
match `CmpQueryDefaultSecurityDescriptor`; the later gate retry passed.

New keys had `Security = HCELL_NIL`, while assignment stored the only descriptor
in the KCB cache. Closing and evicting that KCB therefore lost the assigned
security. Exact eviction in the failing boot was not traced; descriptor loss on
eviction follows directly from the creation, assignment and teardown paths.

Assignment and replacement now persist a private self-relative descriptor in a
hive security cell before publishing the prepared cache. Allocation, mapping and
dirtying failures preserve the previous state. Shared descriptors retain their
other references. Deletion uses the same security lock and releases the actual
mapped cell before freeing it; cross-hive copies reset links and reference count.

`tools/test_registry_security_persistence.py` compiles the shipping Rust helper
against a fault-injecting hive substrate. It checks cache-independent persistence,
volatile storage, transactional failures, shared descriptors and list unlinking.
These are algorithm controls; all three target builds and guest conformance
provide the ABI and integration checks.
