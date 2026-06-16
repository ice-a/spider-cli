use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// 设置配置项
    Set {
        /// 配置键 (如 global-ua, proxy, export-dir)
        key: String,

        /// 配置值
        value: String,
    },

    /// 获取配置项
    Get {
        /// 配置键
        key: String,
    },

    /// 列出所有配置
    List,

    /// 检查并更新到最新版本
    Update,

    /// 生成 shell 自动补全脚本
    Completion {
        /// Shell 类型: bash, zsh, fish
        shell: String,
    },
}

pub fn execute(args: &ConfigArgs) -> anyhow::Result<()> {
    match &args.action {
        ConfigAction::Set { key, value } => crate::config::set(key, value),
        ConfigAction::Get { key } => crate::config::get(key),
        ConfigAction::List => crate::config::list(),
        ConfigAction::Update => crate::config::update(),
        ConfigAction::Completion { shell } => crate::config::completion(shell),
    }
}
