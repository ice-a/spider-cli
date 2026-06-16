use anyhow::{anyhow, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::core;

pub fn dns(domain: &str, _server: Option<&str>) -> Result<()> {
    use std::net::ToSocketAddrs;

    core::info(&format!("DNS 解析: {}", domain));

    let addrs = format!("{}:0", domain).to_socket_addrs()?;

    for addr in addrs {
        println!("  {}", addr.ip());
    }

    Ok(())
}

pub fn hosts(domain: &str, ip: &str, remove: bool) -> Result<()> {
    let hosts_path = if cfg!(target_os = "windows") {
        r"C:\Windows\System32\drivers\etc\hosts"
    } else {
        "/etc/hosts"
    };

    if remove {
        let content = fs::read_to_string(hosts_path)?;
        let new_content: String = content.lines()
            .filter(|line| !line.contains(domain))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(hosts_path, new_content)?;
        core::success(&format!("已移除 {} 的 hosts 映射", domain));
    } else {
        let entry = format!("{} {}", ip, domain);
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(hosts_path)?
            .write_all(format!("\n{}", entry).as_bytes())?;
        core::success(&format!("已添加 hosts: {} → {}", domain, ip));
    }

    Ok(())
}

pub fn port_check(port: u16) -> Result<()> {
    use std::net::TcpListener;

    match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(_) => {
            core::success(&format!("端口 {} 可用", port));
        }
        Err(_) => {
            core::warn(&format!("端口 {} 已被占用", port));
        }
    }

    Ok(())
}

pub fn speed(url: &str, count: usize) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let mut total_ms = 0u128;
        let mut success_count = 0;

        for i in 0..count {
            let start = std::time::Instant::now();
            match client.get(url).send().await {
                Ok(resp) => {
                    let elapsed = start.elapsed().as_millis();
                    total_ms += elapsed;
                    success_count += 1;
                    println!("[{}] {}ms → {}", i + 1, elapsed, resp.status());
                }
                Err(e) => {
                    println!("[{}] 失败: {}", i + 1, e);
                }
            }
        }

        if success_count > 0 {
            println!("\n平均延迟: {}ms", total_ms / success_count);
        }

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

pub fn json_format(file: Option<&Path>) -> Result<()> {
    let input = match file {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            buf
        }
    };

    let value: serde_json::Value = serde_json::from_str(&input)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

pub fn json_compact(file: Option<&Path>) -> Result<()> {
    let input = match file {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            buf
        }
    };

    let value: serde_json::Value = serde_json::from_str(&input)?;
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}

pub fn json_extract(file: &Path, path: &str) -> Result<()> {
    let content = fs::read_to_string(file)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;

    let parts: Vec<&str> = path.trim_start_matches('$').trim_start_matches('.').split('.').collect();
    let mut current = &value;

    for part in parts {
        if let Some(idx) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            if let Ok(i) = idx.parse::<usize>() {
                current = current.get(i).ok_or_else(|| anyhow!("索引 {} 不存在", i))?;
            }
        } else {
            current = current.get(part).ok_or_else(|| anyhow!("字段 '{}' 不存在", part))?;
        }
    }

    println!("{}", serde_json::to_string_pretty(current)?);
    Ok(())
}

pub fn json_merge(base: &Path, overlay: &Path, output: Option<&Path>) -> Result<()> {
    let base_content = fs::read_to_string(base)?;
    let overlay_content = fs::read_to_string(overlay)?;

    let mut base_val: serde_json::Value = serde_json::from_str(&base_content)?;
    let overlay_val: serde_json::Value = serde_json::from_str(&overlay_content)?;

    merge_json(&mut base_val, &overlay_val);

    let result = serde_json::to_string_pretty(&base_val)?;

    match output {
        Some(o) => {
            fs::write(o, &result)?;
            core::success(&format!("已保存到 {}", o.display()));
        }
        None => println!("{}", result),
    }

    Ok(())
}

fn merge_json(base: &mut serde_json::Value, overlay: &serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(overlay_map)) => {
            for (k, v) in overlay_map {
                merge_json(base_map.entry(k.clone()).or_insert(serde_json::Value::Null), v);
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}

pub fn csv_to_json(file: &Path, output: Option<&Path>) -> Result<()> {
    let content = fs::read_to_string(file)?;
    let mut lines = content.lines();

    let headers: Vec<&str> = lines.next()
        .ok_or_else(|| anyhow!("CSV 为空"))?
        .split(',')
        .map(|s| s.trim())
        .collect();

    let mut records = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let mut record = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            let value = values.get(i).map(|v| {
                if let Ok(n) = v.parse::<f64>() {
                    serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap())
                } else {
                    serde_json::Value::String(v.to_string())
                }
            }).unwrap_or(serde_json::Value::Null);
            record.insert(header.to_string(), value);
        }
        records.push(serde_json::Value::Object(record));
    }

    let result = serde_json::to_string_pretty(&records)?;

    match output {
        Some(o) => {
            fs::write(o, &result)?;
            core::success(&format!("已转换并保存到 {}", o.display()));
        }
        None => println!("{}", result),
    }

    Ok(())
}

pub fn encoding_detect(file: &Path) -> Result<()> {
    let data = fs::read(file)?;

    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        println!("UTF-8 BOM");
    } else if data.starts_with(&[0xFF, 0xFE]) {
        println!("UTF-16 LE");
    } else if data.starts_with(&[0xFE, 0xFF]) {
        println!("UTF-16 BE");
    } else if let Ok(_s) = std::str::from_utf8(&data) {
        println!("UTF-8 (无 BOM)");
    } else if let Some(_s) = strip_bom(&data) {
        println!("可能是 GBK/GB2312");
    } else {
        println!("二进制或未知编码");
    }

    println!("文件大小: {} 字节", data.len());
    Ok(())
}

fn strip_bom(data: &[u8]) -> Option<&[u8]> {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        Some(&data[3..])
    } else {
        None
    }
}
