# 测试扩展模块注册机制 - 核心库不包含扩展模块（未注册）


def test_import_llm_without_registration():
    try:
        import llm

        print("FAIL: Should have raised RuntimeError")
    except RuntimeError as e:
        print("PASS: RuntimeError raised as expected for missing module")


test_import_llm_without_registration()
