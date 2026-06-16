use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ProtoArgs {
    #[command(subcommand)]
    pub action: ProtoAction,
}

#[derive(Subcommand)]
pub enum ProtoAction {
    /// 解码 protobuf 二进制为 JSON
    Decode {
        /// 二进制文件路径
        file: PathBuf,

        /// .proto 定义文件路径 (可选)
        #[arg(short, long)]
        schema: Option<PathBuf>,

        /// 输出格式: json, pretty
        #[arg(long, default_value = "pretty")]
        format: String,
    },
}

pub fn execute(args: &ProtoArgs) -> anyhow::Result<()> {
    match &args.action {
        ProtoAction::Decode { file, schema, format } => {
            crate::app_reverse::proto_decode(file, schema.as_deref(), format)
        }
    }
}
