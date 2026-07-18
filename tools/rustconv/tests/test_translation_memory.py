from rustconv.response_cache import ExactBodyTranslationMemory


def identity(class_name: str):
    return {
        "body_text": f"HRESULT {class_name}::Forward(DWORD value) {{ return inner(value); }}",
        "class_name": class_name,
        "method": "Forward",
        "return_type": "HRESULT",
        "params": [("DWORD", "value")],
        "class_shape": "IFolder|inner:*mut IFolder",
        "namespace": "mechanical__factseal_v1__callseal_v1",
    }


def test_exact_body_translation_memory_instantiates_class_name(tmp_path):
    memory = ExactBodyTranslationMemory(tmp_path)
    rust = (
        "pub unsafe fn CFontsFolder_Forward_body(this: *mut CFontsFolder, value: DWORD) "
        "-> HRESULT { inner(value) }"
    )
    memory.put(rust, **identity("CFontsFolder"))
    hit = memory.get(**identity("CAdminToolsFolder"))
    assert hit is not None
    assert "CAdminToolsFolder_Forward_body" in hit
    assert "*mut CAdminToolsFolder" in hit
    assert "CFontsFolder" not in hit


def test_exact_body_translation_memory_is_scoped_by_shape_and_namespace(tmp_path):
    memory = ExactBodyTranslationMemory(tmp_path)
    memory.put("pub unsafe fn CFontsFolder_Forward_body() {}", **identity("CFontsFolder"))
    changed = identity("CAdminToolsFolder")
    changed["class_shape"] = "different"
    assert memory.get(**changed) is None
    changed = identity("CAdminToolsFolder")
    changed["namespace"] = "unsealed"
    assert memory.get(**changed) is None
