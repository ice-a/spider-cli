use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct AiArgs {
    #[command(subcommand)]
    pub action: AiAction,
}

#[derive(Subcommand)]
pub enum AiAction {
    /// AI 分析 JS 文件 (混淆检测/加密函数/密钥/API提取/逆向建议)
    AnalyzeJs {
        /// JS 文件路径
        file: PathBuf,
    },

    /// AI 分析 HAR 流量 (请求模式/可疑接口/参数分析)
    AnalyzeTraffic {
        /// HAR 文件路径
        file: PathBuf,
    },

    /// AI 生成逆向报告 (JS + HAR 综合分析)
    Report {
        /// JS 文件或目录 (可多个)
        #[arg(short, long)]
        js: Vec<PathBuf>,

        /// HAR 文件 (可多个)
        #[arg(short, long)]
        har: Vec<PathBuf>,
    },
}

pub fn execute(args: &AiArgs) -> anyhow::Result<()> {
    match &args.action {
        AiAction::AnalyzeJs { file } => crate::ai::analyze_js(file),
        AiAction::AnalyzeTraffic { file } => crate::ai::analyze_traffic(file),
        AiAction::Report { js, har } => crate::ai::generate_report(js, har),
    }
}
