use clap::{Args, Subcommand};

#[derive(Args)]
pub struct AiConfigArgs {
    #[command(subcommand)]
    pub action: AiConfigAction,
}

#[derive(Subcommand)]
pub enum AiConfigAction {
    /// 交互式配置 AI 厂商/API Key/模型
    Setup,

    /// 显示当前配置
    Show,

    /// 快速配置 (命令行参数)
    Set {
        /// 厂商名称 (Claude/GPT/Gemini/小米/DeepSeek/通义/豆包/自定义)
        #[arg(short, long)]
        provider: Option<String>,

        /// API Base URL
        #[arg(short, long)]
        base_url: Option<String>,

        /// API Key
        #[arg(short, long)]
        api_key: Option<String>,

        /// 模型名称
        #[arg(short, long)]
        model: Option<String>,
    },

    /// AI 辅助逆向分析
    Reverse {
        /// JS 文件路径或代码
        file: String,
    },

    /// AI 对话
    Chat {
        /// 用户消息
        message: Vec<String>,
    },
}

pub fn execute(args: &AiConfigArgs) -> anyhow::Result<()> {
    match &args.action {
        AiConfigAction::Setup => crate::ai_config::setup(),
        AiConfigAction::Show => {
            crate::ai_config::show_config();
            Ok(())
        }
        AiConfigAction::Set { provider, base_url, api_key, model } => {
            let mut config = crate::ai_config::load_config().unwrap_or(crate::ai_config::AiConfig {
                provider: "自定义 (OpenAI 兼容)".into(),
                base_url: "https://api.openai.com".into(),
                api_key: String::new(),
                model: "gpt-4".into(),
            });
            if let Some(p) = provider { config.provider = p.clone(); }
            if let Some(u) = base_url { config.base_url = u.clone(); }
            if let Some(k) = api_key { config.api_key = k.clone(); }
            if let Some(m) = model { config.model = m.clone(); }
            let path = dirs::home_dir().unwrap_or_default().join(".reptool").join("ai_config.json");
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(&path, serde_json::to_string_pretty(&config)?)?;
            crate::core::success("AI 配置已更新");
            crate::ai_config::show_config();
            Ok(())
        }
        AiConfigAction::Reverse { file } => crate::ai_config::reverse_with_ai(file),
        AiConfigAction::Chat { message } => {
            let msg = message.join(" ");
            crate::ai_config::chat(&msg)
        }
    }
}
