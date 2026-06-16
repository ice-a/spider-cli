use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct MiniArgs {
    #[command(subcommand)]
    pub action: MiniAction,
}

#[derive(Subcommand)]
pub enum MiniAction {
    /// 解包微信小程序 wxapkg
    Unpack {
        /// wxapkg 文件路径
        file: PathBuf,

        /// 输出目录
        #[arg(short, long, default_value = "./wxapkg_output")]
        output: PathBuf,

        /// 提取接口和加密函数
        #[arg(long)]
        extract_apis: bool,
    },

    /// APK 简易解析
    Apk {
        /// APK 文件路径
        file: PathBuf,

        /// 提取网络请求 URL
        #[arg(long)]
        extract_urls: bool,

        /// 提取静态密钥
        #[arg(long)]
        extract_keys: bool,
    },
}

pub fn execute(args: &MiniArgs) -> anyhow::Result<()> {
    match &args.action {
        MiniAction::Unpack { file, output, extract_apis } => {
            crate::app_reverse::unpack_wxapkg(file, output, *extract_apis)
        }
        MiniAction::Apk { file, extract_urls, extract_keys } => {
            crate::app_reverse::parse_apk(file, *extract_urls, *extract_keys)
        }
    }
}
