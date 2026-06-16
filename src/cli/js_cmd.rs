use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::core::Mode;

#[derive(Args)]
pub struct JsArgs {
    #[command(subcommand)]
    pub action: JsAction,
}

#[derive(Subcommand)]
pub enum JsAction {
    /// JS 格式化/美化
    Format {
        /// JS 文件路径
        file: PathBuf,

        /// 输出文件路径 (默认覆盖原文件)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// 压缩模式 (默认美化)
        #[arg(long)]
        minify: bool,
    },

    /// JS 反混淆
    Deobfuscate {
        /// JS 文件路径
        file: PathBuf,

        /// 反混淆技术: auto, decrypt, control_flow, eval, dead_code
        #[arg(short, long, default_value = "auto")]
        technique: DeobfTechnique,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 扫描 JS 提取接口
    ScanApi {
        /// JS 文件或目录路径
        path: PathBuf,

        /// 输出 JSON 格式
        #[arg(long)]
        json: bool,
    },

    /// 追踪加密函数
    AnalyzeSign {
        /// JS 文件路径
        file: PathBuf,

        /// 追踪的函数名 (逗号分隔)
        #[arg(short, long)]
        functions: Option<String>,

        /// 输出 JSON
        #[arg(long)]
        json: bool,
    },

    /// 提取常量 (key/iv/appid/secret/盐值)
    ExtractKeys {
        /// JS 文件路径
        file: PathBuf,

        /// 输出 JSON
        #[arg(long)]
        json: bool,
    },

    /// 生成 JS Hook 脚本
    HookGenerate {
        /// 要 hook 的函数名
        function_name: String,

        /// Hook 类型: auto, console, network
        #[arg(short, long, default_value = "auto")]
        hook_type: String,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 批量扫描 JS 文件夹
    BatchScan {
        /// JS 文件夹路径
        dir: PathBuf,

        /// 输出 JSON
        #[arg(long)]
        json: bool,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// JS 代码片段隔离运行
    RunSnippet {
        /// JS 文件路径
        file: PathBuf,

        /// 运行后端: quickjs, chrome, firefox
        #[arg(short, long, default_value = "quickjs")]
        engine: String,

        /// 传入参数 (JSON 数组)
        #[arg(short, long)]
        args: Option<String>,

        /// 浏览器模式时的目标 URL
        #[arg(long)]
        url: Option<String>,

        /// 调试模式 (弹出浏览器窗口)
        #[arg(long)]
        debug: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DeobfTechnique {
    Auto,
    Decrypt,
    ControlFlow,
    Eval,
    DeadCode,
}

pub fn execute(args: &JsArgs, mode: &Mode) -> anyhow::Result<()> {
    match &args.action {
        JsAction::Format { file, output, minify } => {
            crate::js::format(file, output.as_deref(), *minify)
        }
        JsAction::Deobfuscate { file, technique, output } => {
            crate::js::deobfuscate(file, technique, output.as_deref(), mode)
        }
        JsAction::ScanApi { path, json } => {
            crate::js::scan_api(path, *json, mode)
        }
        JsAction::AnalyzeSign { file, functions, json } => {
            crate::js::analyze_sign(file, functions.as_deref(), *json, mode)
        }
        JsAction::ExtractKeys { file, json } => {
            crate::js::extract_keys(file, *json, mode)
        }
        JsAction::HookGenerate { function_name, hook_type, output } => {
            crate::js::hook_generate(function_name, hook_type, output.as_deref())
        }
        JsAction::BatchScan { dir, json, output } => {
            crate::js::batch_scan(dir, *json, output.as_deref(), mode)
        }
        JsAction::RunSnippet { file, engine, args, url, debug } => {
            crate::js::run_snippet(file, engine, args.as_deref(), url.as_deref(), *debug, mode)
        }
    }
}
