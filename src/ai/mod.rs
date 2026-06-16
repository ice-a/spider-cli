use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;


pub fn analyze_js(file: &Path) -> Result<()> {
    let code = fs::read_to_string(file)?;
    let file_name = file.file_name().unwrap_or_default().to_string_lossy().to_string();

    println!("{}", format!("=== AI 逆向分析报告: {} ===", file_name).green().bold());
    println!();

    // 1. 混淆检测
    let obfuscation = detect_obfuscation(&code);
    println!("{}", "1. 混淆检测".yellow().bold());
    println!("   混淆类型: {}", obfuscation.type_name);
    println!("   混淆程度: {}", obfuscation.level);
    if !obfuscation.indicators.is_empty() {
        println!("   特征:");
        for ind in &obfuscation.indicators {
            println!("     - {}", ind);
        }
    }
    println!();

    // 2. 加密函数分析
    let crypto_funcs = find_crypto_functions(&code);
    println!("{}", "2. 加密函数分析".yellow().bold());
    if crypto_funcs.is_empty() {
        println!("   未发现明显加密函数");
    } else {
        for func in &crypto_funcs {
            println!("   {} ({})", func.name.green().bold(), func.func_type);
            println!("     位置: 第 {} 行", func.line);
            if !func.params.is_empty() {
                println!("     参数: {}", func.params);
            }
            if !func.calls.is_empty() {
                println!("     调用: {}", func.calls.join(", "));
            }
        }
    }
    println!();

    // 3. API 接口提取
    let apis = extract_api_endpoints(&code);
    println!("{}", "3. API 接口提取".yellow().bold());
    if apis.is_empty() {
        println!("   未发现 API 接口");
    } else {
        for (i, api) in apis.iter().enumerate() {
            println!("   [{}] {} ({})", i + 1, api.url, api.method);
        }
    }
    println!();

    // 4. 密钥/常量提取
    let secrets = extract_secrets(&code);
    println!("{}", "4. 密钥/常量提取".yellow().bold());
    if secrets.is_empty() {
        println!("   未发现明显密钥");
    } else {
        for s in &secrets {
            println!("   {} = {}", s.key.cyan(), s.value.dimmed());
        }
    }
    println!();

    // 5. 逆向建议
    println!("{}", "5. 逆向建议".yellow().bold());
    let suggestions = generate_suggestions(&obfuscation, &crypto_funcs, &apis, &secrets);
    for (i, sug) in suggestions.iter().enumerate() {
        println!("   {}. {}", i + 1, sug);
    }
    println!();

    println!("{}", "=== 分析完成 ===".green().bold());
    Ok(())
}

pub fn analyze_traffic(har_path: &Path) -> Result<()> {
    let har = crate::har::load_har(har_path)?;

    println!("{}", "=== AI 流量分析报告 ===".green().bold());
    println!();

    let entries = &har.log.entries;
    println!("总请求数: {}", entries.len());

    // 分析请求模式
    let mut methods = std::collections::HashMap::new();
    let mut content_types = std::collections::HashMap::new();
    let mut suspicious = Vec::new();

    for entry in entries {
        *methods.entry(entry.request.method.clone()).or_insert(0) += 1;

        for h in &entry.request.headers {
            if h.name.to_lowercase() == "content-type" {
                *content_types.entry(h.value.clone()).or_insert(0) += 1;
            }
        }

        // 检测可疑特征
        let url = &entry.request.url;
        if url.contains("sign") || url.contains("token") || url.contains("encrypt") {
            suspicious.push(format!("{} {} (含签名/token参数)", entry.request.method, url));
        }

        if let Some(ref post) = entry.request.postData {
            if let Some(ref text) = post.text {
                if text.contains("sign") || text.contains("token") || text.contains("encrypt") {
                    suspicious.push(format!("{} {} (请求体含签名字段)", entry.request.method, url));
                }
            }
        }
    }

    println!();
    println!("{}", "HTTP 方法分布:".yellow().bold());
    for (method, count) in &methods {
        println!("   {} = {}", method, count);
    }

    if !content_types.is_empty() {
        println!();
        println!("{}", "Content-Type 分布:".yellow().bold());
        for (ct, count) in &content_types {
            println!("   {} = {}", ct, count);
        }
    }

    if !suspicious.is_empty() {
        println!();
        println!("{}", "可疑接口 (含签名/加密):".red().bold());
        for s in &suspicious {
            println!("   ⚠ {}", s);
        }
    }

    // 分析参数模式
    println!();
    println!("{}", "参数分析:".yellow().bold());
    let mut all_param_keys = std::collections::HashMap::new();
    for entry in entries {
        for q in &entry.request.queryString {
            *all_param_keys.entry(q.name.clone()).or_insert(0) += 1;
        }
        if let Some(ref post) = entry.request.postData {
            if let Some(ref text) = post.text {
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(text) {
                    if let Some(map) = obj.as_object() {
                        for key in map.keys() {
                            *all_param_keys.entry(key.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let mut sorted_params: Vec<_> = all_param_keys.into_iter().collect();
    sorted_params.sort_by(|a, b| b.1.cmp(&a.1));

    for (key, count) in sorted_params.iter().take(20) {
        let marker = if key.contains("sign") || key.contains("token") || key.contains("encrypt") {
            " ← 签名字段".red().to_string()
        } else if key.contains("time") || key.contains("ts") {
            " ← 时间戳".yellow().to_string()
        } else if key.contains("nonce") || key.contains("random") {
            " ← 随机数".cyan().to_string()
        } else {
            String::new()
        };
        println!("   {} = {} 次{}", key, count, marker);
    }

    println!();
    println!("{}", "=== 分析完成 ===".green().bold());
    Ok(())
}

pub fn generate_report(js_files: &[std::path::PathBuf], har_files: &[std::path::PathBuf]) -> Result<()> {
    println!("{}", "=== 逆向分析报告 ===".green().bold());
    println!();

    if !js_files.is_empty() {
        println!("{}", "JS 文件分析:".yellow().bold());
        for file in js_files {
            if let Ok(code) = fs::read_to_string(file) {
                let name = file.file_name().unwrap_or_default().to_string_lossy();
                let crypto_count = find_crypto_functions(&code).len();
                let api_count = extract_api_endpoints(&code).len();
                let secret_count = extract_secrets(&code).len();
                println!("  {} (加密函数: {}, API: {}, 密钥: {})", name, crypto_count, api_count, secret_count);
            }
        }
    }

    if !har_files.is_empty() {
        println!();
        println!("{}", "HAR 文件分析:".yellow().bold());
        for file in har_files {
            if let Ok(har) = crate::har::load_har(file) {
                let name = file.file_name().unwrap_or_default().to_string_lossy();
                let entry_count = har.log.entries.len();
                let suspicious = har.log.entries.iter().filter(|e| {
                    e.request.url.contains("sign") || e.request.url.contains("token")
                }).count();
                println!("  {} (请求: {}, 可疑签名接口: {})", name, entry_count, suspicious);
            }
        }
    }

    println!();
    println!("{}", "=== 报告完成 ===".green().bold());
    Ok(())
}

// ========== 内部分析函数 ==========

struct ObfuscationInfo {
    type_name: String,
    level: String,
    indicators: Vec<String>,
}

fn detect_obfuscation(code: &str) -> ObfuscationInfo {
    let mut indicators = Vec::new();
    let mut score = 0;

    // 检测 _0x 前缀变量
    if regex::Regex::new(r#"_0x\w+"#).unwrap().is_match(code) {
        indicators.push("存在 _0x 前缀变量 (Obfuscator.io 特征)".to_string());
        score += 30;
    }

    // 检测字符串数组
    if regex::Regex::new(r#"(?:var|let|const)\s+\w+\s*=\s*\[['"]\w{4,}['"](?:,\s*['"]\w{4,}['"])+\]"#).unwrap().is_match(code) {
        indicators.push("存在字符串数组 (字符串加密)".to_string());
        score += 25;
    }

    // 检测 eval
    if code.contains("eval(") {
        indicators.push("存在 eval 调用".to_string());
        score += 20;
    }

    // 检测 fromCharCode
    if code.contains("String.fromCharCode") {
        indicators.push("存在 String.fromCharCode (字符编码)".to_string());
        score += 15;
    }

    // 检测控制流平坦化
    if regex::Regex::new(r#"switch\s*\(\s*\w+\s*\[\s*\+\+\s*\w+\s*\]\s*\)"#).unwrap().is_match(code) {
        indicators.push("存在 switch-case 控制流平坦化".to_string());
        score += 25;
    }

    // 检测 hex 转义
    let hex_count = code.matches("\\x").count();
    if hex_count > 10 {
        indicators.push(format!("大量 hex 转义序列 ({} 处)", hex_count));
        score += 15;
    }

    // 检测 base64
    if code.contains("atob(") || code.contains("btoa(") {
        indicators.push("存在 base64 编解码".to_string());
        score += 10;
    }

    // 检测 dead code
    if code.contains("void(0)") || code.contains("debugger") {
        indicators.push("存在 dead code (void/debugger)".to_string());
        score += 5;
    }

    let (type_name, level) = if score >= 60 {
        ("重度混淆 (Obfuscator.io 级别)".to_string(), "██████████ 100%".to_string())
    } else if score >= 40 {
        ("中度混淆".to_string(), format!("{}{}", "█".repeat(score / 10), "░".repeat(10 - score / 10)))
    } else if score >= 15 {
        ("轻度混淆".to_string(), format!("{}{}", "█".repeat(score / 10), "░".repeat(10 - score / 10)))
    } else {
        ("未混淆 / 原始代码".to_string(), "░░░░░░░░░░ 0%".to_string())
    };

    ObfuscationInfo { type_name, level, indicators }
}

struct CryptoFunc {
    name: String,
    func_type: String,
    line: usize,
    params: String,
    calls: Vec<String>,
}

fn find_crypto_functions(code: &str) -> Vec<CryptoFunc> {
    let mut results = Vec::new();

    let patterns = vec![
        (r#"function\s+(\w*(?:sign|Sign|encrypt|Encrypt|decrypt|Decrypt|hash|Hash|md5|sha|aes|rsa|hmac|getSign|makeSign|calcSign|generateToken)\w*)\s*\("#, "函数定义"),
        (r#"(\w*(?:sign|Sign|encrypt|Encrypt|decrypt|Decrypt)\w*)\s*=\s*function"#, "函数表达式"),
        (r#"(\w*(?:sign|Sign|encrypt|Encrypt|decrypt|Decrypt)\w*)\s*=\s*\("#, "箭头函数"),
        (r#"(\w*(?:sign|Sign|encrypt|Encrypt|decrypt|Decrypt)\w*)\s*=\s*(?:async\s+)?(?:\(|function)"#, "异步函数"),
    ];

    for (pattern, func_type) in patterns {
        let re = regex::Regex::new(pattern).unwrap();
        for cap in re.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                let pos = cap.get(0).unwrap().start();
                let line = code[..pos].lines().count() + 1;
                let context = &code[pos..(pos + 300).min(code.len())];
                let params = extract_params(context);
                let calls = extract_crypto_calls(context);

                if !results.iter().any(|r: &CryptoFunc| r.name == name.as_str()) {
                    results.push(CryptoFunc {
                        name: name.as_str().to_string(),
                        func_type: func_type.to_string(),
                        line,
                        params,
                        calls,
                    });
                }
            }
        }
    }

    results.sort_by(|a, b| a.line.cmp(&b.line));
    results
}

struct ApiEndpoint {
    url: String,
    method: String,
}

fn extract_api_endpoints(code: &str) -> Vec<ApiEndpoint> {
    let mut results = Vec::new();

    let url_re = regex::Regex::new(r#"(?:https?://|/api/|/v\d+/)[\w\-/.?&=%+#@!]*"#).unwrap();
    for mat in url_re.find_iter(code) {
        let url = mat.as_str().to_string();
        if !results.iter().any(|r: &ApiEndpoint| r.url == url) {
            results.push(ApiEndpoint { url, method: "GET".to_string() });
        }
    }

    let fetch_re = regex::Regex::new(r#"(\w+)\s*\.\s*(get|post|put|delete|patch)\s*\(\s*['"`]([^'"`]+)['"`]"#).unwrap();
    for cap in fetch_re.captures_iter(code) {
        if let (Some(method), Some(url)) = (cap.get(2), cap.get(3)) {
            let url_str = url.as_str().to_string();
            if !results.iter().any(|r: &ApiEndpoint| r.url == url_str) {
                results.push(ApiEndpoint {
                    url: url_str,
                    method: method.as_str().to_uppercase(),
                });
            }
        }
    }

    results
}

struct SecretItem {
    key: String,
    value: String,
}

fn extract_secrets(code: &str) -> Vec<SecretItem> {
    let mut results = Vec::new();

    let patterns = vec![
        r#"(?:key|secret|salt|iv|appid|token|apiKey|SECRET|KEY)\s*[:=]\s*['"`]([^'"`]{4,})['"`]"#,
        r#"(?:var|let|const)\s+\w*(?:key|Key|KEY|secret|Secret|SECRET)\w*\s*=\s*['"`]([^'"`]{4,})['"`]"#,
    ];

    for pattern in patterns {
        let re = regex::Regex::new(pattern).unwrap();
        for cap in re.captures_iter(code) {
            if let (Some(key_match), Some(val)) = (cap.get(0), cap.get(1)) {
                let key_str = key_match.as_str().split('=').next().unwrap_or("key").trim().to_string();
                let val_str = val.as_str().to_string();
                if !results.iter().any(|r: &SecretItem| r.value == val_str) && val_str.len() > 3 {
                    results.push(SecretItem { key: key_str, value: val_str });
                }
            }
        }
    }

    results
}

fn extract_params(code: &str) -> String {
    if let Some(start) = code.find('(') {
        if let Some(end) = code[start..].find(')') {
            return code[start + 1..start + 1 + end].trim().to_string();
        }
    }
    String::new()
}

fn extract_crypto_calls(code: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"(?:md5|sha1|sha256|sha512|hmac|aes|des|rsa|encrypt|decrypt|sign|hash)\s*\("#).unwrap();
    re.find_iter(code)
        .map(|m| m.as_str().trim_end_matches('(').to_string())
        .collect()
}

fn generate_suggestions(
    obf: &ObfuscationInfo,
    crypto: &[CryptoFunc],
    apis: &[ApiEndpoint],
    secrets: &[SecretItem],
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if obf.level.contains("100%") || obf.level.contains("80%") {
        suggestions.push("建议使用 reptool js deobfuscate --technique auto 进行自动反混淆".to_string());
    }

    if !crypto.is_empty() {
        suggestions.push(format!("发现 {} 个加密函数，建议使用 reptool js hook-generate 生成 hook 脚本在浏览器中拦截", crypto.len()));
    }

    if !secrets.is_empty() {
        suggestions.push(format!("发现 {} 个密钥/常量，可用于 reptool crypto 命令验证加密逻辑", secrets.len()));
    }

    if !apis.is_empty() {
        suggestions.push(format!("发现 {} 个 API 接口，建议使用 reptool proxy 抓包分析请求参数", apis.len()));
    }

    suggestions.push("建议使用 reptool proxy start --hook-fetch 启动代理并注入 hook 脚本".to_string());
    suggestions.push("建议将 HAR 文件导出后使用 reptool har parse 分析请求/响应模式".to_string());

    suggestions
}
