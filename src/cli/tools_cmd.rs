use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ToolsArgs {
    #[command(subcommand)]
    pub action: ToolsAction,
}

#[derive(Subcommand)]
pub enum ToolsAction {
    /// DNS 解析
    Dns {
        /// 域名
        domain: String,

        /// 自定义 DNS 服务器
        #[arg(long)]
        server: Option<String>,
    },

    /// 自定义 hosts 映射
    Hosts {
        /// 域名
        domain: String,

        /// IP 地址
        ip: String,

        /// 移除映射
        #[arg(long)]
        remove: bool,
    },

    /// 端口占用检测
    PortCheck {
        /// 端口号
        port: u16,
    },

    /// 网络测速
    Speed {
        /// 目标 URL
        url: String,

        /// 测试次数
        #[arg(short, long, default_value = "5")]
        count: usize,
    },

    /// JSON 格式化
    JsonFormat {
        /// 输入文件 (或 stdin)
        file: Option<PathBuf>,
    },

    /// JSON 压缩
    JsonCompact {
        /// 输入文件 (或 stdin)
        file: Option<PathBuf>,
    },

    /// JSON 路径提取
    JsonExtract {
        /// 输入文件
        file: PathBuf,

        /// JSONPath 表达式 (如 "$.data.list")
        path: String,
    },

    /// JSON 合并
    JsonMerge {
        /// 基础 JSON 文件
        base: PathBuf,

        /// 合并 JSON 文件
        overlay: PathBuf,

        /// 输出文件
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// CSV 转 JSON
    CsvToJson {
        /// CSV 文件路径
        file: PathBuf,

        /// 输出文件
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 编码探测
    EncodingDetect {
        /// 文件路径
        file: PathBuf,
    },
}

pub fn execute(args: &ToolsArgs) -> anyhow::Result<()> {
    match &args.action {
        ToolsAction::Dns { domain, server } => crate::tools::dns(domain, server.as_deref()),
        ToolsAction::Hosts { domain, ip, remove } => crate::tools::hosts(domain, ip, *remove),
        ToolsAction::PortCheck { port } => crate::tools::port_check(*port),
        ToolsAction::Speed { url, count } => crate::tools::speed(url, *count),
        ToolsAction::JsonFormat { file } => crate::tools::json_format(file.as_deref()),
        ToolsAction::JsonCompact { file } => crate::tools::json_compact(file.as_deref()),
        ToolsAction::JsonExtract { file, path } => crate::tools::json_extract(file, path),
        ToolsAction::JsonMerge { base, overlay, output } => {
            crate::tools::json_merge(base, overlay, output.as_deref())
        }
        ToolsAction::CsvToJson { file, output } => {
            crate::tools::csv_to_json(file, output.as_deref())
        }
        ToolsAction::EncodingDetect { file } => crate::tools::encoding_detect(file),
    }
}
