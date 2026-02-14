use quickpython::{ExceptionType, Module, Value};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;

static LLM_CONFIG: Mutex<Option<LlmConfig>> = Mutex::new(None);

#[derive(Clone)]
struct LlmConfig {
    endpoint: String,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub fn create_module() -> Module {
    let mut module = Module::new("llm");

    module.add_function("configure", llm_configure);
    module.add_function("chat", llm_chat);

    module
}

fn llm_configure(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "llm.configure() takes exactly 1 argument: config dict",
        ));
    }

    let config_dict = match &args[0] {
        Value::Dict(d) => d.borrow(),
        _ => {
            return Err(Value::error(
                ExceptionType::TypeError,
                "config must be a dict",
            ));
        }
    };

    // 提取 endpoint
    let endpoint_key = quickpython::DictKey::String("endpoint".to_string());
    let endpoint = match config_dict.get(&endpoint_key) {
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            return Err(Value::error(
                ExceptionType::TypeError,
                "config['endpoint'] must be a string",
            ));
        }
        None => {
            return Err(Value::error(
                ExceptionType::KeyError,
                "config missing required key: 'endpoint'",
            ));
        }
    };

    // 提取 api_key
    let api_key_key = quickpython::DictKey::String("api_key".to_string());
    let api_key = match config_dict.get(&api_key_key) {
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            return Err(Value::error(
                ExceptionType::TypeError,
                "config['api_key'] must be a string",
            ));
        }
        None => {
            return Err(Value::error(
                ExceptionType::KeyError,
                "config missing required key: 'api_key'",
            ));
        }
    };

    // 提取 model
    let model_key = quickpython::DictKey::String("model".to_string());
    let model = match config_dict.get(&model_key) {
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            return Err(Value::error(
                ExceptionType::TypeError,
                "config['model'] must be a string",
            ));
        }
        None => {
            return Err(Value::error(
                ExceptionType::KeyError,
                "config missing required key: 'model'",
            ));
        }
    };

    let config = LlmConfig {
        endpoint,
        api_key,
        model,
    };

    let mut global_config = LLM_CONFIG.lock().unwrap();
    *global_config = Some(config);

    Ok(Value::None)
}

fn llm_chat(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "llm.chat() takes exactly 1 argument: messages list",
        ));
    }

    let messages_list = match &args[0] {
        Value::List(l) => l.borrow(),
        _ => {
            return Err(Value::error(
                ExceptionType::TypeError,
                "messages must be a list",
            ));
        }
    };

    // 解析消息列表
    let mut messages = Vec::new();
    for (i, msg_value) in messages_list.items.iter().enumerate() {
        let msg_dict = match msg_value {
            Value::Dict(d) => d.borrow(),
            _ => {
                return Err(Value::error(
                    ExceptionType::TypeError,
                    &format!("messages[{}] must be a dict", i),
                ));
            }
        };

        // 提取 role
        let role_key = quickpython::DictKey::String("role".to_string());
        let role = match msg_dict.get(&role_key) {
            Some(Value::String(s)) => s.clone(),
            Some(_) => {
                return Err(Value::error(
                    ExceptionType::TypeError,
                    &format!("messages[{}]['role'] must be a string", i),
                ));
            }
            None => {
                return Err(Value::error(
                    ExceptionType::KeyError,
                    &format!("messages[{}] missing required key: 'role'", i),
                ));
            }
        };

        // 提取 content
        let content_key = quickpython::DictKey::String("content".to_string());
        let content = match msg_dict.get(&content_key) {
            Some(Value::String(s)) => s.clone(),
            Some(_) => {
                return Err(Value::error(
                    ExceptionType::TypeError,
                    &format!("messages[{}]['content'] must be a string", i),
                ));
            }
            None => {
                return Err(Value::error(
                    ExceptionType::KeyError,
                    &format!("messages[{}] missing required key: 'content'", i),
                ));
            }
        };

        messages.push(Message { role, content });
    }

    if messages.is_empty() {
        return Err(Value::error(
            ExceptionType::ValueError,
            "messages list cannot be empty",
        ));
    }

    let config = {
        let global_config = LLM_CONFIG.lock().unwrap();
        match global_config.as_ref() {
            Some(c) => c.clone(),
            None => {
                return Err(Value::error(
                    ExceptionType::RuntimeError,
                    "LLM not configured. Call llm.configure() first",
                ));
            }
        }
    };

    let client = match Client::builder().build() {
        Ok(c) => c,
        Err(e) => {
            return Err(Value::error(
                ExceptionType::RuntimeError,
                &format!("Failed to create HTTP client: {}", e),
            ));
        }
    };

    let request_body = ChatRequest {
        model: config.model.clone(),
        messages,
    };

    let response = match client
        .post(&config.endpoint)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            return Err(Value::error(
                ExceptionType::RuntimeError,
                &format!("HTTP request failed: {}", e),
            ));
        }
    };

    let response_text = match response.text() {
        Ok(t) => t,
        Err(e) => {
            return Err(Value::error(
                ExceptionType::RuntimeError,
                &format!("Failed to read response: {}", e),
            ));
        }
    };

    let chat_response: ChatResponse = match serde_json::from_str(&response_text) {
        Ok(r) => r,
        Err(e) => {
            return Err(Value::error(
                ExceptionType::RuntimeError,
                &format!("Failed to parse response: {}", e),
            ));
        }
    };

    if chat_response.choices.is_empty() {
        return Err(Value::error(
            ExceptionType::RuntimeError,
            "No response from LLM",
        ));
    }

    // 返回 JSON 字符串，包含 role 和 content
    let response_message = &chat_response.choices[0].message;
    let result = json!({
        "role": response_message.role,
        "content": response_message.content
    });

    Ok(Value::String(result.to_string()))
}
