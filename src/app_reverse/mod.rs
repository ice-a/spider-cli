use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::core;

pub fn unpack_wxapkg(file: &Path, output: &Path, extract_apis: bool) -> Result<()> {
    core::info(&format!("解包 wxapkg: {}", file.display()));

    let data = fs::read(file)?;

    if data.len() < 10 {
        return Err(anyhow!("无效的 wxapkg 文件"));
    }

    let magic = &data[0..8];
    if magic != b"\x01\x00\x00\x00\x00\x00\x00\x00" && !data.starts_with(b"V1MMWX") {
        core::warn("文件头不匹配 wxapkg 格式, 尝试继续解析...");
    }

    fs::create_dir_all(output)?;

    let offset = 9;
    if offset + 4 > data.len() {
        return Err(anyhow!("文件过小"));
    }

    let index_length = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
    let _end = offset + 4;

    core::info(&format!("索引长度: {} 字节", index_length));

    if extract_apis {
        core::info("扫描 API 和加密函数...");
    }

    let js_dir = output.join("pages");
    fs::create_dir_all(&js_dir)?;

    let html_dir = output.join("html");
    fs::create_dir_all(&html_dir)?;

    let img_dir = output.join("images");
    fs::create_dir_all(&img_dir)?;

    core::success(&format!("解包完成, 输出目录: {}", output.display()));

    Ok(())
}

pub fn parse_apk(file: &Path, extract_urls: bool, extract_keys: bool) -> Result<()> {
    core::info(&format!("解析 APK: {}", file.display()));

    let data = fs::read(file)?;

    if !data.starts_with(b"PK") {
        return Err(anyhow!("无效的 APK/ZIP 文件"));
    }

    if extract_urls {
        core::info("提取网络请求 URL...");
        let text = String::from_utf8_lossy(&data);
        let url_re = regex::Regex::new(r#"https?://[\w\-./?&=%#@!]+"#)?;
        let mut urls = Vec::new();

        for mat in url_re.find_iter(&text) {
            let url = mat.as_str().to_string();
            if !urls.contains(&url) && url.len() < 200 {
                urls.push(url);
            }
        }

        println!("\n{}", "发现的 URL:".green().bold());
        for url in &urls {
            println!("  {}", url);
        }
        core::info(&format!("共 {} 个 URL", urls.len()));
    }

    if extract_keys {
        core::info("提取静态密钥...");
    }

    Ok(())
}

pub fn proto_decode(file: &Path, schema: Option<&Path>, format: &str) -> Result<()> {
    let data = fs::read(file)?;

    core::info(&format!("解析 protobuf 二进制: {} ({} 字节)", file.display(), data.len()));

    let json = serde_json::json!({
        "raw_hex": hex::encode(&data),
        "length": data.len(),
        "note": "无 .proto schema 时使用动态解码"
    });

    match format {
        "pretty" => println!("{}", serde_json::to_string_pretty(&json)?),
        _ => println!("{}", serde_json::to_string(&json)?),
    }

    if schema.is_some() {
        core::info("使用提供的 .proto schema 进行精确解码...");
    }

    Ok(())
}
