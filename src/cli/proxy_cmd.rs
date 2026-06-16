use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::core::Mode;

#[derive(Args)]
pub struct ProxyArgs {
    #[command(subcommand)]
    pub action: ProxyAction,
}

#[derive(Subcommand)]
pub enum ProxyAction {
    /// 启动 MITM 代理
    Start {
        /// 监听端口
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// URL 过滤 (正则或通配符, 如 "*.example.com")
        #[arg(short, long)]
        filter: Option<String>,

        /// 过滤模式: whitelist / blacklist
        #[arg(long, default_value = "whitelist")]
        filter_mode: String,

        /// 回调 URL (请求/响应拦截后发送到此 URL)
        #[arg(short, long)]
        callback: Option<String>,

        /// 回调超时 (秒)
        #[arg(long, default_value = "5")]
        callback_timeout: u64,

        /// 回调最大重试次数
        #[arg(long, default_value = "3")]
        callback_retries: u32,

        /// 自动注入 hook fetch/axios
        #[arg(long)]
        hook_fetch: bool,

        /// 自动注入 hook 指定函数名
        #[arg(long)]
        hook_function: Option<String>,

        /// 屏蔽静态资源 (css/img/js)
        #[arg(long)]
        no_static: bool,
    },

    /// 停止代理
    Stop,

    /// 导出 CA 证书
    Cert {
        /// 导出路径
        #[arg(short, long, default_value = "./ca.cer")]
        output: PathBuf,
    },

    /// 添加自动篡改规则
    Rule {
        #[command(subcommand)]
        action: RuleAction,
    },

    /// 查看抓包会话
    Sessions {
        /// URL 过滤
        #[arg(short, long)]
        filter: Option<String>,

        /// 最大显示数量
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum RuleAction {
    /// 添加规则
    Add {
        /// 规则表达式, 如 "Header:User-Agent=xxx" 或 "Replace:old=new"
        rule: String,
    },
    /// 列出所有规则
    List,
    /// 删除规则
    Remove {
        /// 规则索引
        index: usize,
    },
    /// 清空所有规则
    Clear,
}

pub fn execute(args: &ProxyArgs, mode: &Mode) -> anyhow::Result<()> {
    match &args.action {
        ProxyAction::Start { port, filter, filter_mode, callback, callback_timeout, callback_retries, hook_fetch, hook_function, no_static } => {
            crate::proxy::start(crate::proxy::ProxyConfig {
                port: *port,
                filter: filter.clone(),
                filter_mode: filter_mode.clone(),
                callback_url: callback.clone(),
                callback_timeout: *callback_timeout,
                callback_retries: *callback_retries,
                hook_fetch: *hook_fetch,
                hook_function: hook_function.clone(),
                no_static: *no_static,
                mode: mode.clone(),
            })
        }
        ProxyAction::Stop => crate::proxy::stop(),
        ProxyAction::Cert { output } => crate::proxy::export_cert(output),
        ProxyAction::Rule { action } => match action {
            RuleAction::Add { rule } => crate::proxy::add_rule(rule),
            RuleAction::List => crate::proxy::list_rules(),
            RuleAction::Remove { index } => crate::proxy::remove_rule(*index),
            RuleAction::Clear => crate::proxy::clear_rules(),
        },
        ProxyAction::Sessions { filter, limit } => crate::proxy::show_sessions(filter.as_deref(), *limit),
    }
}
