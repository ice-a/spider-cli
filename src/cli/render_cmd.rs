use clap::Args;
use std::path::PathBuf;

use crate::core::Mode;

#[derive(Args)]
pub struct RenderArgs {
    /// 浏览器引擎: chrome, firefox
    #[arg(short, long, default_value = "chrome")]
    engine: String,

    /// 目标 URL
    #[arg(short, long)]
    url: Option<String>,

    /// 本地 HTML 文件
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// 注入的 JS 脚本文件
    #[arg(short, long)]
    script: Option<PathBuf>,

    /// 调试模式 (弹出浏览器窗口)
    #[arg(long)]
    debug: bool,

    /// 录制 HAR
    #[arg(long)]
    capture_har: bool,

    /// HAR 输出路径
    #[arg(long)]
    har_output: Option<PathBuf>,

    /// 自定义 UA
    #[arg(long)]
    ua: Option<String>,

    /// 自定义 Cookie (name=value, 多次使用)
    #[arg(long)]
    cookie: Vec<String>,

    /// 导出页面全局变量
    #[arg(long)]
    dump_vars: bool,

    /// 替换线上 JS (本地文件路径)
    #[arg(long)]
    js_replace: Option<String>,

    /// 超时 (秒)
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Canvas 指纹篡改
    #[arg(long)]
    spoof_canvas: bool,

    /// WebGL 参数伪造
    #[arg(long)]
    spoof_webgl: bool,
}

pub fn execute(args: &RenderArgs, mode: &Mode) -> anyhow::Result<()> {
    crate::render::run(
        &args.engine,
        args.url.as_deref(),
        args.file.as_deref(),
        args.script.as_deref(),
        args.debug,
        args.capture_har,
        args.har_output.as_deref(),
        args.ua.as_deref(),
        &args.cookie,
        args.dump_vars,
        args.js_replace.as_deref(),
        args.timeout,
        args.spoof_canvas,
        args.spoof_webgl,
        mode,
    )
}
