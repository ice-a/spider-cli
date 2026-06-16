use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::core;

pub fn params_editor(manual: bool, params: &[String], sort: &str, order: Option<&str>, skip_empty: bool, output_format: &str) -> Result<()> {
    let mut entries: Vec<(String, String)> = if manual || params.is_empty() {
        core::info("手动输入参数 (格式: key=value, 空行结束):");
        let mut entries = Vec::new();
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            let line = line.trim().to_string();
            if line.is_empty() {
                break;
            }
            if let Some((k, v)) = line.split_once('=') {
                entries.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        entries
    } else {
        params.iter().filter_map(|p| {
            let (k, v) = p.split_once('=')?;
            Some((k.trim().to_string(), v.trim().to_string()))
        }).collect()
    };

    if skip_empty {
        entries.retain(|(_, v)| !v.is_empty());
    }

    match sort {
        "asc" => entries.sort_by(|a, b| a.0.cmp(&b.0)),
        "desc" => entries.sort_by(|a, b| b.0.cmp(&a.0)),
        "custom" => {
            if let Some(order_str) = order {
                let order_keys: Vec<&str> = order_str.split(',').map(|s| s.trim()).collect();
                entries.sort_by(|a, b| {
                    let pos_a = order_keys.iter().position(|&k| k == a.0).unwrap_or(usize::MAX);
                    let pos_b = order_keys.iter().position(|&k| k == b.0).unwrap_or(usize::MAX);
                    pos_a.cmp(&pos_b)
                });
            }
        }
        _ => {}
    }

    match output_format {
        "url" => {
            let s: String = entries.iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            println!("{}", s);
        }
        "json" => {
            let map: HashMap<&str, &str> = entries.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            println!("{}", serde_json::to_string_pretty(&map)?);
        }
        "form" => {
            let s: String = entries.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            println!("{}", s);
        }
        _ => {
            for (k, v) in &entries {
                println!("{}={}", k, v);
            }
        }
    }
    Ok(())
}

pub fn step_calc(steps: &[String], data: Option<&str>, output_format: &str) -> Result<()> {
    use md5::Digest;
    use sha1::Sha1;
    use sha2::Sha256;

    let mut current = data.unwrap_or("").as_bytes().to_vec();

    for (i, step) in steps.iter().enumerate() {
        let parts: Vec<&str> = step.splitn(2, ':').collect();
        let op = parts[0];
        let arg = parts.get(1).copied();

        let result = match op {
            "sort-params" => {
                let json_str = arg.unwrap_or("{}");
                let map: HashMap<String, String> = serde_json::from_str(json_str)?;
                let mut sorted: Vec<_> = map.into_iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let s = sorted.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("&");
                core::info(&format!("Step {} - sort-params: {}", i + 1, s));
                s.into_bytes()
            }
            "concat-salt" => {
                let salt = arg.unwrap_or("");
                let s = format!("{}{}", String::from_utf8_lossy(&current), salt);
                core::info(&format!("Step {} - concat-salt: {}", i + 1, s));
                s.into_bytes()
            }
            "md5" => {
                let mut hasher = <md5::Md5 as Digest>::new();
                Digest::update(&mut hasher, &current);
                let hash = hex::encode(hasher.finalize());
                core::info(&format!("Step {} - md5: {}", i + 1, hash));
                hash.into_bytes()
            }
            "sha1" => {
                let mut hasher = Sha1::new();
                Digest::update(&mut hasher, &current);
                let hash = hex::encode(hasher.finalize());
                core::info(&format!("Step {} - sha1: {}", i + 1, hash));
                hash.into_bytes()
            }
            "sha256" => {
                let mut hasher = Sha256::new();
                Digest::update(&mut hasher, &current);
                let hash = hex::encode(hasher.finalize());
                core::info(&format!("Step {} - sha256: {}", i + 1, hash));
                hash.into_bytes()
            }
            "hmac-sha256" => {
                let key = arg.unwrap_or("");
                use hmac::{Hmac, Mac};
                type HmacSha256 = Hmac<sha2::Sha256>;
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
                mac.update(&current);
                let hash = hex::encode(mac.finalize().into_bytes());
                core::info(&format!("Step {} - hmac-sha256: {}", i + 1, hash));
                hash.into_bytes()
            }
            "base64-encode" => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&current);
                core::info(&format!("Step {} - base64: {}", i + 1, encoded));
                encoded.into_bytes()
            }
            "base64-decode" => {
                use base64::Engine;
                let decoded = base64::engine::general_purpose::STANDARD.decode(&current)?;
                core::info(&format!("Step {} - base64-decode: {}", i + 1, String::from_utf8_lossy(&decoded)));
                decoded
            }
            "url-encode" => {
                let input_str = String::from_utf8_lossy(&current);
                let s = urlencoding::encode(&input_str);
                core::info(&format!("Step {} - url-encode: {}", i + 1, s));
                s.as_bytes().to_vec()
            }
            "url-decode" => {
                let input_str = String::from_utf8_lossy(&current);
                let s = urlencoding::decode(&input_str)?;
                core::info(&format!("Step {} - url-decode: {}", i + 1, s));
                s.into_owned().into_bytes()
            }
            "hex-encode" => {
                let s = hex::encode(&current);
                core::info(&format!("Step {} - hex: {}", i + 1, s));
                s.into_bytes()
            }
            "hex-decode" => {
                let s = hex::decode(String::from_utf8_lossy(&current).as_ref())?;
                core::info(&format!("Step {} - hex-decode: {}", i + 1, String::from_utf8_lossy(&s)));
                s
            }
            _ => return Err(anyhow!("未知步骤: {}", op)),
        };

        current = result;
    }

    let final_output = match output_format {
        "base64" => {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(&current)
        }
        _ => String::from_utf8_lossy(&current).to_string(),
    };

    println!("\n{}", "最终结果:".green().bold());
    println!("{}", final_output);
    Ok(())
}

pub fn url_encode(input: &str) -> Result<()> {
    println!("{}", urlencoding::encode(input));
    Ok(())
}

pub fn url_decode(input: &str) -> Result<()> {
    println!("{}", urlencoding::decode(input)?);
    Ok(())
}

pub fn base64_encode(input: &str) -> Result<()> {
    use base64::Engine;
    println!("{}", base64::engine::general_purpose::STANDARD.encode(input));
    Ok(())
}

pub fn base64_decode(input: &str) -> Result<()> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(input)?;
    println!("{}", String::from_utf8_lossy(&decoded));
    Ok(())
}

pub fn unicode_escape(input: &str) -> Result<()> {
    let s: String = input.chars().map(|c| {
        if c.is_ascii() {
            c.to_string()
        } else {
            format!("\\u{:04x}", c as u32)
        }
    }).collect();
    println!("{}", s);
    Ok(())
}

pub fn unicode_unescape(input: &str) -> Result<()> {
    let mut result = String::new();
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some('u') = chars.next() {
                let hex: String = chars.by_ref().take(4).collect();
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(ch) = char::from_u32(code) {
                        result.push(ch);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }
    println!("{}", result);
    Ok(())
}

pub fn hex_encode(input: &str) -> Result<()> {
    println!("{}", hex::encode(input));
    Ok(())
}

pub fn hex_decode(input: &str) -> Result<()> {
    let decoded = hex::decode(input)?;
    println!("{}", String::from_utf8_lossy(&decoded));
    Ok(())
}

pub fn timestamp(fixed: Option<i64>, offset: i64, bits: &str, format: &str) -> Result<()> {
    let ts = match fixed {
        Some(v) => v,
        None => {
            use chrono::Utc;
            let now = Utc::now().timestamp_millis();
            now / if bits == "10" { 1000 } else { 1 }
        }
    };

    let adjusted = ts + offset * if bits == "10" { 1000 } else { 1 };

    match bits {
        "10" => println!("{}", adjusted / 1000),
        "formatted" => {
            use chrono::{TimeZone, Utc};
            let dt = Utc.timestamp_millis_opt(adjusted).single().unwrap();
            println!("{}", dt.format(format));
        }
        _ => println!("{}", adjusted),
    }
    Ok(())
}

pub fn random(length: usize, charset: &str, seed: Option<u64>, count: usize) -> Result<()> {
    use rand::Rng;

    let chars: Vec<char> = match charset {
        "alpha" => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
        "numeric" => "0123456789".chars().collect(),
        "hex" => "0123456789abcdef".chars().collect(),
        _ => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect(),
    };

    for _ in 0..count {
        if let Some(s) = seed {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(s);
            let s: String = (0..length).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
            println!("{}", s);
        } else {
            let mut rng = rand::thread_rng();
            let s: String = (0..length).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
            println!("{}", s);
        }
    }
    Ok(())
}

pub fn sign_sort(params: &str, salt: Option<&str>, algorithm: &str, separator: &str, joiner: &str, urlencode: bool) -> Result<()> {
    use md5::Digest;

    // 支持多种输入格式: JSON / key=value,key=value / key:value;key:value
    let map: HashMap<String, String> = if let Ok(m) = serde_json::from_str::<HashMap<String, String>>(params) {
        m
    } else if params.contains('=') {
        // key=value,key=value 格式
        params.split(',')
            .filter_map(|p| {
                let (k, v) = p.split_once('=')?;
                Some((k.trim().to_string(), v.trim().to_string()))
            })
            .collect()
    } else if params.contains(';') {
        // key:value;key:value 格式
        params.split(';')
            .filter_map(|p| {
                let (k, v) = p.split_once(':')?;
                Some((k.trim().to_string(), v.trim().to_string()))
            })
            .collect()
    } else {
        return Err(anyhow!("无法解析参数，请使用 JSON {{\"a\":\"1\"}} 或 key=value 格式"));
    };
    let mut sorted: Vec<_> = map.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let joined: String = sorted.iter().map(|(k, v)| {
        let val = if urlencode { urlencoding::encode(v).to_string() } else { v.clone() };
        format!("{}{}{}", k, separator, val)
    }).collect::<Vec<_>>().join(joiner);

    let with_salt = match salt {
        Some(s) => format!("{}{}", joined, s),
        None => joined.clone(),
    };

    println!("拼接结果: {}", with_salt);

    let hash = match algorithm {
        "md5" => {
            let mut hasher = <md5::Md5 as Digest>::new();
            Digest::update(&mut hasher, with_salt.as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha1" => {
            let mut hasher = sha1::Sha1::new();
            Digest::update(&mut hasher, with_salt.as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha256" => {
            let mut hasher = sha2::Sha256::new();
            Digest::update(&mut hasher, with_salt.as_bytes());
            hex::encode(hasher.finalize())
        }
        _ => return Err(anyhow!("不支持的算法: {}", algorithm)),
    };

    println!("{} 签名: {}", algorithm, hash);
    Ok(())
}

pub fn diff_sign(src_str: &str, src_sign: &str, dst_str: &str, dst_sign: &str) -> Result<()> {
    println!("源签名原文: {}", src_str);
    println!("源签名值:   {}", src_sign);
    println!("目标签名原文: {}", dst_str);
    println!("目标签名值:   {}", dst_sign);
    println!();

    if src_sign == dst_sign {
        core::success("签名值相同");
    } else {
        core::warn("签名值不同");
        let max_len = src_sign.len().max(dst_sign.len());
        for i in 0..max_len {
            let sc = src_sign.get(i..i+1).unwrap_or("");
            let dc = dst_sign.get(i..i+1).unwrap_or("");
            if sc != dc {
                println!("  位置 {}: '{}' vs '{}' ← 差异", i, sc, dc);
            }
        }
    }

    let src_map: HashMap<&str, &str> = src_str.split('&').filter_map(|p| p.split_once('=')).collect();
    let dst_map: HashMap<&str, &str> = dst_str.split('&').filter_map(|p| p.split_once('=')).collect();

    let mut all_keys: Vec<&str> = src_map.keys().chain(dst_map.keys()).copied().collect();
    all_keys.sort();
    all_keys.dedup();

    println!("\n参数差异:");
    for key in all_keys {
        let sv = src_map.get(key);
        let dv = dst_map.get(key);
        match (sv, dv) {
            (Some(a), Some(b)) if a != b => println!("  {} : '{}' vs '{}' ← 值不同", key, a, b),
            (Some(_), None) => println!("  {} : 存在 vs 缺失 ← 源有目标无", key),
            (None, Some(_)) => println!("  {} : 缺失 vs 存在 ← 源无目标有", key),
            _ => {}
        }
    }

    Ok(())
}

pub fn hex_view(file: Option<&Path>, offset: usize, length: usize, aes_align: bool) -> Result<()> {
    let data = match file {
        Some(path) => fs::read(path)?,
        None => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            buf
        }
    };

    let start = offset;
    let end = if length == 0 { data.len() } else { (offset + length).min(data.len()) };
    let data = &data[start..end];

    println!("长度: {} 字节", data.len());
    if aes_align {
        println!("AES 分组对齐 (16 字节): {}", if data.len() % 16 == 0 { "对齐" } else { "未对齐" });
    }
    println!();

    for (i, chunk) in data.chunks(16).enumerate() {
        let addr = start + i * 16;
        let hex_str: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
        let ascii: String = chunk.iter().map(|&b| if b >= 32 && b < 127 { b as char } else { '.' }).collect();
        if aes_align && chunk.len() == 16 {
            println!("─── AES block ───");
        }
        println!("{:08x}  {:<48}  {}", addr, hex_str, ascii);
    }
    Ok(())
}

pub fn cookie_builder(cookies: &[String]) -> Result<()> {
    let cookie_str: String = cookies.join("; ");
    println!("Cookie: {}", cookie_str);
    Ok(())
}

pub fn js_test(_file: &Path, _function: &str, _args: &str, engine: &str) -> Result<()> {
    core::info(&format!("使用 {} 引擎执行 JS 片段", engine));
    core::warn("QuickJS 引擎暂未集成, 请使用 chrome/firefox 引擎");
    Ok(())
}

pub fn token_replace(file: &Path, old: &str, new: &str, output: Option<&Path>) -> Result<()> {
    let content = fs::read_to_string(file)?;
    let replaced = content.replace(old, new);

    let count = content.matches(old).count();
    core::info(&format!("替换了 {} 处 token", count));

    match output {
        Some(path) => {
            fs::write(path, &replaced)?;
            core::success(&format!("已保存到 {}", path.display()));
        }
        None => {
            fs::write(file, &replaced)?;
            core::success("已覆盖原文件");
        }
    }
    Ok(())
}
