use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct HookArgs {
    #[command(subcommand)]
    pub action: HookAction,
}

#[derive(Subcommand)]
pub enum HookAction {
    /// 生成 Frida Hook 脚本
    Generate {
        /// Hook 目标 (类名/函数名/库名)
        target: String,

        /// Hook 类型: ssl, crypto, http, websocket, storage, native, class, general
        #[arg(short = 't', long, default_value = "general")]
        hook_type: String,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 附加到进程 (显示 frida 命令)
    Attach {
        /// 进程名或 PID
        process: String,

        /// 脚本文件路径
        script: String,
    },

    /// 列出设备进程
    Ps,
}

pub fn execute(args: &HookArgs) -> anyhow::Result<()> {
    match &args.action {
        HookAction::Generate { target, hook_type, output } => {
            crate::frida::generate_hook(target, hook_type, output.as_deref())
        }
        HookAction::Attach { process, script } => {
            crate::frida::attach(process, script)
        }
        HookAction::Ps => crate::frida::list_processes(),
    }
}
