"""A preamble with nothing to translate is decided from the source, for $0."""

from rustconv.__main__ import _preamble_has_translatable_content as real


def test_header_switches_are_not_file_scope_content():
    # setuplib genlist.c: the model correctly answered nothing and the run hard-failed
    assert not real('#include "precomp.h"\n#include "genlist.h"\n\n#define NDEBUG\n#include <debug.h>\n')
    assert not real("#define WIN32_NO_STATUS\n#include <windef.h>\n#undef WIN32_NO_STATUS\n")


def test_real_content_still_goes_to_the_model():
    assert real("#define NDEBUG\nstatic const int table[] = { 1, 2 };\n")
    assert real("#define INITGUID\n#include <guiddef.h>\n")  # changes which GUIDs it emits
    assert real("#define MAX_ITEMS 10\n")  # a valued define is a constant
