```
warning: Missing child 103700211514640 of node AstNode { tag: TagDeclStmt, children: [Some(..)],
         loc: SrcSpan { fileid: 1, begin_line: 3, .. }, type_id: None, .. }
Exported Clang AST was invalid. Check warnings above for unimplemented features.

thread 'main' panicked at c2rust-transpile-0.22.1/src/c_ast/mod.rs:314:30:
Could not find CDeclId(33364) in TypedAstContext
```

### Reproducer (9 lines, no headers)

```c
/* repro.c */
int f(int x) {
    {
        __label__ done;      /* GNU C local label declaration */
        if (x) goto done;
        x = 1;
        done:;
    }
    return x;
}
```

### Characterization

| case | result |
|---|---|
| `__label__` declared and used | ICE, 1 × `Missing child` |
| `__label__` declared, never used | ICE, 1 × `Missing child` |
| `__label__ a, b;` (two in one decl) | ICE, 2 × `Missing child` |
| `__label__` inside a statement-expression | ICE, 1 × `Missing child` |
| ordinary label + `goto`, no `__label__` | **clean** |

So the AST exporter emits no child for the `LabelDecl` inside a `DeclStmt`; one warning per
declared local label. Ordinary labels are unaffected.

### Two separate defects

1. **Feature gap:** `__label__` is not exported. (A GNU extension, but c2rust targets GNU C.)
2. **Robustness — the more annoying half:** c2rust *already knows* the AST is invalid and says
   so with a file:line. It then keeps going and dies with `Could not find CDeclId(33364) in
   TypedAstContext`, whose backtrace lands in `CommentLocator::pre` → `get_src_loc` — a place
   with no relationship to the actual problem. **The diagnostic it printed is actionable; the
   panic it exits with is not.** Bailing out after "Exported Clang AST was invalid" (or making
   `get_src_loc` return `None` for a missing decl, as `Index<CExprId>` already does with its
   `BADEXPR` fallback at `c_ast/mod.rs:1360`) would turn an ICE into a clean error.

---

---
Found while transpiling ReactOS as a translation anchor. Reproduced against c2rust 0.22.1 from crates.io on x86_64 Linux, target `x86_64-w64-mingw32`.
