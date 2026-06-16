use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core;

const CONFIG_FILE: &str = "ai_config.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiProvider {
    pub name: String,
    pub base_url: String,
    pub models_endpoint: Option<String>,
}

pub fn get_providers() -> Vec<AiProvider> {
    vec![
        AiProvider {
            name: "Claude (Anthropic)".into(),
            base_url: "https://api.anthropic.com".into(),
            models_endpoint: None,
        },
        AiProvider {
            name: "GPT (OpenAI)".into(),
            base_url: "https://api.openai.com".into(),
            models_endpoint: Some("/v1/models".into()),
        },
        AiProvider {
            name: "Gemini (Google)".into(),
            base_url: "https://generativelanguage.googleapis.com".into(),
            models_endpoint: Some("/v1/models".into()),
        },
        AiProvider {
            name: "小米 MiMo".into(),
            base_url: "https://api.xiaomi.com".into(),
            models_endpoint: None,
        },
        AiProvider {
            name: "DeepSeek".into(),
            base_url: "https://api.deepseek.com".into(),
            models_endpoint: Some("/v1/models".into()),
        },
        AiProvider {
            name: "通义千问 (阿里)".into(),
            base_url: "https://dashscope.aliyuncs.com".into(),
            models_endpoint: Some("/v1/models".into()),
        },
        AiProvider {
            name: "豆包 (字节)".into(),
            base_url: "https://ark.cn-beijing.volces.com".into(),
            models_endpoint: None,
        },
        AiProvider {
            name: "自定义 (OpenAI 兼容)".into(),
            base_url: String::new(),
            models_endpoint: Some("/v1/models".into()),
        },
    ]
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".reptool")
        .join(CONFIG_FILE)
}

pub fn load_config() -> Option<AiConfig> {
    let path = config_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_config(config: &AiConfig) -> Result<()> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}

pub fn setup() -> Result<()> {
    println!("{}", "=== AI 逆向助手配置 ===".green().bold());
    println!();

    let providers = get_providers();

    // 选择厂商
    println!("选择 AI 厂商:");
    for (i, p) in providers.iter().enumerate() {
        println!("  {}. {}", i + 1, p.name);
    }
    println!();
    print!("请输入编号 (1-{}): ", providers.len());
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse().map_err(|_| anyhow!("无效编号"))?;
    if choice == 0 || choice > providers.len() {
        return Err(anyhow!("编号超出范围"));
    }

    let provider = &providers[choice - 1];
    println!("已选择: {}", provider.name.cyan());
    println!();

    // 输入 Base URL
    let base_url = if provider.base_url.is_empty() {
        print!("请输入 API Base URL: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut url = String::new();
        std::io::stdin().read_line(&mut url)?;
        url.trim().to_string()
    } else {
        print!("API Base URL [{}]: ", provider.base_url);
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut url = String::new();
        std::io::stdin().read_line(&mut url)?;
        let url = url.trim().to_string();
        if url.is_empty() { provider.base_url.clone() } else { url }
    };

    // 输入 API Key
    print!("API Key: ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut api_key = String::new();
    std::io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();
    if api_key.is_empty() {
        return Err(anyhow!("API Key 不能为空"));
    }

    println!();
    core::info("正在获取模型列表...");

    // 获取模型列表
    let models = fetch_models(&base_url, &api_key, provider.models_endpoint.as_deref())?;

    if models.is_empty() {
        core::warn("无法获取模型列表, 请手动输入模型名称");
        print!("模型名称: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut model = String::new();
        std::io::stdin().read_line(&mut model)?;
        let model = model.trim().to_string();

        let config = AiConfig {
            provider: provider.name.clone(),
            base_url,
            api_key,
            model,
        };
        save_config(&config)?;
    } else {
        println!("可用模型:");
        for (i, m) in models.iter().enumerate() {
            println!("  {}. {}", i + 1, m);
        }
        println!();
        print!("请选择模型编号 (1-{}): ", models.len());
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut model_input = String::new();
        std::io::stdin().read_line(&mut model_input)?;
        let model_choice: usize = model_input.trim().parse().unwrap_or(1);
        let model = models.get(model_choice - 1).cloned().unwrap_or_else(|| models[0].clone());

        let config = AiConfig {
            provider: provider.name.clone(),
            base_url,
            api_key,
            model,
        };
        save_config(&config)?;
    }

    println!();
    core::success("AI 配置已保存!");
    show_config();
    Ok(())
}

pub fn show_config() {
    match load_config() {
        Some(config) => {
            println!();
            println!("{}", "当前 AI 配置:".yellow().bold());
            println!("  厂商:   {}", config.provider);
            println!("  Base:   {}", config.base_url);
            println!("  模型:   {}", config.model);
            println!("  Key:    {}...{}", &config.api_key[..4.min(config.api_key.len())], &config.api_key[config.api_key.len().saturating_sub(4)..]);
        }
        None => {
            core::warn("未配置 AI, 请运行: reptool ai setup");
        }
    }
}

fn fetch_models(base_url: &str, api_key: &str, endpoint: Option<&str>) -> Result<Vec<String>> {
    let ep = match endpoint {
        Some(e) => e,
        None => return Ok(vec![]),
    };

    let url = format!("{}{}", base_url.trim_end_matches('/'), ep);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            core::warn(&format!("获取模型列表失败: HTTP {}", resp.status()));
            return Ok(vec![]);
        }

        let body: serde_json::Value = resp.json().await?;

        // 兼容 OpenAI 格式: { "data": [{ "id": "gpt-4", ... }] }
        if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            let models: Vec<String> = data
                .iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                .collect();
            return Ok(models);
        }

        // 兼容其他格式: { "models": ["model1", "model2"] }
        if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
            let models: Vec<String> = models
                .iter()
                .filter_map(|m| m.as_str().map(String::from))
                .collect();
            return Ok(models);
        }

        Ok(vec![])
    })
}

pub fn chat(user_message: &str) -> Result<()> {
    let config = load_config()
        .ok_or_else(|| anyhow!("未配置 AI, 请先运行: reptool ai setup"))?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        let provider_lower = config.provider.to_lowercase();

        if provider_lower.contains("claude") || provider_lower.contains("anthropic") {
            chat_claude(&client, &config, user_message).await
        } else {
            // OpenAI 兼容格式 (GPT, Gemini, DeepSeek, 小米, 通义, 豆包, 自定义)
            chat_openai_compat(&client, &config, user_message).await
        }
    })
}

async fn chat_claude(client: &reqwest::Client, config: &AiConfig, user_message: &str) -> Result<()> {
    let url = format!("{}/v1/messages", config.base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": config.model,
        "max_tokens": 4096,
        "messages": [{
            "role": "user",
            "content": user_message
        }]
    });

    let resp = client
        .post(&url)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API 错误 ({}): {}", status, text));
    }

    let result: serde_json::Value = resp.json().await?;

    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
        for block in content {
            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                println!("{}", text);
            }
        }
    }

    Ok(())
}

async fn chat_openai_compat(client: &reqwest::Client, config: &AiConfig, user_message: &str) -> Result<()> {
    let url = format!("{}/v1/chat/completions", config.base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": config.model,
        "messages": [{
            "role": "user",
            "content": user_message
        }],
        "max_tokens": 4096
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API 错误 ({}): {}", status, text));
    }

    let result: serde_json::Value = resp.json().await?;

    if let Some(choices) = result.get("choices").and_then(|c| c.as_array()) {
        if let Some(choice) = choices.first() {
            if let Some(content) = choice.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                println!("{}", content);
            }
        }
    }

    Ok(())
}

pub fn reverse_with_ai(file_path: &str) -> Result<()> {
    let _config = load_config()
        .ok_or_else(|| anyhow!("未配置 AI, 请先运行: reptool ai setup"))?;

    let code = if std::path::Path::new(file_path).exists() {
        fs::read_to_string(file_path)?
    } else {
        file_path.to_string()
    };

    let prompt = format!(
        r#"你是一个专业的 Web 逆向工程师。请分析以下 JavaScript 代码，完成以下任务：

1. 识别混淆类型和程度
2. 找出所有加密/签名函数 (md5/sha/aes/rsa/hmac/sign/encrypt 等)
3. 分析参数生成逻辑
4. 提取密钥/盐值/IV 等常量
5. 还原加密函数的输入输出关系
6. 生成等效的 Python/Node.js 复现代码
7. 给出逆向建议

代码：
```javascript
{}
```

请用中文回答。"#,
        code
    );

    println!("{}", "正在调用 AI 分析...".blue().bold());
    println!();

    chat(&prompt)
}
