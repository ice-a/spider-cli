use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::core::{self, Mode};

pub fn run(file: &Path, proxy: Option<&Path>, concurrency: usize, _delay: Option<&u64>, retries: u32, _mode: &Mode) -> Result<()> {
    let content = fs::read_to_string(file)?;
    let _config: serde_json::Value = toml::from_str(&content)?;

    core::info(&format!("加载任务文件: {}", file.display()));

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(true);

        if let Some(p) = proxy {
            let proxy_str = fs::read_to_string(p)?;
            let proxies: Vec<&str> = proxy_str.lines().filter(|l| !l.trim().is_empty()).collect();
            if !proxies.is_empty() {
                core::info(&format!("加载 {} 个代理", proxies.len()));
            }
        }

        let _client = builder.build()?;
        let _semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));

        core::success(&format!("开始执行, 并发: {}, 重试: {}", concurrency, retries));
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

pub fn http_single(
    url: &str,
    method: &str,
    headers: Option<&str>,
    body: Option<&str>,
    proxy: Option<&str>,
    cookie: Option<&str>,
    output: Option<&Path>,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(true);

        if let Some(p) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }

        let client = builder.build()?;
        let mut req = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => client.request(reqwest::Method::from_bytes(method.as_bytes())?, url),
        };

        if let Some(h) = headers {
            let map: serde_json::Value = serde_json::from_str(h)?;
            if let Some(obj) = map.as_object() {
                for (k, v) in obj {
                    req = req.header(k.as_str(), v.as_str().unwrap_or(""));
                }
            }
        }

        if let Some(c) = cookie {
            req = req.header("Cookie", c);
        }

        if let Some(b) = body {
            req = req.body(b.to_string());
        }

        let resp = req.send().await?;
        println!("Status: {}", resp.status());
        println!("Headers:");
        for (k, v) in resp.headers() {
            println!("  {}: {}", k, v.to_str().unwrap_or("?"));
        }

        let body = resp.text().await?;
        println!("\nBody:");
        println!("{}", body);

        if let Some(o) = output {
            fs::write(o, &body)?;
            core::success(&format!("已保存到 {}", o.display()));
        }

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

pub fn ws_connect(url: &str, message: Option<&str>, listen: bool) -> Result<()> {
    use futures_util::{SinkExt, StreamExt};

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();

        core::success(&format!("已连接 WebSocket: {}", url));

        if let Some(msg) = message {
            write.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await?;
            core::info(&format!("已发送: {}", msg));
        }

        if listen {
            core::info("监听中... (Ctrl+C 退出)");
            while let Some(Ok(msg)) = read.next().await {
                match msg {
                    tokio_tungstenite::tungstenite::Message::Text(text) => {
                        println!("← {}", text);
                    }
                    tokio_tungstenite::tungstenite::Message::Binary(bin) => {
                        println!("← [binary {} bytes]", bin.len());
                    }
                    _ => {}
                }
            }
        }

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

pub fn session_save(output: &Path, cookies: &[String], auth: Option<&str>, headers: Option<&str>) -> Result<()> {
    let mut session = serde_json::Map::new();

    let mut cookie_map = serde_json::Map::new();
    for c in cookies {
        if let Some((k, v)) = c.split_once('=') {
            cookie_map.insert(k.trim().to_string(), serde_json::Value::String(v.trim().to_string()));
        }
    }
    session.insert("cookies".to_string(), serde_json::Value::Object(cookie_map));

    if let Some(a) = auth {
        session.insert("authorization".to_string(), serde_json::Value::String(a.to_string()));
    }

    if let Some(h) = headers {
        let map: serde_json::Value = serde_json::from_str(h)?;
        session.insert("headers".to_string(), map);
    }

    fs::create_dir_all(output.parent().unwrap_or(Path::new(".")))?;
    fs::write(output, serde_json::to_string_pretty(&session)?)?;
    core::success(&format!("会话已保存到 {}", output.display()));

    Ok(())
}

pub fn session_load(file: &Path, url: Option<&str>) -> Result<()> {
    let content = fs::read_to_string(file)?;
    let session: serde_json::Value = serde_json::from_str(&content)?;

    println!("{}", serde_json::to_string_pretty(&session)?);

    if let Some(u) = url {
        core::info(&format!("使用会话请求: {}", u));
    }

    Ok(())
}

pub fn req_single(
    method: &str,
    url: &str,
    headers: &[String],
    body: Option<&str>,
    content_type: Option<&str>,
    cookie: Option<&str>,
    proxy: Option<&str>,
    timeout: u64,
    verbose: bool,
    output: Option<&Path>,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(timeout));

        if let Some(p) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }

        let client = builder.build()?;
        let mut req = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => client.request(reqwest::Method::from_bytes(method.as_bytes())?, url),
        };

        for h in headers {
            if let Some((k, v)) = h.split_once('=') {
                req = req.header(k.trim(), v.trim());
            }
        }

        if let Some(ct) = content_type {
            req = req.header("Content-Type", ct);
        }

        if let Some(c) = cookie {
            req = req.header("Cookie", c);
        }

        if let Some(b) = body {
            req = req.body(b.to_string());
        }

        let resp = req.send().await?;

        if verbose {
            println!("{} HTTP/1.1 {}", method, url);
            println!("Status: {}", resp.status());
            println!("Response Headers:");
            for (k, v) in resp.headers() {
                println!("  {}: {}", k, v.to_str().unwrap_or("?"));
            }
        } else {
            println!("Status: {}", resp.status());
        }

        let body = resp.text().await?;
        println!("{}", body);

        if let Some(o) = output {
            fs::write(o, &body)?;
            core::success(&format!("已保存到 {}", o.display()));
        }

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
