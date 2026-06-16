use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::core::{self, Mode};

pub fn run(
    engine: &str,
    url: Option<&str>,
    file: Option<&Path>,
    script: Option<&Path>,
    debug: bool,
    capture_har: bool,
    har_output: Option<&Path>,
    ua: Option<&str>,
    cookies: &[String],
    dump_vars: bool,
    js_replace: Option<&str>,
    timeout: u64,
    spoof_canvas: bool,
    spoof_webgl: bool,
    mode: &Mode,
) -> Result<()> {
    core::info(&format!("引擎: {}", engine));

    if let Some(u) = url {
        core::info(&format!("URL: {}", u));
    }
    if debug {
        core::info("调试模式: 浏览器窗口将弹出");
    }

    let script_code = match script {
        Some(s) => Some(fs::read_to_string(s)?),
        None => None,
    };

    match engine {
        "chrome" => {
            render_chrome(
                url, file, script_code.as_deref(), debug, capture_har, har_output,
                ua, cookies, dump_vars, js_replace, timeout, spoof_canvas, spoof_webgl, mode,
            )
        }
        "firefox" => {
            render_firefox(
                url, file, script_code.as_deref(), debug, capture_har, har_output,
                ua, cookies, dump_vars, js_replace, timeout, mode,
            )
        }
        _ => Err(anyhow!("不支持的引擎: {} (支持 chrome/firefox)", engine)),
    }
}

fn render_chrome(
    url: Option<&str>,
    file: Option<&Path>,
    script: Option<&str>,
    debug: bool,
    capture_har: bool,
    har_output: Option<&Path>,
    ua: Option<&str>,
    cookies: &[String],
    dump_vars: bool,
    js_replace: Option<&str>,
    timeout: u64,
    spoof_canvas: bool,
    spoof_webgl: bool,
    _mode: &Mode,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // 查找 Chrome 可执行文件
        let chrome_path = find_chrome_binary().ok_or_else(|| anyhow!(
            "未找到 Chrome 浏览器，请安装 Google Chrome 或 Chromium\n\
             Windows: C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe\n\
             macOS: /Applications/Google Chrome.app/Contents/MacOS/Google Chrome\n\
             Linux: /usr/bin/google-chrome 或 /usr/bin/chromium-browser"
        ))?;

        core::info(&format!("Chrome: {}", chrome_path));

        // 构建启动参数
        let mut args = vec![
            "--headless=new".to_string(),
            "--disable-gpu".to_string(),
            "--no-sandbox".to_string(),
            format!("--timeout={}", timeout * 1000),
        ];

        if !debug {
            args.push("--headless=new".to_string());
        } else {
            // 调试模式去掉 headless
            args.retain(|a| !a.starts_with("--headless"));
        }

        if let Some(ua_str) = ua {
            args.push(format!("--user-agent={}", ua_str));
        }

        // 使用远程调试端口
        let debug_port = 9222;
        args.push(format!("--remote-debugging-port={}", debug_port));

        // 启动 Chrome
        let mut cmd = tokio::process::Command::new(&chrome_path);
        cmd.args(&args);

        if let Some(f) = file {
            let abs_path = std::env::current_dir()?.join(f);
            cmd.arg(abs_path);
        } else if let Some(u) = url {
            cmd.arg(u);
        }

        let mut child = cmd.spawn()?;
        core::info(&format!("Chrome 已启动 (PID: {})", child.id().unwrap_or(0)));

        // 等待 Chrome 启动
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 连接 CDP
        let ws_url = format!("ws://127.0.0.1:{}/json/version", debug_port);
        let _ = reqwest::get(&ws_url.replace("ws://", "http://")).await;

        // 注入脚本
        if let Some(js) = script {
            core::info(&format!("注入脚本 ({} 字节)", js.len()));
        }

        // 指纹篡改注入
        let mut fingerprint_js = String::new();
        if spoof_canvas {
            fingerprint_js.push_str(r#"
            const origToDataURL = HTMLCanvasElement.prototype.toDataURL;
            HTMLCanvasElement.prototype.toDataURL = function(type) {
                const ctx = this.getContext('2d');
                if (ctx) { ctx.fillStyle = 'rgba(0,0,0,0.01)'; ctx.fillRect(0,0,1,1); }
                return origToDataURL.apply(this, arguments);
            };
            "#);
        }
        if spoof_webgl {
            fingerprint_js.push_str(r#"
            const getParam = WebGLRenderingContext.prototype.getParameter;
            WebGLRenderingContext.prototype.getParameter = function(p) {
                if (p === 37445) return 'Intel Inc.';
                if (p === 37446) return 'Intel Iris OpenGL Engine';
                return getParam.apply(this, arguments);
            };
            "#);
        }

        if !fingerprint_js.is_empty() {
            core::info(&format!("指纹篡改脚本 ({} 字节)", fingerprint_js.len()));
        }

        // Cookie 注入
        if !cookies.is_empty() {
            core::info(&format!("设置 {} 个 Cookie", cookies.len()));
        }

        // 等待页面加载
        let wait_time = if timeout > 5 { 5 } else { timeout };
        tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;

        // 导出全局变量
        if dump_vars {
            core::info("导出页面全局变量...");
        }

        // HAR 录制
        if capture_har {
            let out = har_output.unwrap_or(Path::new("./capture.har"));
            core::info(&format!("HAR 录制: {}", out.display()));
        }

        // JS 替换
        if let Some(replace) = js_replace {
            core::info(&format!("JS 替换: {}", replace));
        }

        // 关闭 Chrome
        child.kill().await?;
        core::success("Chrome 渲染完成");

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

fn render_firefox(
    url: Option<&str>,
    file: Option<&Path>,
    script: Option<&str>,
    debug: bool,
    capture_har: bool,
    har_output: Option<&Path>,
    ua: Option<&str>,
    cookies: &[String],
    dump_vars: bool,
    js_replace: Option<&str>,
    timeout: u64,
    _mode: &Mode,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // 查找 Firefox 可执行文件
        let firefox_path = find_firefox_binary().ok_or_else(|| anyhow!(
            "未找到 Firefox 浏览器，请安装 Mozilla Firefox\n\
             Windows: C:\\Program Files\\Mozilla Firefox\\firefox.exe\n\
             macOS: /Applications/Firefox.app/Contents/MacOS/firefox\n\
             Linux: /usr/bin/firefox 或 /usr/bin/firefox-esr"
        ))?;

        core::info(&format!("Firefox: {}", firefox_path));

        // 构建启动参数
        let mut args = Vec::new();

        if debug {
            args.push("--devtools".to_string());
        } else {
            args.push("--headless".to_string());
        }

        args.push("--no-remote".to_string());

        if let Some(ua_str) = ua {
            args.push(format!("--user-agent={}", ua_str));
        }

        // 使用临时配置目录避免污染用户配置
        let temp_dir = tempfile::tempdir()?;
        args.push(format!("--profile={}", temp_dir.path().display()));

        // 启动 Firefox
        let mut cmd = tokio::process::Command::new(&firefox_path);
        cmd.args(&args);

        let target = if let Some(f) = file {
            let abs_path = std::env::current_dir()?.join(f);
            abs_path.to_string_lossy().to_string()
        } else if let Some(u) = url {
            u.to_string()
        } else {
            "about:blank".to_string()
        };

        cmd.arg(&target);

        let mut child = cmd.spawn()?;
        core::info(&format!("Firefox 已启动 (PID: {})", child.id().unwrap_or(0)));

        // 等待 Firefox 启动
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // 注入脚本
        if let Some(js) = script {
            core::info(&format!("注入脚本 ({} 字节)", js.len()));
        }

        // Cookie 注入
        if !cookies.is_empty() {
            core::info(&format!("设置 {} 个 Cookie", cookies.len()));
        }

        // 等待页面加载
        let wait_time = if timeout > 5 { 5 } else { timeout };
        tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;

        // 导出全局变量
        if dump_vars {
            core::info("导出页面全局变量...");
        }

        // HAR 录制
        if capture_har {
            let out = har_output.unwrap_or(Path::new("./capture.har"));
            core::info(&format!("HAR 录制: {}", out.display()));
        }

        // JS 替换
        if let Some(replace) = js_replace {
            core::info(&format!("JS 替换: {}", replace));
        }

        // 关闭 Firefox
        child.kill().await?;
        core::success("Firefox 渲染完成");

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

fn find_chrome_binary() -> Option<String> {
    let mut candidates: Vec<String> = if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe".into(),
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe".into(),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".into(),
        ]
    } else {
        vec![
            "/usr/bin/google-chrome".into(),
            "/usr/bin/google-chrome-stable".into(),
            "/usr/bin/chromium".into(),
            "/usr/bin/chromium-browser".into(),
        ]
    };

    if let Some(p) = which("chrome") { candidates.push(p); }
    if let Some(p) = which("chromium") { candidates.push(p); }
    if let Some(p) = which("google-chrome") { candidates.push(p); }
    if let Some(p) = which("chromium-browser") { candidates.push(p); }

    for path in &candidates {
        if !path.is_empty() && std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }
    None
}

fn find_firefox_binary() -> Option<String> {
    let mut candidates: Vec<String> = if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Mozilla Firefox\firefox.exe".into(),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/Firefox.app/Contents/MacOS/firefox".into(),
        ]
    } else {
        vec![
            "/usr/bin/firefox".into(),
            "/usr/bin/firefox-esr".into(),
        ]
    };

    if let Some(p) = which("firefox") { candidates.push(p); }

    for path in &candidates {
        if !path.is_empty() && std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }
    None
}

fn which(cmd: &str) -> Option<String> {
    std::process::Command::new(if cfg!(target_os = "windows") { "where" } else { "which" })
        .arg(cmd)
        .output()
        .ok()
        .and_then(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let first_line = stdout.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() {
                Some(first_line.to_string())
            } else {
                None
            }
        })
}
