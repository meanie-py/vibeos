# Offline C++ cohort

Python, Rust, C++, and JSON files here are inputs to the offline cohort.
The graders compile saved candidates and genuine references; they make no model calls.

cohort-manifest.json froze two development and two held-out units with exact
source and banked-body hashes before mechanism work. This is a selected-function
evaluation, not four complete module conversions. The reference graders extract
verbatim C++ bodies and include untouched banked Rust. They fail on source or
bank hash drift. Their reports also record compiler version, transitive header
hashes and the preprocessed C++ reference hash. They build in a temporary
directory by default; --artifact-dir retains generated sources and binaries.

Measured development scope: SheRemoveQuotesA/W matches genuine C++ on 173/173
full-buffer and return-pointer cases. The bounded Rust std::stol and
std::string::substr helper matches a genuine C++ standard-library oracle on
372/372 development cases. The C++ oracle probes long width and binary-prefix
support; the Rust helper returns UnsupportedInput beyond its documented bases,
widths and ASCII locale. The caller must preserve C++ argument expressions:
substr(start, end) interprets end as a length. The separate --split heldout
random seed was not used to choose implementation behavior. The macro audit
flags the original txt2nls candidate's missing MAXIMUM_LEADBYTES declaration.
It supports active, source-owned numeric object-like literals; expression,
function-like, multiline, redefined and inactive macros do not become automatic
Rust constants. Two macro tests, one header-provenance test and three Rust tests cover these
helpers.

Measured held-out scope: verbatim CEnumIDListBase::Skip/Reset and banked Rust
match on 83/83 bounded cursor cases. Untouched banked Next fails rustc E0308
(pointer slot assigned integer zero), so that candidate is rejected. The host
substrate cannot faithfully execute AppendItemsFromEnumerator or DeleteList
with their ownership, callbacks and allocator behavior. CCopyAsPathMenu's
DoCopyAsPath has only partial preserved outputs for two required helpers and
lacks a COM/clipboard host substrate. These methods are unsupported, not
accepted. No held-out body was repaired. Harness construction inspected the
held-out bytes, so this is a split from translation-repair development, not a
blinded experiment.

From tools/rustconv, set REPO to the frozen-source
worktree and BANK to the shell32 worktree's tools/rustconv/.responses directory:

    python3 -m rustconv.cpp_cohort.grade_semantics --split dev --output /path/to/semantics.json
    python3 -m rustconv.cpp_cohort.grade_quotes --repo "$REPO" --bank "$BANK" --output /path/to/quotes.json
    python3 -m rustconv.cpp_cohort.grade_cenum --repo "$REPO" --bank "$BANK" --output /path/to/cenum.json
    python3 -m rustconv.cpp_cohort.macro_contract "$REPO/sdk/tools/txt2nls/main.cpp" --candidate "$REPO/tools/rustconv/.responses/host-pilots/txt2nls/nonthinking/candidate-d4c8ac3a0b93b84e.rs" --output /path/to/macro.json
    python3 -m unittest -v rustconv.cpp_cohort.test_macro_contract rustconv.cpp_cohort.test_host_provenance
    rustc --edition=2021 --test rustconv/cpp_cohort/cpp_std_semantics.rs -o /path/to/semantics-tests
    /path/to/semantics-tests

The C++ shims make selected bodies host executable. They do not prove full COM
ABI, OS integration, allocation ownership or behavior on other hosts. The
whole-unit verdict must retain the rejected and unsupported methods.
