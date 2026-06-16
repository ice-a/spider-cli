use clap::{Parser, Subcommand};

use crate::core::Mode;

#[derive(Parser)]
#[command(name = "reptool", version, about = "Rust 全栈逆向/爬虫/代理 CLI 工具链")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 执行模式: manual(手工), semi(半自动), auto(全自动)
    #[arg(long, global = true, default_value = "auto")]
    pub mode: Mode,
}

#[derive(Subcommand)]
pub enum Commands {
    /// MITM 中间人代理抓包
    Proxy(proxy_cmd::ProxyArgs),

    /// HAR 文件解析/对比/重放/导出
    Har(har_cmd::HarArgs),

    /// JS/TS 逆向: 格式化/反混淆/接口提取/Hook生成
    Js(js_cmd::JsArgs),

    /// 手工逆向计算: 分步加密/签名排序/参数编辑
    Calc(calc_cmd::CalcArgs),

    /// 加密工具箱: hash/对称/非对称/编码/时间/随机
    Crypto(crypto_cmd::CryptoArgs),

    /// 多协议爬虫: HTTP/WS 批量请求
    Crawl(crawl_cmd::CrawlArgs),

    /// 多格式代码导出
    Export(export_cmd::ExportArgs),

    /// 单发手工请求
    Req(req_cmd::ReqArgs),

    /// 小程序/APK/protobuf 逆向
    Mini(mini_cmd::MiniArgs),

    /// protobuf 解码
    Proto(proto_cmd::ProtoArgs),

    /// 辅助工具: DNS/端口/测速/json/编码探测
    Tools(tools_cmd::ToolsArgs),

    /// 双内核无头渲染: Chrome/Firefox
    Render(render_cmd::RenderArgs),

    /// 配置持久化/版本更新/自动补全
    Config(config_cmd::ConfigArgs),

    /// MCP Server (stdio, 给 Claude/LLM)
    Mcp,

    /// AI 辅助逆向分析
    Ai(ai_cmd::AiArgs),

    /// Frida Hook 生成/附加
    Hook(hook_cmd::HookArgs),

    /// AI 配置 (多厂商支持)
    AiConfig(ai_config_cmd::AiConfigArgs),
}

impl Cli {
    pub fn execute(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Proxy(args) => proxy_cmd::execute(args, &self.mode),
            Commands::Har(args) => har_cmd::execute(args, &self.mode),
            Commands::Js(args) => js_cmd::execute(args, &self.mode),
            Commands::Calc(args) => calc_cmd::execute(args, &self.mode),
            Commands::Crypto(args) => crypto_cmd::execute(args),
            Commands::Crawl(args) => crawl_cmd::execute(args, &self.mode),
            Commands::Export(args) => export_cmd::execute(args),
            Commands::Req(args) => req_cmd::execute(args),
            Commands::Mini(args) => mini_cmd::execute(args),
            Commands::Proto(args) => proto_cmd::execute(args),
            Commands::Tools(args) => tools_cmd::execute(args),
            Commands::Render(args) => render_cmd::execute(args, &self.mode),
            Commands::Config(args) => config_cmd::execute(args),
            Commands::Mcp => crate::mcp::run(),
            Commands::Ai(args) => ai_cmd::execute(args),
            Commands::Hook(args) => hook_cmd::execute(args),
            Commands::AiConfig(args) => ai_config_cmd::execute(args),
        }
    }
}

pub mod proxy_cmd;
pub mod har_cmd;
pub mod js_cmd;
pub mod calc_cmd;
pub mod crypto_cmd;
pub mod crawl_cmd;
pub mod export_cmd;
pub mod req_cmd;
pub mod mini_cmd;
pub mod proto_cmd;
pub mod tools_cmd;
pub mod render_cmd;
pub mod config_cmd;
pub mod ai_cmd;
pub mod hook_cmd;
pub mod ai_config_cmd;
