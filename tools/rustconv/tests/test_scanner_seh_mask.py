"""The scanner must see a PSEH function whole, however it is laid out."""

from rustconv.scanner import mask_seh_for_parse, scan_file

SRC = """int f(void)
{
    g();
    _SEH2_TRY
    {
        if (x) _SEH2_LEAVE;
        _SEH2_YIELD(return 3);
    }
    _SEH2_FINALLY
    {
        NtClose(h);
    }
    _SEH2_END;
    return a;
}

int h(void) { _SEH2_TRY { a=1; } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { a=2; } _SEH2_END; return a; }
"""


def test_mask_keeps_length_and_lines():
    masked = mask_seh_for_parse(SRC.encode())
    assert len(masked) == len(SRC.encode())
    assert masked.count(b"\n") == SRC.count("\n")
    assert b"_SEH2" not in masked


def test_pseh_functions_are_scanned_whole(tmp_path):
    p = tmp_path / "x.c"
    p.write_text(SRC)
    scan = scan_file(p)
    bodies = {fn.name: fn.body_text for fn in scan.functions}
    assert set(bodies) == {"f", "h"}
    assert "NtClose(h);" in bodies["f"] and "return a;" in bodies["f"]
    assert "_SEH2_FINALLY" in bodies["f"]  # the body text itself is the unmasked C
    assert "_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)" in bodies["h"]
    assert "NtClose" not in (scan.preamble_text or "")


RPC_SRC = """static int k(int x)
{
    int ret = 0;
    if (x)
    {
        RpcTryExcept
        {
            ret = A(x);
        }
        RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        {
            ret = B(RpcExceptionCode());
        }
        RpcEndExcept;

        if (ret != 0)
        {
            RpcTryExcept
            {
                ret = C(x);
            }
            RpcExcept(EXCEPTION_EXECUTE_HANDLER)
            {
                ret = B(RpcExceptionCode());
            }
            RpcEndExcept;

            if (ret != 0)
                goto done;
        }
        __TRY
        {
            D(ret);
        }
        __EXCEPT_PAGE_FAULT
        {
            ret = 1;
        }
        __ENDTRY
    }
done:
    return ret;
}

static int m(void) { return 1; }
"""


def test_rpc_and_wine_try_blocks_are_scanned_whole(tmp_path):
    masked = mask_seh_for_parse(RPC_SRC.encode())
    assert len(masked) == len(RPC_SRC.encode())
    assert b"RpcTryExcept" not in masked and b"__TRY" not in masked
    p = tmp_path / "r.c"
    p.write_text(RPC_SRC)
    scan = scan_file(p)
    bodies = {fn.name: fn.body_text for fn in scan.functions}
    assert set(bodies) == {"k", "m"}
    assert "return ret;" in bodies["k"] and "__EXCEPT_PAGE_FAULT" in bodies["k"]
    assert "return ret" not in (scan.preamble_text or "")
