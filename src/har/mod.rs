use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::{self, Mode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Har {
    pub log: HarLog,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarLog {
    pub version: String,
    pub entries: Vec<HarEntry>,
    #[serde(default)]
    pub pages: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct HarEntry {
    #[serde(default)]
    pub startedDateTime: String,
    pub request: HarRequest,
    pub response: HarResponse,
    #[serde(default)]
    pub timings: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct HarRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<NameValuePair>,
    #[serde(default)]
    pub queryString: Vec<NameValuePair>,
    #[serde(default)]
    pub postData: Option<HarPostData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct HarResponse {
    pub status: u16,
    #[serde(default)]
    pub statusText: String,
    #[serde(default)]
    pub headers: Vec<NameValuePair>,
    #[serde(default)]
    pub content: HarContent,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[allow(non_snake_case)]
pub struct HarContent {
    #[serde(default)]
    pub size: usize,
    #[serde(default)]
    pub mimeType: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct HarPostData {
    #[serde(default)]
    pub mimeType: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub params: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NameValuePair {
    pub name: String,
    pub value: String,
}

pub fn load_har(path: &Path) -> Result<Har> {
    let content = fs::read_to_string(path)?;
    let har: Har = serde_json::from_str(&content)?;
    Ok(har)
}

pub fn parse(path: &Path, extract: Option<&[String]>, post_only: bool, regex: Option<&str>, json_output: bool, _mode: &Mode) -> Result<()> {
    let har = load_har(path)?;
    let re = regex.map(|r| regex::Regex::new(r)).transpose()?;

    let entries: Vec<&HarEntry> = har.log.entries.iter()
        .filter(|e| {
            if post_only && e.request.method != "POST" {
                return false;
            }
            if let Some(ref re) = re {
                if !re.is_match(&e.request.url) {
                    return false;
                }
            }
            true
        })
        .collect();

    core::info(&format!("共 {} 条匹配记录", entries.len()));

    let default_extracts = vec!["urls".to_string()];
    let extracts = extract.unwrap_or(&default_extracts);

    for (i, entry) in entries.iter().enumerate() {
        if json_output {
            println!("{}", serde_json::to_string(entry)?);
        } else {
            println!("[{}] {} {}", i, entry.request.method, entry.request.url);

            for ext in extracts {
                match ext.as_str() {
                    "cookies" => {
                        for h in &entry.request.headers {
                            if h.name.to_lowercase() == "cookie" {
                                println!("  Cookie: {}", h.value);
                            }
                        }
                    }
                    "headers" => {
                        for h in &entry.request.headers {
                            println!("  {}: {}", h.name, h.value);
                        }
                    }
                    "params" => {
                        for q in &entry.request.queryString {
                            println!("  {}={}", q.name, q.value);
                        }
                        if let Some(ref post) = entry.request.postData {
                            if let Some(ref text) = post.text {
                                println!("  Body: {}", text);
                            }
                        }
                    }
                    "apis" => {
                        println!("  API: {} {}", entry.request.method, entry.request.url);
                    }
                    _ => {}
                }
            }
            println!();
        }
    }

    Ok(())
}

pub fn diff(old_path: &Path, new_path: &Path, highlight_sign: bool) -> Result<()> {
    let old_har = load_har(old_path)?;
    let new_har = load_har(new_path)?;

    let old_urls: Vec<&str> = old_har.log.entries.iter().map(|e| e.request.url.as_str()).collect();
    let new_urls: Vec<&str> = new_har.log.entries.iter().map(|e| e.request.url.as_str()).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();

    for url in &new_urls {
        if !old_urls.contains(url) {
            added.push(url);
        }
    }
    for url in &old_urls {
        if !new_urls.contains(url) {
            removed.push(url);
        }
    }

    if added.is_empty() && removed.is_empty() {
        core::success("两份 HAR 接口列表相同");
    } else {
        if !added.is_empty() {
            println!("{}", "新增接口:".green().bold());
            for url in &added {
                println!("  + {}", url);
            }
        }
        if !removed.is_empty() {
            println!("{}", "移除接口:".red().bold());
            for url in &removed {
                println!("  - {}", url);
            }
        }
    }

    if highlight_sign {
        let sign_keywords = ["sign", "token", "encrypt", "signature", "x-sign", "authorization"];
        println!("\n{}", "签名/Token 字段变化:".yellow().bold());
        for new_entry in &new_har.log.entries {
            if let Some(old_entry) = old_har.log.entries.iter().find(|e| e.request.url == new_entry.request.url) {
                for h in &new_entry.request.headers {
                    if sign_keywords.iter().any(|k| h.name.to_lowercase().contains(k)) {
                        if let Some(old_h) = old_entry.request.headers.iter().find(|oh| oh.name == h.name) {
                            if old_h.value != h.value {
                                println!("  {} : '{}' → '{}'", h.name, old_h.value, h.value);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn extract_creds(path: &Path, output: Option<&Path>) -> Result<()> {
    let har = load_har(path)?;
    let cred_keywords = ["session", "token", "authorization", "cookie", "csrf", "x-sign", "x-token", "x-auth"];

    let mut creds: HashMap<String, String> = HashMap::new();

    for entry in &har.log.entries {
        for h in &entry.request.headers {
            let lower = h.name.to_lowercase();
            if cred_keywords.iter().any(|k| lower.contains(k)) {
                creds.insert(h.name.clone(), h.value.clone());
            }
        }
    }

    core::info(&format!("识别到 {} 个凭证字段", creds.len()));
    for (k, v) in &creds {
        println!("  {}: {}", k, v);
    }

    if let Some(out) = output {
        let json = serde_json::to_string_pretty(&creds)?;
        fs::write(out, json)?;
        core::success(&format!("已导出到 {}", out.display()));
    }

    Ok(())
}

pub fn replay(path: &Path, concurrency: usize, delay: Option<&u64>, proxy: Option<&str>, replace_token: Option<&str>, _output: Option<&Path>, _mode: &Mode) -> Result<()> {
    let har = load_har(path)?;

    core::info(&format!("准备重放 {} 条请求, 并发: {}", har.log.entries.len(), concurrency));

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(true);

        if let Some(p) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }

        let client = builder.build()?;

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut handles = Vec::new();

        let entries = har.log.entries.clone();

        for (i, entry) in entries.iter().enumerate() {
            let permit = semaphore.clone().acquire_owned().await?;
            let client = client.clone();
            let entry = entry.clone();
            let replace = replace_token.map(|s| s.to_string());

            handles.push(tokio::spawn(async move {
                let _permit = permit;

                let mut builder = match entry.request.method.as_str() {
                    "GET" => client.get(&entry.request.url),
                    "POST" => client.post(&entry.request.url),
                    "PUT" => client.put(&entry.request.url),
                    "DELETE" => client.delete(&entry.request.url),
                    "PATCH" => client.patch(&entry.request.url),
                    _ => client.request(reqwest::Method::from_bytes(entry.request.method.as_bytes())?, &entry.request.url),
                };

                for h in &entry.request.headers {
                    let mut value = h.value.clone();
                    if let Some(ref token) = replace {
                        if h.name.to_lowercase() == "authorization" {
                            value = value.replace(token.split(':').next().unwrap_or(""), token.split(':').last().unwrap_or(""));
                        }
                    }
                    builder = builder.header(&h.name, &value);
                }

                if let Some(ref post) = entry.request.postData {
                    if let Some(ref text) = post.text {
                        builder = builder.body(text.clone());
                    }
                }

                let resp = builder.send().await;
                let status = match &resp {
                    Ok(r) => r.status().as_u16(),
                    Err(_) => 0,
                };

                if status >= 400 {
                    core::warn(&format!("[{}] {} {} → {}", i, entry.request.method, entry.request.url, status));
                } else {
                    println!("[{}] {} {} → {}", i, entry.request.method, entry.request.url, status);
                }

                Ok::<(), anyhow::Error>(())
            }));

            if let Some(d) = delay {
                tokio::time::sleep(tokio::time::Duration::from_millis(*d)).await;
            }
        }

        for h in handles {
            let _ = h.await?;
        }

        core::success("重放完成");
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

pub fn filter(path: &Path, method: Option<&str>, regex: Option<&str>, body_regex: Option<&str>, response_regex: Option<&str>) -> Result<()> {
    let har = load_har(path)?;
    let url_re = regex.map(|r| regex::Regex::new(r)).transpose()?;
    let body_re = body_regex.map(|r| regex::Regex::new(r)).transpose()?;
    let resp_re = response_regex.map(|r| regex::Regex::new(r)).transpose()?;

    let filtered: Vec<&HarEntry> = har.log.entries.iter()
        .filter(|e| {
            if let Some(m) = method {
                if !e.request.method.eq_ignore_ascii_case(m) {
                    return false;
                }
            }
            if let Some(ref re) = url_re {
                if !re.is_match(&e.request.url) {
                    return false;
                }
            }
            if let Some(ref re) = body_re {
                let body_text = e.request.postData.as_ref()
                    .and_then(|p| p.text.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("");
                if !re.is_match(body_text) {
                    return false;
                }
            }
            if let Some(ref re) = resp_re {
                let resp_text = e.response.content.text.as_deref().unwrap_or("");
                if !re.is_match(resp_text) {
                    return false;
                }
            }
            true
        })
        .collect();

    core::info(&format!("过滤后 {} 条记录", filtered.len()));
    for (i, e) in filtered.iter().enumerate() {
        println!("[{}] {} {} → {}", i, e.request.method, e.request.url, e.response.status);
    }

    Ok(())
}

pub fn export(path: &Path, format: &crate::cli::har_cmd::ExportFormat, output: Option<&Path>, index: Option<usize>, session: bool) -> Result<()> {
    let har = load_har(path)?;

    let entries: Vec<&HarEntry> = match index {
        Some(i) => har.log.entries.get(i).into_iter().collect(),
        None => har.log.entries.iter().collect(),
    };

    if session {
        let mut cookies: HashMap<String, String> = HashMap::new();
        for entry in &entries {
            for h in &entry.request.headers {
                if h.name.to_lowercase() == "cookie" {
                    for part in h.value.split(';') {
                        if let Some((k, v)) = part.trim().split_once('=') {
                            cookies.insert(k.trim().to_string(), v.trim().to_string());
                        }
                    }
                }
            }
        }
        let json = serde_json::to_string_pretty(&cookies)?;
        match output {
            Some(o) => fs::write(o, json)?,
            None => println!("{}", json),
        }
        return Ok(());
    }

    let mut result = String::new();
    for (i, entry) in entries.iter().enumerate() {
        match format {
            crate::cli::har_cmd::ExportFormat::Curl => {
                result.push_str(&format!("curl -X {} '{}'", entry.request.method, entry.request.url));
                for h in &entry.request.headers {
                    result.push_str(&format!(" -H '{}: {}'", h.name, h.value));
                }
                if let Some(ref post) = entry.request.postData {
                    if let Some(ref text) = post.text {
                        result.push_str(&format!(" -d '{}'", text));
                    }
                }
                result.push('\n');
            }
            crate::cli::har_cmd::ExportFormat::Python => {
                result.push_str("import requests\n\n");
                result.push_str(&format!("response = requests.{}(\n", entry.request.method.to_lowercase()));
                result.push_str(&format!("    '{}',\n", entry.request.url));
                result.push_str("    headers={\n");
                for h in &entry.request.headers {
                    result.push_str(&format!("        '{}': '{}',\n", h.name, h.value));
                }
                result.push_str("    },\n");
                if let Some(ref post) = entry.request.postData {
                    if let Some(ref text) = post.text {
                        result.push_str(&format!("    data='{}',\n", text));
                    }
                }
                result.push_str(")\n");
            }
            crate::cli::har_cmd::ExportFormat::Java => {
                result.push_str("OkHttpClient client = new OkHttpClient();\n");
                result.push_str(&format!("Request request = new Request.Builder()\n  .url(\"{}\")\n", entry.request.url));
                result.push_str(&format!("  .{}(", entry.request.method.to_lowercase()));
                for h in &entry.request.headers {
                    result.push_str(&format!("\n    .addHeader(\"{}\", \"{}\")", h.name, h.value));
                }
                result.push_str(")\n  .build();\n");
            }
            crate::cli::har_cmd::ExportFormat::Go => {
                result.push_str(&format!("req, _ := http.NewRequest(\"{}\", \"{}\", nil)\n", entry.request.method, entry.request.url));
                for h in &entry.request.headers {
                    result.push_str(&format!("req.Header.Set(\"{}\", \"{}\")\n", h.name, h.value));
                }
            }
            crate::cli::har_cmd::ExportFormat::Fetch => {
                result.push_str(&format!("fetch('{}', {{\n  method: '{}',\n", entry.request.url, entry.request.method));
                result.push_str("  headers: {\n");
                for h in &entry.request.headers {
                    result.push_str(&format!("    '{}': '{}',\n", h.name, h.value));
                }
                result.push_str("  },\n");
                if let Some(ref post) = entry.request.postData {
                    if let Some(ref text) = post.text {
                        result.push_str(&format!("  body: '{}',\n", text));
                    }
                }
                result.push_str("})\n");
            }
            _ => {
                result.push_str(&format!("[{}] {} {}\n", i, entry.request.method, entry.request.url));
            }
        }
        result.push('\n');
    }

    match output {
        Some(o) => fs::write(o, &result)?,
        None => print!("{}", result),
    }

    Ok(())
}

pub fn multipart(path: &Path, index: usize, output: &Path) -> Result<()> {
    let har = load_har(path)?;
    let entry = har.log.entries.get(index).ok_or_else(|| anyhow!("索引 {} 超出范围", index))?;

    fs::create_dir_all(output)?;

    if let Some(ref post) = entry.request.postData {
        if let Some(ref params) = post.params {
            for (i, param) in params.iter().enumerate() {
                if let Some(obj) = param.as_object() {
                    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                        if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
                            let file_path = output.join(format!("{}_{}", i, name));
                            fs::write(&file_path, value)?;
                            core::info(&format!("已还原: {}", file_path.display()));
                        }
                    }
                }
            }
        }
        if let Some(ref text) = post.text {
            let file_path = output.join("body.txt");
            fs::write(&file_path, text)?;
            core::info(&format!("请求体已保存: {}", file_path.display()));
        }
    }

    core::success("multipart 还原完成");
    Ok(())
}
