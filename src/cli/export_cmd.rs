use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(Args)]
pub struct ExportArgs {
    /// HAR 文件路径
    file: PathBuf,

    /// 导出格式
    #[arg(short, long, default_value = "curl")]
    format: ExportFormat,

    /// 输出文件路径
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 导出指定索引
    #[arg(short, long)]
    index: Option<usize>,

    /// 生成完整爬虫模板
    #[arg(long)]
    template: bool,

    /// 模板语言: rust, python
    #[arg(long, default_value = "python")]
    lang: String,
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

pub fn execute(args: &ExportArgs) -> anyhow::Result<()> {
    let fmt = match &args.format {
        ExportFormat::Curl => crate::export::ExportFormat::Curl,
        ExportFormat::Python => crate::export::ExportFormat::Python,
        ExportFormat::Java => crate::export::ExportFormat::Java,
        ExportFormat::Go => crate::export::ExportFormat::Go,
        ExportFormat::Fetch => crate::export::ExportFormat::Fetch,
        ExportFormat::Postman => crate::export::ExportFormat::Postman,
        ExportFormat::Insomnia => crate::export::ExportFormat::Insomnia,
        ExportFormat::Jmeter => crate::export::ExportFormat::Jmeter,
    };
    crate::export::run(&args.file, &fmt, args.output.as_deref(), args.index, args.template, &args.lang)
}
