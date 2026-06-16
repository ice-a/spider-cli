use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core;

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".reptool")
        .join("config.toml")
}

fn secrets_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".reptool")
        .join("secrets.json")
}

pub fn set(key: &str, value: &str) -> Result<()> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap())?;

    let mut config: HashMap<String, String> = if path.exists() {
        let content = fs::read_to_string(&path)?;
        toml::from_str(&content)?
    } else {
        HashMap::new()
    };

    config.insert(key.to_string(), value.to_string());
    let content = toml::to_string_pretty(&config)?;
    fs::write(&path, content)?;

    core::success(&format!("已设置 {} = {}", key, value));
    Ok(())
}

pub fn get(key: &str) -> Result<()> {
    let path = config_path();
    if !path.exists() {
        return Err(anyhow!("配置文件不存在"));
    }

    let content = fs::read_to_string(&path)?;
    let config: HashMap<String, String> = toml::from_str(&content)?;

    match config.get(key) {
        Some(value) => println!("{} = {}", key, value),
        None => return Err(anyhow!("配置项 '{}' 不存在", key)),
    }

    Ok(())
}

pub fn list() -> Result<()> {
    let path = config_path();
    if !path.exists() {
        core::info("无配置");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;
    let config: HashMap<String, String> = toml::from_str(&content)?;

    for (k, v) in &config {
        println!("{} = {}", k, v);
    }

    Ok(())
}

pub fn update() -> Result<()> {
    core::info("检查更新...");
    core::info("当前版本: 0.1.0");
    core::info("请访问 https://github.com/your/reptool 查看最新版本");
    Ok(())
}

pub fn completion(shell: &str) -> Result<()> {
    match shell {
        "bash" => {
            println!(r#"# Reptool bash completion
_reptool_completions() {{
    local cur prev commands
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    commands="proxy har js calc crypto crawl export req mini proto tools render config mcp"

    if [[ ${{cur}} == -* ]]; then
        COMPREPLY=( $(compgen -W "--help --version --mode" -- ${{cur}}) )
    elif [[ ${{prev}} == proxy ]]; then
        COMPREPLY=( $(compgen -W "start stop cert rule sessions" -- ${{cur}}) )
    elif [[ ${{prev}} == har ]]; then
        COMPREPLY=( $(compgen -W "parse diff extract-creds replay filter export multipart" -- ${{cur}}) )
    elif [[ ${{prev}} == js ]]; then
        COMPREPLY=( $(compgen -W "format deobfuscate scan-api analyze-sign extract-keys hook-generate batch-run run-snippet" -- ${{cur}}) )
    elif [[ ${{prev}} == crypto ]]; then
        COMPREPLY=( $(compgen -W "hash hmac encrypt decrypt rsa-encrypt rsa-decrypt rsa-sign rsa-verify random-ua random-str" -- ${{cur}}) )
    else
        COMPREPLY=( $(compgen -W "${{commands}}" -- ${{cur}}) )
    fi
    return 0
}}
complete -F _reptool_completions reptool"#);
        }
        "zsh" => {
            println!(r#"#compdef reptool
_reptool() {{
    _arguments \
        '1:command:(proxy har js calc crypto crawl export req mini proto tools render config mcp)' \
        '*::arg:->args'
}}
_reptool "$@""#);
        }
        "fish" => {
            println!(r#"complete -c reptool -f
complete -c reptool -n "__fish_use_subcommand" -a proxy -d "MITM proxy"
complete -c reptool -n "__fish_use_subcommand" -a har -d "HAR parse"
complete -c reptool -n "__fish_use_subcommand" -a js -d "JS reverse"
complete -c reptool -n "__fish_use_subcommand" -a calc -d "Calculator"
complete -c reptool -n "__fish_use_subcommand" -a crypto -d "Crypto tools"
complete -c reptool -n "__fish_use_subcommand" -a crawl -d "Crawler"
complete -c reptool -n "__fish_use_subcommand" -a export -d "Export code"
complete -c reptool -n "__fish_use_subcommand" -a mcp -d "MCP Server""#);
        }
        _ => return Err(anyhow!("不支持的 shell: {} (支持 bash/zsh/fish)", shell)),
    }

    Ok(())
}

pub fn save_secret(name: &str, value: &str) -> Result<()> {
    let path = secrets_path();
    fs::create_dir_all(path.parent().unwrap())?;

    let mut secrets: HashMap<String, String> = if path.exists() {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content)?
    } else {
        HashMap::new()
    };

    secrets.insert(name.to_string(), value.to_string());
    fs::write(&path, serde_json::to_string_pretty(&secrets)?)?;

    core::success(&format!("已保存密钥: {}", name));
    Ok(())
}

pub fn list_secrets() -> Result<()> {
    let path = secrets_path();
    if !path.exists() {
        core::info("无已保存的密钥");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;
    let secrets: HashMap<String, String> = serde_json::from_str(&content)?;

    for (k, v) in &secrets {
        println!("{}: {}", k, v);
    }

    Ok(())
}

pub fn delete_secret(name: &str) -> Result<()> {
    let path = secrets_path();
    if !path.exists() {
        return Err(anyhow!("无已保存的密钥"));
    }

    let content = fs::read_to_string(&path)?;
    let mut secrets: HashMap<String, String> = serde_json::from_str(&content)?;

    if secrets.remove(name).is_some() {
        fs::write(&path, serde_json::to_string_pretty(&secrets)?)?;
        core::success(&format!("已删除密钥: {}", name));
    } else {
        return Err(anyhow!("密钥 '{}' 不存在", name));
    }

    Ok(())
}
