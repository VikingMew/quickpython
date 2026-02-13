# 测试 LLM 模块的未配置错误

import llm


def test_llm_chat_not_configured():
    try:
        messages = [{"role": "user", "content": "Hello"}]
        llm.chat(messages)
        print("FAIL: Should have raised RuntimeError")
    except RuntimeError as e:
        print("PASS: RuntimeError raised when not configured")


test_llm_chat_not_configured()
