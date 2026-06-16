use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::core::Mode;

#[derive(Args)]
pub struct CrawlArgs {
    #[command(subcommand)]
    pub action: CrawlAction,
}

#[derive(Subcommand)]
pub enum CrawlAction {
    /// 执行批量任务
    Run {
        /// 任务文件路径 (TOML)
        file: PathBuf,

        /// 代理列表文件
        #[arg(long)]
        proxy: Option<PathBuf>,

        /// 并发数
        #[arg(short, long, default_value = "10")]
        concurrency: usize,

        /// 请求间隔 (毫秒)
        #[arg(long)]
        delay: Option<u64>,

        /// 最大重试次数
        #[arg(long, default_value = "3")]
        retries: u32,
    },

    /// 单次 HTTP 请求
    Http {
        /// URL
        url: String,

        /// HTTP 方法
        #[arg(short, long, default_value = "GET")]
        method: String,

        /// 请求头 (JSON)
        #[arg(short, long)]
        headers: Option<String>,

        /// 请求体
        #[arg(short, long)]
        body: Option<String>,

        /// 代理
        #[arg(long)]
        proxy: Option<String>,

        /// Cookie
        #[arg(long)]
        cookie: Option<String>,

        /// 输出响应体
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// WebSocket 连接
    Ws {
        /// WebSocket URL
        url: String,

        /// 发送消息
        #[arg(short, long)]
        message: Option<String>,

        /// 持续监听
        #[arg(long)]
        listen: bool,
    },

    /// 保存会话
    SessionSave {
        /// 输出文件路径
        output: PathBuf,

        /// Cookie (name=value, 多次使用)
        #[arg(short, long)]
        cookie: Vec<String>,

        /// Token / Authorization
        #[arg(long)]
        auth: Option<String>,

        /// 额外 Header (JSON)
        #[arg(long)]
        headers: Option<String>,
    },

    /// 加载会话并请求
    SessionLoad {
        /// 会话文件路径
        file: PathBuf,

        /// 请求 URL
        #[arg(short, long)]
        url: Option<String>,
    },
}

pub fn execute(args: &CrawlArgs, mode: &Mode) -> anyhow::Result<()> {
    match &args.action {
        CrawlAction::Run { file, proxy, concurrency, delay, retries } => {
            crate::crawl::run(file, proxy.as_deref(), *concurrency, delay.as_ref(), *retries, mode)
        }
        CrawlAction::Http { url, method, headers, body, proxy, cookie, output } => {
            crate::crawl::http_single(url, method, headers.as_deref(), body.as_deref(), proxy.as_deref(), cookie.as_deref(), output.as_deref())
        }
        CrawlAction::Ws { url, message, listen } => {
            crate::crawl::ws_connect(url, message.as_deref(), *listen)
        }
        CrawlAction::SessionSave { output, cookie, auth, headers } => {
            crate::crawl::session_save(output, cookie, auth.as_deref(), headers.as_deref())
        }
        CrawlAction::SessionLoad { file, url } => {
            crate::crawl::session_load(file, url.as_deref())
        }
    }
}
