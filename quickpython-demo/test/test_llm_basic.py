# 测试 LLM 模块基本功能

import llm


# 场景 2: 扩展模块注册成功
def test_llm_module_imported():
    print("PASS: llm module imported successfully")


test_llm_module_imported()


# 场景 3: llm.configure() 配置 - 完整配置
def test_llm_configure():
    config = {
        "endpoint": "https://api.openai.com/v1/chat/completions",
        "api_key": "test-api-key",
        "model": "gpt-4",
    }
    llm.configure(config)
    print("PASS: llm.configure() succeeded with complete config")


test_llm_configure()


# 场景 4: llm.configure() 缺少配置字段 - endpoint
def test_llm_configure_missing_endpoint():
    try:
        config = {"api_key": "test-key", "model": "gpt-4"}
        llm.configure(config)
        print("FAIL: Should have raised KeyError for missing endpoint")
    except KeyError as e:
        print("PASS: KeyError raised for missing endpoint")


test_llm_configure_missing_endpoint()


# 场景 4: llm.configure() 缺少配置字段 - api_key
def test_llm_configure_missing_api_key():
    try:
        config = {
            "endpoint": "https://api.openai.com/v1/chat/completions",
            "model": "gpt-4",
        }
        llm.configure(config)
        print("FAIL: Should have raised KeyError for missing api_key")
    except KeyError as e:
        print("PASS: KeyError raised for missing api_key")


test_llm_configure_missing_api_key()


# 场景 4: llm.configure() 缺少配置字段 - model
def test_llm_configure_missing_model():
    try:
        config = {
            "endpoint": "https://api.openai.com/v1/chat/completions",
            "api_key": "test-key",
        }
        llm.configure(config)
        print("FAIL: Should have raised KeyError for missing model")
    except KeyError as e:
        print("PASS: KeyError raised for missing model")


test_llm_configure_missing_model()


# 场景: llm.configure() 参数错误 - 不是字典
def test_llm_configure_not_dict():
    try:
        llm.configure("not a dict")
        print("FAIL: Should have raised TypeError")
    except TypeError as e:
        print("PASS: TypeError raised for non-dict config")


test_llm_configure_not_dict()


# 场景: llm.configure() 参数错误 - 字段类型错误
def test_llm_configure_wrong_field_type():
    try:
        config = {"endpoint": 123, "api_key": "test-key", "model": "gpt-4"}
        llm.configure(config)
        print("FAIL: Should have raised TypeError")
    except TypeError as e:
        print("PASS: TypeError raised for wrong field type")


test_llm_configure_wrong_field_type()


# 场景 8: llm.chat() 参数错误 - 无参数
def test_llm_chat_no_args():
    try:
        llm.chat()
        print("FAIL: Should have raised TypeError")
    except TypeError as e:
        print("PASS: TypeError raised for no arguments")


test_llm_chat_no_args()


# 场景 8: llm.chat() 参数类型错误 - 不是列表
def test_llm_chat_not_list():
    try:
        llm.chat("not a list")
        print("FAIL: Should have raised TypeError")
    except TypeError as e:
        print("PASS: TypeError raised for non-list messages")


test_llm_chat_not_list()


# 场景: llm.chat() 空消息列表
def test_llm_chat_empty_list():
    try:
        llm.chat([])
        print("FAIL: Should have raised ValueError")
    except ValueError as e:
        print("PASS: ValueError raised for empty messages list")


test_llm_chat_empty_list()


# 场景: llm.chat() 消息格式错误 - 不是字典
def test_llm_chat_message_not_dict():
    try:
        llm.chat(["not a dict"])
        print("FAIL: Should have raised TypeError")
    except TypeError as e:
        print("PASS: TypeError raised for non-dict message")


test_llm_chat_message_not_dict()


# 场景: llm.chat() 消息缺少 role
def test_llm_chat_message_missing_role():
    try:
        llm.chat([{"content": "Hello"}])
        print("FAIL: Should have raised KeyError")
    except KeyError as e:
        print("PASS: KeyError raised for missing role")


test_llm_chat_message_missing_role()


# 场景: llm.chat() 消息缺少 content
def test_llm_chat_message_missing_content():
    try:
        llm.chat([{"role": "user"}])
        print("FAIL: Should have raised KeyError")
    except KeyError as e:
        print("PASS: KeyError raised for missing content")


test_llm_chat_message_missing_content()


print("")
print("=== All basic tests completed ===")
