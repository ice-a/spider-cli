use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::core::Mode;

#[derive(Args)]
pub struct CalcArgs {
    #[command(subcommand)]
    pub action: CalcAction,
}

#[derive(Subcommand)]
pub enum CalcAction {
    /// 参数字典手动编辑器
    Params {
        /// 交互式手动输入模式
        #[arg(long)]
        manual: bool,

        /// 预设参数 (key=value, 多次使用)
        #[arg(short, long)]
        param: Vec<String>,

        /// 排序方式: asc, desc, custom
        #[arg(long, default_value = "asc")]
        sort: String,

        /// 自定义排序顺序 (逗号分隔的 key)
        #[arg(long)]
        order: Option<String>,

        /// 过滤空值
        #[arg(long)]
        skip_empty: bool,

        /// 输出格式: kv, url, json, form
        #[arg(long, default_value = "kv")]
        output_format: String,
    },

    /// 分步加密计算器
    Step {
        /// 步骤链 (格式: "sort-params:{json}", "concat-salt:{salt}", "md5", "sha256", "hmac-sha256:{key}", "aes-cbc-encrypt:{key}:{iv}", "base64-encode", "url-encode")
        #[arg(short, long, num_args = 1..)]
        step: Vec<String>,

        /// 输入数据 (JSON)
        #[arg(short, long)]
        data: Option<String>,

        /// 输出格式: hex, base64
        #[arg(long, default_value = "hex")]
        output_format: String,
    },

    /// 独立编码工具 (不链式)
    UrlEncode {
        /// 输入文本
        input: String,
    },

    UrlDecode {
        /// 输入文本
        input: String,
    },

    Base64Encode {
        /// 输入文本
        input: String,
    },

    Base64Decode {
        /// 输入文本
        input: String,
    },

    UnicodeEscape {
        /// 输入文本
        input: String,
    },

    UnicodeUnescape {
        /// 输入文本
        input: String,
    },

    HexEncode {
        /// 输入文本
        input: String,
    },

    HexDecode {
        /// 输入 hex 字符串
        input: String,
    },

    /// 手动时间戳生成
    Timestamp {
        /// 固定时间戳 (秒或毫秒)
        #[arg(long)]
        fixed: Option<i64>,

        /// 偏移秒数 (±)
        #[arg(long, default_value = "0")]
        offset: i64,

        /// 输出格式: 10, 13, formatted
        #[arg(long, default_value = "13")]
        bits: String,

        /// 日期格式 (formatted 模式)
        #[arg(long, default_value = "%Y-%m-%d %H:%M:%S")]
        format: String,
    },

    /// 随机数/随机串生成
    Random {
        /// 长度
        #[arg(short, long, default_value = "16")]
        length: usize,

        /// 字符集: alphanumeric, alpha, numeric, hex
        #[arg(long, default_value = "alphanumeric")]
        charset: String,

        /// 固定种子 (用于复现)
        #[arg(long)]
        seed: Option<u64>,

        /// 生成数量
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// 参数字典排序一键生成 (升序拼接+盐值+md5)
    SignSort {
        /// 参数 (JSON 或 key=value 格式)
        params: String,

        /// 盐值/密钥
        #[arg(short, long)]
        salt: Option<String>,

        /// 哈希算法: md5, sha1, sha256
        #[arg(long, default_value = "md5")]
        algorithm: String,

        /// 分隔符
        #[arg(long, default_value = "=")]
        separator: String,

        /// 连接符
        #[arg(long, default_value = "&")]
        joiner: String,

        /// 是否 URL encode 值
        #[arg(long)]
        urlencode: bool,
    },

    /// 签名差异对比
    DiffSign {
        /// 源签名原文
        #[arg(long)]
        src_str: String,

        /// 源签名值
        #[arg(long)]
        src_sign: String,

        /// 目标签名原文
        #[arg(long)]
        dst_str: String,

        /// 目标签名值
        #[arg(long)]
        dst_sign: String,
    },

    /// 十六进制/二进制查看器
    HexView {
        /// 文件路径 (或 stdin)
        file: Option<PathBuf>,

        /// 起始偏移
        #[arg(long, default_value = "0")]
        offset: usize,

        /// 长度 (0=全部)
        #[arg(long, default_value = "0")]
        length: usize,

        /// AES 16位分组对齐
        #[arg(long)]
        aes_align: bool,
    },

    /// 手动 Cookie/Header 组装
    CookieBuilder {
        /// Cookie 值 (name=value, 多次使用)
        #[arg(short, long, num_args = 1..)]
        cookie: Vec<String>,
    },

    /// JS 加密函数本地批量测试
    JsTest {
        /// JS 文件路径
        file: PathBuf,

        /// 函数名
        #[arg(short, long)]
        function: String,

        /// 测试参数 (JSON 数组, 如 '["a","b","c"]')
        #[arg(short, long)]
        args: String,

        /// 运行后端: quickjs, chrome, firefox
        #[arg(long, default_value = "quickjs")]
        engine: String,
    },

    /// 批量替换 token 重放 HAR
    TokenReplace {
        /// HAR 文件路径
        file: PathBuf,

        /// 旧 token
        #[arg(long)]
        old: String,

        /// 新 token
        #[arg(long)]
        new: String,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// 自定义盐值/密钥临时存储
    ConfigSave {
        /// 名称
        name: String,

        /// 值
        value: String,
    },

    /// 查看已保存的密钥
    ConfigList,

    /// 删除已保存的密钥
    ConfigDelete {
        /// 名称
        name: String,
    },
}

pub fn execute(args: &CalcArgs, _mode: &Mode) -> anyhow::Result<()> {
    match &args.action {
        CalcAction::Params { manual, param, sort, order, skip_empty, output_format } => {
            crate::calc::params_editor(*manual, param, sort, order.as_deref(), *skip_empty, output_format)
        }
        CalcAction::Step { step, data, output_format } => {
            crate::calc::step_calc(step, data.as_deref(), output_format)
        }
        CalcAction::UrlEncode { input } => crate::calc::url_encode(input),
        CalcAction::UrlDecode { input } => crate::calc::url_decode(input),
        CalcAction::Base64Encode { input } => crate::calc::base64_encode(input),
        CalcAction::Base64Decode { input } => crate::calc::base64_decode(input),
        CalcAction::UnicodeEscape { input } => crate::calc::unicode_escape(input),
        CalcAction::UnicodeUnescape { input } => crate::calc::unicode_unescape(input),
        CalcAction::HexEncode { input } => crate::calc::hex_encode(input),
        CalcAction::HexDecode { input } => crate::calc::hex_decode(input),
        CalcAction::Timestamp { fixed, offset, bits, format } => {
            crate::calc::timestamp(*fixed, *offset, bits, format)
        }
        CalcAction::Random { length, charset, seed, count } => {
            crate::calc::random(*length, charset, *seed, *count)
        }
        CalcAction::SignSort { params, salt, algorithm, separator, joiner, urlencode } => {
            crate::calc::sign_sort(params, salt.as_deref(), algorithm, separator, joiner, *urlencode)
        }
        CalcAction::DiffSign { src_str, src_sign, dst_str, dst_sign } => {
            crate::calc::diff_sign(src_str, src_sign, dst_str, dst_sign)
        }
        CalcAction::HexView { file, offset, length, aes_align } => {
            crate::calc::hex_view(file.as_deref(), *offset, *length, *aes_align)
        }
        CalcAction::CookieBuilder { cookie } => {
            crate::calc::cookie_builder(cookie)
        }
        CalcAction::JsTest { file, function, args, engine } => {
            crate::calc::js_test(file, function, args, engine)
        }
        CalcAction::TokenReplace { file, old, new, output } => {
            crate::calc::token_replace(file, old, new, output.as_deref())
        }
        CalcAction::ConfigSave { name, value } => {
            crate::config::save_secret(name, value)
        }
        CalcAction::ConfigList => crate::config::list_secrets(),
        CalcAction::ConfigDelete { name } => {
            crate::config::delete_secret(name)
        }
    }
}
