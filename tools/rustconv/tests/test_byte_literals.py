from rustconv.byte_literals import escape_nonascii_byte_strings as esc


def test_a_utf8_character_in_a_byte_string_becomes_its_bytes():
    src = 'DbgPrint(b"slot unresolved — SEH disabled\\n\\0".as_ptr());'
    assert esc(src) == 'DbgPrint(b"slot unresolved \\xe2\\x80\\x94 SEH disabled\\n\\0".as_ptr());'


def test_comments_strings_and_ascii_byte_strings_are_untouched():
    src = '/* a — b */ let s = "x — y"; let t = b"ok\\"—"; let u = ab"q";'
    out = esc(src)
    assert out.startswith('/* a — b */ let s = "x — y";')
    assert 'b"ok\\"\\xe2\\x80\\x94"' in out  # the escaped quote does not end the literal
    assert esc('b"plain"') == 'b"plain"'
