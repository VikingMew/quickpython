"""
LLM Chat Demo - 完整展示多轮对话的过程

这个示例展示了：
1. 使用 llm.configure() 配置 API（传入字典）
2. 使用 llm.chat() 发送消息列表（支持对话历史）
3. llm.chat() 返回 JSON 字符串，包含 role 和 content
4. 使用 json.loads() 解析响应
5. 进行五轮对话，每轮都包含完整的对话历史
"""

import json

import llm

print("=== LLM Chat Demo - 5 Round Conversation ===")
print()

# 步骤 1: 配置 LLM
# llm.configure() 接受一个字典，包含 endpoint, api_key, model
print("Step 1: Configuring LLM...")
config = {
    "endpoint": "https://api.openai.com/v1/chat/completions",
    "api_key": "your-api-key-here",
    "model": "gpt-4",
}
llm.configure(config)
print("Configuration complete!")
print()

# 步骤 2: 初始化对话历史
# 包含 system prompt
messages = [
    {
        "role": "system",
        "content": "You are a helpful assistant that explains programming concepts clearly.",
    }
]

print("Step 2: Starting 5-round conversation...")
print("=" * 50)
print()

# 第一轮对话
print("Round 1:")
print("-" * 40)
user_msg_1 = "What is Python?"
print("User: " + user_msg_1)
print()

# 添加用户消息到历史
messages.append({"role": "user", "content": user_msg_1})

# llm.chat() 接受消息列表，包含完整的对话历史
# 返回 JSON 字符串，格式：{"role": "assistant", "content": "..."}
response_json_1 = llm.chat(messages)

print("Received JSON response:")
print(response_json_1)
print()

# 使用 json.loads() 解析 JSON 字符串
response_obj_1 = json.loads(response_json_1)
print("Parsed response object:")
print("  role: " + response_obj_1["role"])
print("  content: " + response_obj_1["content"])
print()

# 添加助手响应到历史
messages.append({"role": "assistant", "content": response_obj_1["content"]})

# 第二轮对话
print("Round 2:")
print("-" * 40)
user_msg_2 = "Can you explain it in simpler terms?"
print("User: " + user_msg_2)
print()

messages.append({"role": "user", "content": user_msg_2})
response_json_2 = llm.chat(messages)
response_obj_2 = json.loads(response_json_2)
print("Assistant: " + response_obj_2["content"])
print()

messages.append({"role": "assistant", "content": response_obj_2["content"]})

# 第三轮对话
print("Round 3:")
print("-" * 40)
user_msg_3 = "What are the main features of Python?"
print("User: " + user_msg_3)
print()

messages.append({"role": "user", "content": user_msg_3})
response_json_3 = llm.chat(messages)
response_obj_3 = json.loads(response_json_3)
print("Assistant: " + response_obj_3["content"])
print()

messages.append({"role": "assistant", "content": response_obj_3["content"]})

# 第四轮对话
print("Round 4:")
print("-" * 40)
user_msg_4 = "How do I start learning Python?"
print("User: " + user_msg_4)
print()

messages.append({"role": "user", "content": user_msg_4})
response_json_4 = llm.chat(messages)
response_obj_4 = json.loads(response_json_4)
print("Assistant: " + response_obj_4["content"])
print()

messages.append({"role": "assistant", "content": response_obj_4["content"]})

# 第五轮对话
print("Round 5:")
print("-" * 40)
user_msg_5 = "Thank you for the information!"
print("User: " + user_msg_5)
print()

messages.append({"role": "user", "content": user_msg_5})
response_json_5 = llm.chat(messages)
response_obj_5 = json.loads(response_json_5)
print("Assistant: " + response_obj_5["content"])
print()

print("=" * 50)
print("=== Demo Complete ===")
print("Total rounds: 5")
print("Total messages in history: " + str(len(messages) + 1))
print()
print("Note: 对话历史在应用层管理")
print("每次调用 llm.chat(messages) 都传入完整的历史")
print("这样 LLM 就能理解上下文并进行连贯的对话")
