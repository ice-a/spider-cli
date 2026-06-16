use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ReqArgs {
    #[command(subcommand)]
    pub action: ReqAction,
}

#[derive(Subcommand)]
pub enum ReqAction {
    /// 单发手工请求 (无并发/无自动重试/无会话)
    Single {
        /// HTTP 方法
        #[arg(short = 'X', long, default_value = "GET")]
        method: String,

        /// URL
        #[arg(short = 'u', long)]
        url: String,

        /// 请求头 (name=value, 多次使用)
        #[arg(short = 'H', long, num_args = 0..)]
        header: Vec<String>,

        /// 请求体
        #[arg(short = 'b', long)]
        body: Option<String>,

        /// Content-Type
        #[arg(short = 'c', long)]
        content_type: Option<String>,

        /// Cookie
        #[arg(long)]
        cookie: Option<String>,

        /// 代理
        #[arg(long)]
        proxy: Option<String>,

        /// 超时 (秒)
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// 输出完整请求/响应报文
        #[arg(long)]
        verbose: bool,

        /// 输出到文件
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

pub fn execute(args: &ReqArgs) -> anyhow::Result<()> {
    match &args.action {
        ReqAction::Single { method, url, header, body, content_type, cookie, proxy, timeout, verbose, output } => {
            crate::crawl::req_single(method, url, header, body.as_deref(), content_type.as_deref(), cookie.as_deref(), proxy.as_deref(), *timeout, *verbose, output.as_deref())
        }
    }
}
