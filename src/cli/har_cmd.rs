use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::core::Mode;

#[derive(Args)]
pub struct HarArgs {
    #[command(subcommand)]
    pub action: HarAction,
}

#[derive(Subcommand)]
pub enum HarAction {
    /// 解析 HAR 文件
    Parse {
        /// HAR 文件路径
        file: PathBuf,

        /// 提取内容: cookies,headers,params,urls,apis
        #[arg(short, long, value_delimiter = ',')]
        extract: Option<Vec<String>>,

        /// 仅显示 POST/上传/登录/支付接口
        #[arg(long)]
        post_only: bool,

        /// 正则过滤 URL
        #[arg(long)]
        regex: Option<String>,

        /// 输出 JSON 格式
        #[arg(long)]
        json: bool,
    },

    /// 对比两份 HAR 文件差异
    Diff {
        /// 旧 HAR 文件
        old: PathBuf,

        /// 新 HAR 文件
        new: PathBuf,

        /// 高亮 sign/token/encrypt 字段
        #[arg(long, default_value = "true")]
        highlight_sign: bool,
    },

    /// 自动识别登录凭证
    ExtractCreds {
        /// HAR 文件路径
        file: PathBuf,

        /// 导出会话 JSON 文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 批量重放请求
    Replay {
        /// HAR 文件路径
        file: PathBuf,

        /// 并发数
        #[arg(short, long, default_value = "5")]
        concurrency: usize,

        /// 请求间隔 (毫秒)
        #[arg(long)]
        delay: Option<u64>,

        /// 代理地址
        #[arg(long)]
        proxy: Option<String>,

        /// 替换 token (格式: old_token=new_token)
        #[arg(long)]
        replace_token: Option<String>,

        /// 输出结果文件
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 增强过滤
    Filter {
        /// HAR 文件路径
        file: PathBuf,

        /// HTTP 方法过滤
        #[arg(long)]
        method: Option<String>,

        /// URL 正则
        #[arg(long)]
        regex: Option<String>,

        /// 正文正则
        #[arg(long)]
        body_regex: Option<String>,

        /// 响应内容正则
        #[arg(long)]
        response_regex: Option<String>,
    },

    /// 导出为其他格式
    Export {
        /// HAR 文件路径
        file: PathBuf,

        /// 导出格式
        #[arg(short, long, default_value = "curl")]
        format: ExportFormat,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// 导出指定索引 (默认全部)
        #[arg(short, long)]
        index: Option<usize>,

        /// 导出全部 Cookie 合并为会话 JSON
        #[arg(long)]
        session: bool,
    },

    /// 还原 multipart 表单
    Multipart {
        /// HAR 文件路径
        file: PathBuf,

        /// 指定请求索引
        #[arg(short, long, default_value = "0")]
        index: usize,

        /// 输出目录
        #[arg(short, long, default_value = "./multipart")]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    Curl,
    Python,
    Java,
    Go,
    Fetch,
    Postman,
    Insomnia,
    Jmeter,
}

pub fn execute(args: &HarArgs, mode: &Mode) -> anyhow::Result<()> {
    match &args.action {
        HarAction::Parse { file, extract, post_only, regex, json } => {
            crate::har::parse(file, extract.as_deref(), *post_only, regex.as_deref(), *json, mode)
        }
        HarAction::Diff { old, new, highlight_sign } => {
            crate::har::diff(old, new, *highlight_sign)
        }
        HarAction::ExtractCreds { file, output } => {
            crate::har::extract_creds(file, output.as_deref())
        }
        HarAction::Replay { file, concurrency, delay, proxy, replace_token, output } => {
            crate::har::replay(file, *concurrency, delay.as_ref(), proxy.as_deref(), replace_token.as_deref(), output.as_deref(), mode)
        }
        HarAction::Filter { file, method, regex, body_regex, response_regex } => {
            crate::har::filter(file, method.as_deref(), regex.as_deref(), body_regex.as_deref(), response_regex.as_deref())
        }
        HarAction::Export { file, format, output, index, session } => {
            crate::har::export(file, format, output.as_deref(), *index, *session)
        }
        HarAction::Multipart { file, index, output } => {
            crate::har::multipart(file, *index, output)
        }
    }
}
