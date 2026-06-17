use anyhow::anyhow;
use anyhow::Result;
use serde_json::{json, Value};
use std::io::BufRead;

pub fn run() -> Result<()> {
    eprintln!("reptool MCP Server v0.1.0 启动 (stdio)");

    let stdin = std::io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(request) = serde_json::from_str::<Value>(trimmed) {
            let response = handle_request(&request);
            let output = serde_json::to_string(&response).unwrap_or_default();
            println!("{}", output);
        }
    }

    Ok(())
}

fn handle_request(request: &Value) -> Value {
    let method = request["method"].as_str().unwrap_or("");
    let id = request["id"].clone();

    match method {
        "initialize" => json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "reptool", "version": "0.1.0" }
            }
        }),
        "notifications/initialized" => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
        "tools/list" => json!({
            "jsonrpc": "2.0", "id": id,
            "result": { "tools": get_all_tools() }
        }),
        "tools/call" => {
            let tool_name = request["params"]["name"].as_str().unwrap_or("");
            let arguments = &request["params"]["arguments"];
            handle_tool_call(tool_name, arguments, id)
        }
        _ => json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32601, "message": format!("Unknown method: {}", method) }
        }),
    }
}

fn get_all_tools() -> Vec<Value> {
    vec![
        tool("har_parse", "解析 HAR 文件提取 cookies/headers/params/urls",
            json!({"type":"object","properties":{"path":{"type":"string"},"extract":{"type":"string"}},"required":["path"]})),
        tool("har_diff", "对比两份 HAR 文件差异",
            json!({"type":"object","properties":{"old":{"type":"string"},"new":{"type":"string"}},"required":["old","new"]})),
        tool("har_extract_creds", "自动识别 HAR 中的登录凭证",
            json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]})),
        tool("har_filter", "过滤 HAR 请求",
            json!({"type":"object","properties":{"path":{"type":"string"},"method":{"type":"string"},"regex":{"type":"string"}},"required":["path"]})),
        tool("har_export", "导出 HAR 为 curl/python/fetch/postman",
            json!({"type":"object","properties":{"path":{"type":"string"},"format":{"type":"string"}},"required":["path","format"]})),
        tool("js_format", "JS 代码格式化/美化",
            json!({"type":"object","properties":{"code":{"type":"string"},"file_path":{"type":"string"},"minify":{"type":"boolean"}}})),
        tool("js_deobfuscate", "JS 反混淆 (eval展开/字符串解密/死代码消除)",
            json!({"type":"object","properties":{"code":{"type":"string"},"file_path":{"type":"string"},"technique":{"type":"string","enum":["auto","decrypt","control_flow","eval","dead_code"]}}})),
        tool("js_extract_apis", "从 JS 提取接口 URL",
            json!({"type":"object","properties":{"file_path":{"type":"string"}},"required":["file_path"]})),
        tool("js_analyze_sign", "分析加密函数和签名逻辑",
            json!({"type":"object","properties":{"file_path":{"type":"string"},"functions":{"type":"string"}},"required":["file_path"]})),
        tool("js_extract_keys", "提取密钥/盐值/IV/AppID",
            json!({"type":"object","properties":{"file_path":{"type":"string"}},"required":["file_path"]})),
        tool("js_hook_generate", "生成 JS Hook 脚本",
            json!({"type":"object","properties":{"function_name":{"type":"string"},"hook_type":{"type":"string","enum":["sign","fetch","auto"]}},"required":["function_name"]})),
        tool("crypto_hash", "哈希计算 (md5/sha1/sha256/sha512)",
            json!({"type":"object","properties":{"algorithm":{"type":"string"},"input":{"type":"string"}},"required":["algorithm","input"]})),
        tool("crypto_hmac", "HMAC 签名",
            json!({"type":"object","properties":{"algorithm":{"type":"string"},"key":{"type":"string"},"data":{"type":"string"}},"required":["algorithm","key","data"]})),
        tool("crypto_encrypt", "对称加密 (aes-cbc/aes-ecb/aes-gcm/des/3des/rc4)",
            json!({"type":"object","properties":{"algorithm":{"type":"string"},"key":{"type":"string"},"data":{"type":"string"},"iv":{"type":"string"}},"required":["algorithm","key","data"]})),
        tool("crypto_decrypt", "对称解密",
            json!({"type":"object","properties":{"algorithm":{"type":"string"},"key":{"type":"string"},"data":{"type":"string"},"iv":{"type":"string"}},"required":["algorithm","key","data"]})),
        tool("crypto_base64", "Base64 编解码",
            json!({"type":"object","properties":{"action":{"type":"string","enum":["encode","decode"]},"data":{"type":"string"}},"required":["action","data"]})),
        tool("crypto_urlencode", "URL 编解码",
            json!({"type":"object","properties":{"action":{"type":"string","enum":["encode","decode"]},"data":{"type":"string"}},"required":["action","data"]})),
        tool("crypto_timestamp", "时间戳工具",
            json!({"type":"object","properties":{"bits":{"type":"string","enum":["10","13"]},"offset":{"type":"integer"}}})),
        tool("crypto_random_ua", "随机 User-Agent",
            json!({"type":"object","properties":{"browser":{"type":"string","enum":["chrome","firefox","safari","all"]},"count":{"type":"integer"}}})),
        tool("calc_sign_sort", "参数字典排序签名 (升序拼接+盐值+md5)",
            json!({"type":"object","properties":{"params":{"type":"string"},"salt":{"type":"string"},"algorithm":{"type":"string"}},"required":["params"]})),
        tool("calc_step", "分步加密计算器",
            json!({"type":"object","properties":{"steps":{"type":"array","items":{"type":"string"}},"data":{"type":"string"}},"required":["steps"]})),
        tool("calc_diff_sign", "签名差异对比",
            json!({"type":"object","properties":{"src_str":{"type":"string"},"src_sign":{"type":"string"},"dst_str":{"type":"string"},"dst_sign":{"type":"string"}},"required":["src_str","src_sign","dst_str","dst_sign"]})),
        tool("proxy_start", "启动 MITM 代理",
            json!({"type":"object","properties":{"port":{"type":"integer"},"filter":{"type":"string"},"callback_url":{"type":"string"},"hook_fetch":{"type":"boolean"}}})),
        tool("proxy_stop", "停止代理", json!({"type":"object","properties":{}})),
        tool("proxy_get_sessions", "获取抓包会话",
            json!({"type":"object","properties":{"filter":{"type":"string"},"limit":{"type":"integer"}}})),
        tool("crawl_http", "发送 HTTP 请求",
            json!({"type":"object","properties":{"url":{"type":"string"},"method":{"type":"string"},"headers":{"type":"object"},"body":{"type":"string"}},"required":["url"]})),
        tool("crawl_ws_connect", "WebSocket 连接",
            json!({"type":"object","properties":{"url":{"type":"string"},"message":{"type":"string"},"listen":{"type":"boolean"}},"required":["url"]})),
        tool("export_curl", "生成 curl 命令",
            json!({"type":"object","properties":{"url":{"type":"string"},"method":{"type":"string"},"headers":{"type":"object"},"body":{"type":"string"}},"required":["url"]})),
        tool("export_python", "生成 Python requests 代码",
            json!({"type":"object","properties":{"url":{"type":"string"},"method":{"type":"string"},"headers":{"type":"object"},"body":{"type":"string"}},"required":["url"]})),
        tool("export_fetch", "生成 JS fetch 代码",
            json!({"type":"object","properties":{"url":{"type":"string"},"method":{"type":"string"},"headers":{"type":"object"},"body":{"type":"string"}},"required":["url"]})),
        tool("export_go", "生成 Go reqwest 代码",
            json!({"type":"object","properties":{"url":{"type":"string"},"method":{"type":"string"},"headers":{"type":"object"},"body":{"type":"string"}},"required":["url"]})),
        tool("tools_dns", "DNS 解析",
            json!({"type":"object","properties":{"domain":{"type":"string"}},"required":["domain"]})),
        tool("tools_port_check", "端口占用检测",
            json!({"type":"object","properties":{"port":{"type":"integer"}},"required":["port"]})),
        tool("tools_json_format", "JSON 格式化",
            json!({"type":"object","properties":{"data":{"type":"string"}},"required":["data"]})),
        tool("tools_json_extract", "JSON 路径提取",
            json!({"type":"object","properties":{"data":{"type":"string"},"path":{"type":"string"}},"required":["data","path"]})),
        tool("mini_wxapkg_parse", "解析微信小程序 wxapkg",
            json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]})),
        tool("mini_apk_extract", "APK 提取 URL/密钥",
            json!({"type":"object","properties":{"path":{"type":"string"},"extract_urls":{"type":"boolean"},"extract_keys":{"type":"boolean"}},"required":["path"]})),
        tool("proto_decode", "Protobuf 解码",
            json!({"type":"object","properties":{"path":{"type":"string"},"schema":{"type":"string"}},"required":["path"]})),
        tool("config_get", "读取配置",
            json!({"type":"object","properties":{"key":{"type":"string"}},"required":["key"]})),
        tool("config_set", "设置配置",
            json!({"type":"object","properties":{"key":{"type":"string"},"value":{"type":"string"}},"required":["key","value"]})),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

fn handle_tool_call(tool_name: &str, args: &Value, id: Value) -> Value {
    let result = match tool_name {
        // === HAR ===
        "har_parse" => {
            let path = args["path"].as_str().unwrap_or("");
            let extract = args["extract"].as_str().map(|s| vec![s.to_string()]);
            call_result(crate::har::parse(
                std::path::Path::new(path), extract.as_deref(), false, None, true, &crate::core::Mode::Auto,
            ))
        }
        "har_diff" => {
            let old = args["old"].as_str().unwrap_or("");
            let new = args["new"].as_str().unwrap_or("");
            call_result(crate::har::diff(std::path::Path::new(old), std::path::Path::new(new), true))
        }
        "har_extract_creds" => {
            let path = args["path"].as_str().unwrap_or("");
            call_result(crate::har::extract_creds(std::path::Path::new(path), None))
        }
        "har_filter" => {
            let path = args["path"].as_str().unwrap_or("");
            let method = args["method"].as_str();
            let regex = args["regex"].as_str();
            call_result(crate::har::filter(std::path::Path::new(path), method, regex, None, None))
        }
        "har_export" => {
            let path = args["path"].as_str().unwrap_or("");
            let fmt = match args["format"].as_str().unwrap_or("curl") {
                "python" => crate::export::ExportFormat::Python,
                "fetch" => crate::export::ExportFormat::Fetch,
                "go" => crate::export::ExportFormat::Go,
                "java" => crate::export::ExportFormat::Java,
                "postman" => crate::export::ExportFormat::Postman,
                _ => crate::export::ExportFormat::Curl,
            };
            call_result(crate::export::run(std::path::Path::new(path), &fmt, None, None, false, "python"))
        }

        // === JS ===
        "js_format" => {
            let code = args["code"].as_str();
            let file = args["file_path"].as_str();
            let minify = args["minify"].as_bool().unwrap_or(false);
            match (code, file) {
                (Some(c), _) => {
                    let tmp = std::env::temp_dir().join("_reptool_fmt.js");
                    std::fs::write(&tmp, c).ok();
                    let r = crate::js::format(&tmp, Some(&tmp), minify);
                    let output = std::fs::read_to_string(&tmp).unwrap_or_default();
                    let _ = std::fs::remove_file(&tmp);
                    match r {
                        Ok(()) => text_result(output),
                        Err(e) => error_result(e),
                    }
                }
                (_, Some(f)) => call_result(crate::js::format(std::path::Path::new(f), None, minify)),
                _ => error_result(anyhow!("需要 code 或 file_path")),
            }
        }
        "js_deobfuscate" => {
            let file = args["file_path"].as_str().or(args["code"].as_str().and_then(|_| None));
            let technique = match args["technique"].as_str().unwrap_or("auto") {
                "decrypt" => crate::cli::js_cmd::DeobfTechnique::Decrypt,
                "control_flow" => crate::cli::js_cmd::DeobfTechnique::ControlFlow,
                "eval" => crate::cli::js_cmd::DeobfTechnique::Eval,
                "dead_code" => crate::cli::js_cmd::DeobfTechnique::DeadCode,
                _ => crate::cli::js_cmd::DeobfTechnique::Auto,
            };
            if let Some(f) = file {
                call_result(crate::js::deobfuscate(std::path::Path::new(f), &technique, None, &crate::core::Mode::Auto))
            } else if let Some(code) = args["code"].as_str() {
                let tmp = std::env::temp_dir().join("_reptool_deobf.js");
                std::fs::write(&tmp, code).ok();
                let r = crate::js::deobfuscate(&tmp, &technique, Some(&tmp), &crate::core::Mode::Auto);
                let output = std::fs::read_to_string(&tmp).unwrap_or_default();
                let _ = std::fs::remove_file(&tmp);
                match r {
                    Ok(()) => text_result(output),
                    Err(e) => error_result(e),
                }
            } else {
                error_result(anyhow!("需要 code 或 file_path"))
            }
        }
        "js_extract_apis" => {
            let file = args["file_path"].as_str().unwrap_or("");
            call_result(crate::js::scan_api(std::path::Path::new(file), true, &crate::core::Mode::Auto))
        }
        "js_analyze_sign" => {
            let file = args["file_path"].as_str().unwrap_or("");
            let funcs = args["functions"].as_str();
            call_result(crate::js::analyze_sign(std::path::Path::new(file), funcs, true, &crate::core::Mode::Auto))
        }
        "js_extract_keys" => {
            let file = args["file_path"].as_str().unwrap_or("");
            call_result(crate::js::extract_keys(std::path::Path::new(file), true, &crate::core::Mode::Auto))
        }
        "js_hook_generate" => {
            let name = args["function_name"].as_str().unwrap_or("hook");
            let hook_type = args["hook_type"].as_str().unwrap_or("sign");
            let tmp = std::env::temp_dir().join("_reptool_hook.js");
            let r = crate::js::hook_generate(name, hook_type, Some(&tmp));
            match r {
                Ok(()) => {
                    let output = std::fs::read_to_string(&tmp).unwrap_or_default();
                    let _ = std::fs::remove_file(&tmp);
                    text_result(output)
                }
                Err(e) => error_result(e),
            }
        }

        // === Crypto ===
        "crypto_hash" => {
            let algo = args["algorithm"].as_str().unwrap_or("md5");
            let input = args["input"].as_str().unwrap_or("");
            call_result(crate::crypto::hash(algo, input, None, "hex"))
        }
        "crypto_hmac" => {
            let algo = args["algorithm"].as_str().unwrap_or("sha256");
            let key = args["key"].as_str().unwrap_or("");
            let data = args["data"].as_str().unwrap_or("");
            call_result(crate::crypto::hmac_sign(algo, key, data, "hex"))
        }
        "crypto_encrypt" => {
            let algo = args["algorithm"].as_str().unwrap_or("aes-cbc");
            let key = args["key"].as_str().unwrap_or("");
            let data = args["data"].as_str().unwrap_or("");
            let iv = args["iv"].as_str();
            call_result(crate::crypto::encrypt(algo, key, iv, data, "text", "hex"))
        }
        "crypto_decrypt" => {
            let algo = args["algorithm"].as_str().unwrap_or("aes-cbc");
            let key = args["key"].as_str().unwrap_or("");
            let data = args["data"].as_str().unwrap_or("");
            let iv = args["iv"].as_str();
            call_result(crate::crypto::decrypt(algo, key, iv, data, "hex", "text"))
        }
        "crypto_base64" => {
            let action = args["action"].as_str().unwrap_or("encode");
            let data = args["data"].as_str().unwrap_or("");
            match action {
                "decode" => call_result(crate::calc::base64_decode(data)),
                _ => call_result(crate::calc::base64_encode(data)),
            }
        }
        "crypto_urlencode" => {
            let action = args["action"].as_str().unwrap_or("encode");
            let data = args["data"].as_str().unwrap_or("");
            match action {
                "decode" => call_result(crate::calc::url_decode(data)),
                _ => call_result(crate::calc::url_encode(data)),
            }
        }
        "crypto_timestamp" => {
            let bits = args["bits"].as_str().unwrap_or("13");
            let offset = args["offset"].as_i64().unwrap_or(0);
            call_result(crate::calc::timestamp(None, offset, bits, "%Y-%m-%d %H:%M:%S"))
        }
        "crypto_random_ua" => {
            let browser = args["browser"].as_str().unwrap_or("all");
            let count = args["count"].as_u64().unwrap_or(1) as usize;
            call_result(crate::crypto::random_ua(browser, count))
        }

        // === Calc ===
        "calc_sign_sort" => {
            let params = args["params"].as_str().unwrap_or("{}");
            let salt = args["salt"].as_str();
            let algo = args["algorithm"].as_str().unwrap_or("md5");
            call_result(crate::calc::sign_sort(params, salt, algo, "=", "&", false))
        }
        "calc_step" => {
            let steps: Vec<String> = args["steps"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let data = args["data"].as_str();
            call_result(crate::calc::step_calc(&steps, data, "hex"))
        }
        "calc_diff_sign" => {
            let src_str = args["src_str"].as_str().unwrap_or("");
            let src_sign = args["src_sign"].as_str().unwrap_or("");
            let dst_str = args["dst_str"].as_str().unwrap_or("");
            let dst_sign = args["dst_sign"].as_str().unwrap_or("");
            call_result(crate::calc::diff_sign(src_str, src_sign, dst_str, dst_sign))
        }

        // === Proxy ===
        "proxy_start" => {
            let port = args["port"].as_u64().unwrap_or(8080) as u16;
            let filter = args["filter"].as_str().map(String::from);
            let callback = args["callback_url"].as_str().map(String::from);
            let hook = args["hook_fetch"].as_bool().unwrap_or(false);
            call_result(crate::proxy::start(crate::proxy::ProxyConfig {
                port, filter, filter_mode: "whitelist".into(),
                callback_url: callback, callback_timeout: 5, callback_retries: 3,
                hook_fetch: hook, hook_function: None, no_static: false, mode: crate::core::Mode::Auto,
            }))
        }
        "proxy_stop" => call_result(crate::proxy::stop()),
        "proxy_get_sessions" => {
            let filter = args["filter"].as_str();
            let limit = args["limit"].as_u64().unwrap_or(50) as usize;
            call_result(crate::proxy::show_sessions(filter, limit))
        }

        // === Crawl ===
        "crawl_http" => {
            let url = args["url"].as_str().unwrap_or("");
            let method = args["method"].as_str().unwrap_or("GET");
            let headers = args["headers"].as_str();
            let body = args["body"].as_str();
            call_result(crate::crawl::http_single(url, method, headers, body, None, None, None))
        }
        "crawl_ws_connect" => {
            let url = args["url"].as_str().unwrap_or("");
            let message = args["message"].as_str();
            let listen = args["listen"].as_bool().unwrap_or(false);
            call_result(crate::crawl::ws_connect(url, message, listen))
        }

        // === Export ===
        "export_curl" => export_to("curl", args),
        "export_python" => export_to("python", args),
        "export_fetch" => export_to("fetch", args),
        "export_go" => export_to("go", args),

        // === Tools ===
        "tools_dns" => {
            let domain = args["domain"].as_str().unwrap_or("");
            call_result(crate::tools::dns(domain, None))
        }
        "tools_port_check" => {
            let port = args["port"].as_u64().unwrap_or(8080) as u16;
            call_result(crate::tools::port_check(port))
        }
        "tools_json_format" => {
            let data = args["data"].as_str().unwrap_or("{}");
            match serde_json::from_str::<serde_json::Value>(data) {
                Ok(v) => text_result(serde_json::to_string_pretty(&v).unwrap_or_default()),
                Err(e) => error_result(anyhow!("JSON 解析错误: {}", e)),
            }
        }
        "tools_json_extract" => {
            let data = args["data"].as_str().unwrap_or("{}");
            let path = args["path"].as_str().unwrap_or("$");
            let tmp = std::env::temp_dir().join("_reptool.json");
            std::fs::write(&tmp, data).ok();
            let r = crate::tools::json_extract(&tmp, path);
            let _ = std::fs::remove_file(&tmp);
            call_result(r)
        }

        // === Mini ===
        "mini_wxapkg_parse" => {
            let path = args["path"].as_str().unwrap_or("");
            let out = std::env::temp_dir().join("_reptool_wxapkg");
            call_result(crate::app_reverse::unpack_wxapkg(std::path::Path::new(path), &out, true))
        }
        "mini_apk_extract" => {
            let path = args["path"].as_str().unwrap_or("");
            let urls = args["extract_urls"].as_bool().unwrap_or(true);
            let keys = args["extract_keys"].as_bool().unwrap_or(true);
            call_result(crate::app_reverse::parse_apk(std::path::Path::new(path), urls, keys))
        }
        "proto_decode" => {
            let path = args["path"].as_str().unwrap_or("");
            let schema = args["schema"].as_str().map(std::path::PathBuf::from);
            call_result(crate::app_reverse::proto_decode(std::path::Path::new(path), schema.as_deref(), "pretty"))
        }

        // === Config ===
        "config_get" => {
            let key = args["key"].as_str().unwrap_or("");
            call_result(crate::config::get(key))
        }
        "config_set" => {
            let key = args["key"].as_str().unwrap_or("");
            let value = args["value"].as_str().unwrap_or("");
            call_result(crate::config::set(key, value))
        }

        _ => error_result(anyhow!("未知工具: {}", tool_name)),
    };

    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn export_to(format: &str, args: &Value) -> Value {
    let url = args["url"].as_str().unwrap_or("");
    let method = args["method"].as_str().unwrap_or("GET");
    let headers = args["headers"].as_object();
    let body = args["body"].as_str();

    let output = match format {
        "python" => {
            let mut code = format!("import requests\n\nresponse = requests.{}(\n    '{}',\n", method.to_lowercase(), url);
            if let Some(h) = headers {
                code.push_str("    headers={\n");
                for (k, v) in h {
                    code.push_str(&format!("        '{}': '{}',\n", k, v.as_str().unwrap_or("")));
                }
                code.push_str("    },\n");
            }
            if let Some(b) = body {
                code.push_str(&format!("    data='{}',\n", b));
            }
            code.push_str(")\nprint(response.status_code)\nprint(response.text)\n");
            code
        }
        "go" => {
            format!("req, _ := http.NewRequest(\"{}\", \"{}\", nil)\n", method, url)
        }
        "fetch" => {
            let mut code = format!("fetch('{}', {{\n  method: '{}',\n", url, method);
            if let Some(h) = headers {
                code.push_str("  headers: {\n");
                for (k, v) in h {
                    code.push_str(&format!("    '{}': '{}',\n", k, v.as_str().unwrap_or("")));
                }
                code.push_str("  },\n");
            }
            if let Some(b) = body {
                code.push_str(&format!("  body: '{}',\n", b));
            }
            code.push_str("}).then(r => r.json()).then(console.log)\n");
            code
        }
        _ => {
            let mut cmd = format!("curl -X {} '{}'", method, url);
            if let Some(h) = headers {
                for (k, v) in h {
                    cmd.push_str(&format!(" -H '{}: {}'", k, v.as_str().unwrap_or("")));
                }
            }
            if let Some(b) = body {
                cmd.push_str(&format!(" -d '{}'", b));
            }
            cmd
        }
    };

    text_result(output)
}

fn call_result(r: Result<()>) -> Value {
    match r {
        Ok(()) => json!({ "content": [{ "type": "text", "text": "完成" }] }),
        Err(e) => json!({ "content": [{ "type": "text", "text": format!("错误: {}", e) }] }),
    }
}

fn text_result(text: String) -> Value {
    json!({ "content": [{ "type": "text", "text": text }] })
}

fn error_result(e: anyhow::Error) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("错误: {}", e) }] })
}
