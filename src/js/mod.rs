use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::core::{self, Mode};
use crate::cli::js_cmd::DeobfTechnique;

pub fn format(file: &Path, output: Option<&Path>, minify: bool) -> Result<()> {
    let code = fs::read_to_string(file)?;

    let result = if minify {
        // 压缩: 移除多余空白
        let re = regex::Regex::new(r#"(?m)^\s*\n"#)?;
        let result = re.replace_all(&code, "\n");
        let re = regex::Regex::new(r#"\s+\n"#)?;
        let result = re.replace_all(&result, "\n");
        let re = regex::Regex::new(r#"\n\s*\n"#)?;
        let result = re.replace_all(&result, "\n");
        let re = regex::Regex::new(r#";\s+"#)?;
        let result = re.replace_all(&result, ";");
        result.to_string()
    } else {
        // 美化: 按 ; { } 换行缩进
        let mut result = String::new();
        let mut indent = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut escaped = false;

        for ch in code.chars() {
            if escaped {
                result.push(ch);
                escaped = false;
                continue;
            }
            if ch == '\\' && in_string {
                result.push(ch);
                escaped = true;
                continue;
            }
            if in_string {
                result.push(ch);
                if ch == string_char {
                    in_string = false;
                }
                continue;
            }
            match ch {
                '\'' | '"' | '`' => {
                    in_string = true;
                    string_char = ch;
                    result.push(ch);
                }
                '{' => {
                    result.push(ch);
                    indent += 1;
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent));
                }
                '}' => {
                    if indent > 0 {
                        indent -= 1;
                    }
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent));
                    result.push(ch);
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent));
                }
                ';' => {
                    result.push(ch);
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent));
                }
                ',' => {
                    result.push(ch);
                    if !in_string {
                        result.push('\n');
                        result.push_str(&"  ".repeat(indent));
                    } else {
                        result.push(ch);
                    }
                }
                _ => result.push(ch),
            }
        }
        result
    };

    let out_path = output.unwrap_or(file);
    fs::write(out_path, &result)?;
    core::success(&format!("格式化完成, 已保存到 {}", out_path.display()));
    Ok(())
}

pub fn deobfuscate(file: &Path, technique: &DeobfTechnique, output: Option<&Path>, _mode: &Mode) -> Result<()> {
    let code = fs::read_to_string(file)?;
    let mut result = code.clone();

    match technique {
        DeobfTechnique::Auto => {
            core::info("自动检测混淆类型...");

            if result.contains("eval(") || result.contains("eval ") {
                core::info("检测到 eval 混淆, 尝试展开...");
                result = expand_eval(&result)?;
            }

            let encrypted_count = count_encrypted_strings(&result);
            if encrypted_count > 0 {
                core::info(&format!("检测到 {} 个疑似加密字符串, 尝试解密...", encrypted_count));
                result = decrypt_strings(&result)?;
            }

            result = remove_dead_code(&result)?;

            if has_control_flow_flattening(&result) {
                core::info("检测到控制流平坦化, 进行基础还原...");
                result = restore_control_flow(&result)?;
            }
        }
        DeobfTechnique::Decrypt => result = decrypt_strings(&result)?,
        DeobfTechnique::ControlFlow => result = restore_control_flow(&result)?,
        DeobfTechnique::Eval => result = expand_eval(&result)?,
        DeobfTechnique::DeadCode => result = remove_dead_code(&result)?,
    }

    let out_path = output.unwrap_or(file);
    fs::write(out_path, &result)?;
    core::success(&format!("反混淆完成, 已保存到 {}", out_path.display()));
    Ok(())
}

pub fn scan_api(path: &Path, json_output: bool, _mode: &Mode) -> Result<()> {
    let files = if path.is_dir() {
        fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| {
                matches!(ext.to_str(), Some("js") | Some("ts") | Some("mjs") | Some("jsx") | Some("tsx"))
            }))
            .collect::<Vec<_>>()
    } else {
        vec![path.to_path_buf()]
    };

    let url_re = regex::Regex::new(r#"(?:https?://|/api/|/v\d+/|/servlet/)[\w\-/.?&=%+#@!]*"#)?;
    let fetch_re = regex::Regex::new(r#"(?:fetch|axios|request|\.get|\.post|\.put|\.delete|\.patch|\.ajax)\s*\(\s*['"`]([^'"`]+)['"`]"#)?;

    let mut apis: Vec<(String, String)> = Vec::new();
    let mut total_files = 0;

    for file in &files {
        if let Ok(code) = fs::read_to_string(file) {
            total_files += 1;
            let file_name = file.file_name().unwrap_or_default().to_string_lossy().to_string();

            for mat in url_re.find_iter(&code) {
                let url = mat.as_str().to_string();
                if !apis.iter().any(|a| a.0 == url) {
                    let ctx = get_context(&code, mat.start(), &file_name);
                    apis.push((url, ctx));
                }
            }
            for cap in fetch_re.captures_iter(&code) {
                if let Some(url) = cap.get(1) {
                    let url_str = url.as_str().to_string();
                    if !apis.iter().any(|a| a.0 == url_str) {
                        let ctx = get_context(&code, cap.get(0).unwrap().start(), &file_name);
                        apis.push((url_str, ctx));
                    }
                }
            }
        }
    }

    if json_output {
        let json_apis: Vec<serde_json::Value> = apis.iter().map(|(url, ctx)| {
            serde_json::json!({ "url": url, "context": ctx })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&json_apis)?);
    } else {
        core::info(&format!("扫描 {} 个文件, 共发现 {} 个接口", total_files, apis.len()));
        for (i, (url, ctx)) in apis.iter().enumerate() {
            println!("  [{}] {}", i + 1, url);
            if !ctx.is_empty() {
                println!("       {}", ctx.dimmed());
            }
        }
    }
    Ok(())
}

pub fn analyze_sign(file: &Path, functions: Option<&str>, json_output: bool, _mode: &Mode) -> Result<()> {
    let code = fs::read_to_string(file)?;
    let fns = functions.unwrap_or("sign,encrypt,getSign,makeSign,generateToken,calcSign,getXSign,getEncrypt");
    let keywords: Vec<&str> = fns.split(',').map(|s| s.trim()).collect();

    let mut results = Vec::new();

    for keyword in &keywords {
        let re = regex::Regex::new(&format!(
            r#"(?:function\s+{0}\s*\(|{0}\s*=\s*function\s*\(|{0}\s*=\s*\(|var\s+{0}\s*=\s*function|let\s+{0}\s*=\s*function|const\s+{0}\s*=\s*function)"#,
            regex::escape(keyword)
        ))?;

        for mat in re.find_iter(&code) {
            let start = mat.start();
            let end = (start + 800).min(code.len());
            let context = &code[start..end];

            let func_body = extract_function_body(context);
            let crypto_calls = find_crypto_calls(&func_body);
            let params = extract_function_params(context);

            results.push(serde_json::json!({
                "function": keyword,
                "position": start,
                "params": params,
                "crypto_calls": crypto_calls,
                "body_preview": func_body.chars().take(300).collect::<String>(),
            }));
        }
    }

    let key_re = regex::Regex::new(r#"(?:key|secret|salt|iv|appid|app_id|token|signKey|apiKey|SECRET|KEY)\s*[:=]\s*['"`]([^'"`]{3,})['"`]"#)?;
    let mut keys = Vec::new();
    for cap in key_re.captures_iter(&code) {
        if let Some(k) = cap.get(1) {
            let val = k.as_str().to_string();
            if !keys.contains(&val) && val.len() > 2 {
                keys.push(val);
            }
        }
    }

    if json_output {
        let output = serde_json::json!({ "functions": results, "keys": keys });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for r in &results {
            println!("\n{} {}:", "找到函数".green().bold(), r["function"]);
            println!("  位置: {}", r["position"]);
            println!("  参数: {}", r["params"]);
            if let Some(calls) = r["crypto_calls"].as_array() {
                if !calls.is_empty() {
                    println!("  加密调用: {}", calls.iter().filter_map(|c| c.as_str()).collect::<Vec<_>>().join(", "));
                }
            }
            println!("  预览: {}", r["body_preview"].as_str().unwrap_or("").dimmed());
        }
        if !keys.is_empty() {
            println!("\n{}", "发现常量密钥:".yellow().bold());
            for k in &keys { println!("  {}", k); }
        }
    }
    Ok(())
}

pub fn extract_keys(file: &Path, json_output: bool, _mode: &Mode) -> Result<()> {
    let code = fs::read_to_string(file)?;

    let patterns = vec![
        (r#"(?:var|let|const)\s+(?:\w*(?:key|KEY|Key)\w*)\s*=\s*['"`]([^'"`]+)['"`]"#, "密钥"),
        (r#"(?:var|let|const)\s+(?:\w*(?:iv|IV|Iv)\w*)\s*=\s*['"`]([^'"`]+)['"`]"#, "IV向量"),
        (r#"(?:var|let|const)\s+(?:\w*(?:secret|SECRET|Secret)\w*)\s*=\s*['"`]([^'"`]+)['"`]"#, "Secret"),
        (r#"(?:var|let|const)\s+(?:\w*(?:salt|SALT|Salt)\w*)\s*=\s*['"`]([^'"`]+)['"`]"#, "盐值"),
        (r#"(?:var|let|const)\s+(?:\w*(?:appid|APPID|AppId|app_id)\w*)\s*=\s*['"`]([^'"`]+)['"`]"#, "AppID"),
        (r#"(?:var|let|const)\s+(?:\w*(?:token|TOKEN|Token)\w*)\s*=\s*['"`]([^'"`]+)['"`]"#, "Token"),
        (r#"(?:key|secret|salt|iv)\s*[:=]\s*['"`]([A-Za-z0-9+/=_\-]{8,})['"`]"#, "长密钥"),
        (r#"['"`]([0-9a-fA-F]{32,64})['"`]"#, "可疑Hex串"),
        (r#"['"`]([A-Za-z0-9+/]{40,}={0,2})['"`]"#, "可疑Base64串"),
    ];

    let mut results: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for (pattern, name) in patterns {
        let re = regex::Regex::new(pattern)?;
        let values: Vec<String> = re.captures_iter(&code)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|v| !v.is_empty() && v.len() > 3 && !v.starts_with("http"))
            .collect();
        if !values.is_empty() {
            results.insert(name.to_string(), values);
        }
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for (name, values) in &results {
            println!("{}:", name);
            for v in values { println!("  {}", v); }
        }
    }
    Ok(())
}

pub fn hook_generate(function_name: &str, hook_type: &str, output: Option<&Path>) -> Result<()> {
    let hook = match hook_type {
        "sign" | "auto" => format!(r#"
// Reptool Hook - 签名函数拦截
(function() {{
    const target = '{function_name}';
    const original = window[target] || window[target.toLowerCase()] || window[target.toUpperCase()];
    if (typeof original === 'function') {{
        window[target] = function() {{
            const args = Array.from(arguments);
            console.log('%c[HOOK] {function_name} called:', 'color: #ff6600; font-weight: bold;', args);
            console.trace('%c[HOOK] Stack:', 'color: #ff6600;');
            const result = original.apply(this, arguments);
            console.log('%c[HOOK] {function_name} returned:', 'color: #00cc00; font-weight: bold;', result);
            return result;
        }};
        console.log('%c[Reptool] Hook {function_name} active', 'color: #00ff00; font-weight: bold;');
    }} else {{
        console.warn('[Reptool] Function not found: ' + target);
    }}
}})();
"#),
        "fetch" => r#"
// Reptool Hook - fetch + XHR interceptor
(function() {
    const origFetch = window.fetch;
    window.fetch = function() {
        const url = arguments[0]?.url || arguments[0];
        const opts = arguments[1] || {};
        console.log('%c[FETCH] ' + (opts.method||'GET') + ' ' + url, 'color: #cc00ff; font-weight: bold;');
        if (opts.body) console.log('%c[FETCH Body]', 'color: #cc00ff;', opts.body);
        if (opts.headers) console.log('%c[FETCH Headers]', 'color: #cc00ff;', opts.headers);
        return origFetch.apply(this, arguments).then(resp => {
            resp.clone().text().then(t => console.log('%c[FETCH Resp] ' + resp.status, 'color: #00cc00;', t.substring(0,500)));
            return resp;
        });
    };
    const origOpen = XMLHttpRequest.prototype.open;
    const origSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function(m, u) { this._m = m; this._u = u; return origOpen.apply(this, arguments); };
    XMLHttpRequest.prototype.send = function(body) {
        console.log('%c[XHR] ' + this._m + ' ' + this._u, 'color: #ff9900; font-weight: bold;');
        if (body) console.log('%c[XHR Body]', 'color: #ff9900;', body);
        const self = this;
        this.addEventListener('load', () => console.log('%c[XHR Resp] ' + self.status, 'color: #00cc00;', self.responseText?.substring(0,500)));
        return origSend.apply(this, arguments);
    };
    console.log('%c[Reptool] fetch+XHR Hook active', 'color: #00ff00; font-weight: bold; font-size: 14px;');
})();
"#.to_string(),
        _ => format!(r#"
// Reptool Hook - {function_name}
(function() {{
    const t = '{function_name}';
    if (typeof window[t] === 'function') {{
        const o = window[t];
        window[t] = function() {{
            console.log('%c[HOOK] {function_name}()', 'color: #ff6600;', ...arguments);
            const r = o.apply(this, arguments);
            console.log('%c[HOOK] →', 'color: #00cc00;', r);
            return r;
        }};
        console.log('%c[Reptool] Hook {function_name} active', 'color: #00ff00; font-weight: bold;');
    }}
}})();
"#),
    };

    match output {
        Some(o) => { fs::write(o, &hook)?; core::success(&format!("已保存到 {}", o.display())); }
        None => print!("{}", hook),
    }
    Ok(())
}

pub fn batch_scan(dir: &Path, json_output: bool, output: Option<&Path>, _mode: &Mode) -> Result<()> {
    let mut all_apis: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut all_keys = Vec::new();
    let mut all_crypto = Vec::new();

    let files: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| matches!(ext.to_str(), Some("js")|Some("ts")|Some("mjs"))))
        .collect();

    core::info(&format!("扫描 {} 个 JS 文件", files.len()));

    let url_re = regex::Regex::new(r#"(?:https?://|/api/|/v\d+/)[\w\-/.?&=%+#@!]*"#)?;
    let key_re = regex::Regex::new(r#"(?:key|secret|salt|iv)\s*[:=]\s*['"`]([^'"`]{4,})['"`]"#)?;
    let crypto_re = regex::Regex::new(r#"(?:md5|sha1|sha256|sha512|hmac|aes|des|rsa|encrypt|decrypt)\s*[:=]\s*(?:function|\()"#)?;

    for file in &files {
        if let Ok(code) = fs::read_to_string(file) {
            let name = file.file_name().unwrap_or_default().to_string_lossy().to_string();
            let mut file_apis = Vec::new();
            for mat in url_re.find_iter(&code) {
                let url = mat.as_str().to_string();
                if !file_apis.contains(&url) { file_apis.push(url); }
            }
            if !file_apis.is_empty() { all_apis.insert(name, file_apis); }
            for cap in key_re.captures_iter(&code) {
                if let Some(k) = cap.get(1) {
                    let v = k.as_str().to_string();
                    if !all_keys.contains(&v) { all_keys.push(v); }
                }
            }
            for mat in crypto_re.find_iter(&code) {
                let f = mat.as_str().to_string();
                if !all_crypto.contains(&f) { all_crypto.push(f); }
            }
        }
    }

    if json_output {
        let json = serde_json::json!({
            "apis": all_apis, "keys": all_keys, "crypto_functions": all_crypto
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        let total: usize = all_apis.values().map(|v| v.len()).sum();
        core::info(&format!("发现 {} 个接口, {} 个密钥, {} 个加密函数", total, all_keys.len(), all_crypto.len()));
        for (file, apis) in &all_apis {
            println!("\n{} ({} 个):", file.green().bold(), apis.len());
            for a in apis { println!("  {}", a); }
        }
        if !all_keys.is_empty() {
            println!("\n{}", "密钥:".yellow().bold());
            for k in &all_keys { println!("  {}", k); }
        }
        if !all_crypto.is_empty() {
            println!("\n{}", "加密函数:".cyan().bold());
            for f in &all_crypto { println!("  {}", f); }
        }
    }

    if let Some(out) = output {
        fs::write(out, serde_json::to_string_pretty(&serde_json::json!({
            "apis": all_apis, "keys": all_keys, "crypto_functions": all_crypto
        }))?)?;
        core::success(&format!("已保存到 {}", out.display()));
    }
    Ok(())
}

pub fn run_snippet(file: &Path, engine: &str, _args: Option<&str>, _url: Option<&str>, _debug: bool, _mode: &Mode) -> Result<()> {
    let _code = fs::read_to_string(file)?;
    match engine {
        "quickjs" => { core::info("QuickJS 暂未集成"); }
        "chrome" | "firefox" => { core::info(&format!("请使用: reptool render --engine {} --script {}", engine, file.display())); }
        _ => return Err(anyhow!("不支持的引擎: {}", engine)),
    }
    Ok(())
}

// ========== 内部工具函数 ==========

fn count_encrypted_strings(code: &str) -> usize {
    let mut count = 0;
    let arr_re = regex::Regex::new(r#"(?:var|let|const)\s+\w+\s*=\s*\[['"]\w{4,}['"](?:,\s*['"]\w{4,}['"])+\]"#).unwrap();
    if arr_re.is_match(code) { count += 10; }
    count += regex::Regex::new(r#"atob\s*\("#).unwrap().find_iter(code).count();
    count += regex::Regex::new(r#"String\.fromCharCode"#).unwrap().find_iter(code).count();
    let hex_count = regex::Regex::new(r#"\\x[0-9a-fA-F]{2}"#).unwrap().find_iter(code).count();
    if hex_count > 5 { count += hex_count / 3; }
    let uni_count = regex::Regex::new(r#"\\u[0-9a-fA-F]{4}"#).unwrap().find_iter(code).count();
    if uni_count > 5 { count += uni_count / 3; }
    count
}

fn has_control_flow_flattening(code: &str) -> bool {
    if regex::Regex::new(r#"switch\s*\(\s*\w+\s*\[\s*\+\+\s*\w+\s*\]\s*\)"#).unwrap().is_match(code) {
        return true;
    }
    if regex::Regex::new(r#"while\s*\(\s*!?\s*[]\d]*\s*\)\s*\{[^}]*switch"#).unwrap().is_match(code) {
        return true;
    }
    let case_count = code.matches("case '").count() + code.matches("case \"").count();
    case_count > 10
}

fn expand_eval(code: &str) -> Result<String> {
    let mut result = code.to_string();

    // eval(atob('...'))
    let re = regex::Regex::new(r#"eval\(\s*atob\(\s*['"`]([A-Za-z0-9+/=]+)['"`]\s*\)\s*\)"#)?;
    for cap in re.captures_iter(code) {
        if let Some(encoded) = cap.get(1) {
            use base64::Engine;
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded.as_str()) {
                if let Ok(text) = String::from_utf8(decoded) {
                    result = result.replace(cap.get(0).unwrap().as_str(), &format!("/* eval expanded */\n{}", text));
                }
            }
        }
    }

    // eval(decodeURIComponent(escape(atob('...'))))
    let re = regex::Regex::new(r#"eval\(\s*decodeURIComponent\(\s*escape\(\s*atob\(\s*['"`]([A-Za-z0-9+/=]+)['"`]\s*\)\s*\)\s*\)\s*\)"#)?;
    for cap in re.captures_iter(code) {
        if let Some(encoded) = cap.get(1) {
            use base64::Engine;
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded.as_str()) {
                if let Ok(text) = String::from_utf8(decoded) {
                    let unescaped = text.replace("\\x", "0x");
                    result = result.replace(cap.get(0).unwrap().as_str(), &format!("/* eval expanded */\n{}", unescaped));
                }
            }
        }
    }

    Ok(result)
}

fn decrypt_strings(code: &str) -> Result<String> {
    let mut result = code.to_string();

    // atob 解码
    use base64::Engine;
    let re = regex::Regex::new(r#"atob\(\s*['"`]([A-Za-z0-9+/=]+)['"`]\s*\)"#)?;
    for cap in re.captures_iter(code) {
        if let Some(encoded) = cap.get(1) {
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded.as_str()) {
                if let Ok(text) = String::from_utf8(decoded) {
                    if text.is_ascii() && text.len() < 2000 && !text.contains('\0') {
                        result = result.replace(cap.get(0).unwrap().as_str(), &format!("'{}'", text.replace('\'', "\\'")));
                    }
                }
            }
        }
    }

    // hex 转义 \x41 → A
    let mut decoded = String::new();
    let mut chars = result.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                if next == 'x' {
                    chars.next();
                    let hex_str: String = chars.by_ref().take(2).collect();
                    if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                        if byte >= 32 && byte < 127 {
                            decoded.push(byte as char);
                            continue;
                        }
                        decoded.push_str(&format!("\\x{}", hex_str));
                        continue;
                    }
                    decoded.push_str(&format!("\\x{}", hex_str));
                    continue;
                }
                if next == 'u' {
                    chars.next();
                    let hex_str: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex_str, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            decoded.push(ch);
                            continue;
                        }
                    }
                    decoded.push_str(&format!("\\u{}", hex_str));
                    continue;
                }
            }
        }
        decoded.push(c);
    }
    result = decoded;

    Ok(result)
}

fn remove_dead_code(code: &str) -> Result<String> {
    let mut result = code.to_string();
    let patterns = vec![
        r#"void\s*\(\s*0\s*\)\s*;?\s*\n?"#,
        r#"debugger\s*;?\s*\n?"#,
        r#"console\.\s*(?:log|debug|info)\s*\([^)]*\)\s*;?\s*\n?"#,
        r#"if\s*\(\s*!?\s*\d+\s*\)\s*\{\s*\}"#,
    ];
    for pattern in patterns {
        let re = regex::Regex::new(pattern)?;
        result = re.replace_all(&result, "").to_string();
    }
    let re = regex::Regex::new(r#";\s*;"#)?;
    result = re.replace_all(&result, ";").to_string();
    Ok(result)
}

fn restore_control_flow(code: &str) -> Result<String> {
    let result = code.to_string();
    let switch_re = regex::Regex::new(r#"switch\s*\(\s*\w+\s*\[\s*\+\+\s*\w+\s*\]\s*\)\s*\{"#)?;
    if switch_re.is_match(&result) {
        let case_re = regex::Regex::new(r#"case\s+['"](\d+)['"]\s*:"#)?;
        let cases: Vec<String> = case_re.captures_iter(&result)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();
        if !cases.is_empty() {
            core::info(&format!("检测到 {} 个 case, 顺序: {}", cases.len(), cases.join(" → ")));
        }
    }
    Ok(result)
}

fn get_context(code: &str, pos: usize, file: &str) -> String {
    let start = pos.saturating_sub(30);
    let end = (pos + 80).min(code.len());
    let context: String = code[start..end].chars().filter(|c| !c.is_control()).collect();
    format!("[{}] ...{}...", file, context.chars().take(100).collect::<String>())
}

fn extract_function_body(code: &str) -> String {
    if let Some(start) = code.find('{') {
        let mut depth = 0;
        for (i, c) in code[start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => { depth -= 1; if depth == 0 { return code[start..=start + i].to_string(); } }
                _ => {}
            }
        }
    }
    code.chars().take(500).collect()
}

fn extract_function_params(code: &str) -> String {
    if let Some(start) = code.find('(') {
        if let Some(end) = code[start..].find(')') {
            return code[start + 1..start + 1 + end].trim().to_string();
        }
    }
    String::new()
}

fn find_crypto_calls(code: &str) -> Vec<String> {
    regex::Regex::new(r#"(?:md5|sha1|sha256|sha512|hmac|aes|des|rsa|encrypt|decrypt|sign|hash)\s*\("#).unwrap()
        .find_iter(code)
        .map(|m| m.as_str().trim_end_matches('(').to_string())
        .collect()
}
