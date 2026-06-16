use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct CryptoArgs {
    #[command(subcommand)]
    pub action: CryptoAction,
}

#[derive(Subcommand)]
pub enum CryptoAction {
    /// 哈希计算
    Hash {
        /// 算法: md5, md5-salt, sha1, sha256, sha512
        #[arg(short, long)]
        algo: String,

        /// 输入数据
        #[arg(short, long)]
        input: String,

        /// 盐值 (md5-salt 模式)
        #[arg(long)]
        salt: Option<String>,

        /// 输出格式: hex, base64
        #[arg(long, default_value = "hex")]
        output: String,
    },

    /// HMAC 计算
    Hmac {
        /// 算法: md5, sha1, sha256, sha512
        #[arg(short, long)]
        algo: String,

        /// 密钥
        #[arg(short, long)]
        key: String,

        /// 输入数据
        #[arg(short, long)]
        data: String,

        /// 输出格式: hex, base64
        #[arg(long, default_value = "hex")]
        output: String,
    },

    /// 对称加密
    Encrypt {
        /// 算法: aes-cbc, aes-ecb, aes-gcm, des, 3des, rc4
        #[arg(short, long)]
        algo: String,

        /// 密钥
        #[arg(short, long)]
        key: String,

        /// IV 向量 (CBC/GCM)
        #[arg(long)]
        iv: Option<String>,

        /// 输入数据
        #[arg(short, long)]
        data: String,

        /// 输入格式: text, hex, base64
        #[arg(long, default_value = "text")]
        input_format: String,

        /// 输出格式: hex, base64
        #[arg(long, default_value = "hex")]
        output_format: String,
    },

    /// 对称解密
    Decrypt {
        /// 算法: aes-cbc, aes-ecb, aes-gcm, des, 3des, rc4
        #[arg(short, long)]
        algo: String,

        /// 密钥
        #[arg(short, long)]
        key: String,

        /// IV 向量 (CBC/GCM)
        #[arg(long)]
        iv: Option<String>,

        /// 输入数据
        #[arg(short, long)]
        data: String,

        /// 输入格式: hex, base64
        #[arg(long, default_value = "hex")]
        input_format: String,

        /// 输出格式: text, hex, base64
        #[arg(long, default_value = "text")]
        output_format: String,
    },

    /// RSA 加密
    RsaEncrypt {
        /// 公钥文件路径 (PEM)
        #[arg(short, long)]
        key: PathBuf,

        /// 输入数据
        #[arg(short, long)]
        data: String,
    },

    /// RSA 解密
    RsaDecrypt {
        /// 私钥文件路径 (PEM)
        #[arg(short, long)]
        key: PathBuf,

        /// 输入数据 (hex/base64)
        #[arg(short, long)]
        data: String,
    },

    /// RSA 签名
    RsaSign {
        /// 私钥文件路径 (PEM)
        #[arg(short, long)]
        key: PathBuf,

        /// 输入数据
        #[arg(short, long)]
        data: String,

        /// 算法: sha256
        #[arg(long, default_value = "sha256")]
        algorithm: String,
    },

    /// RSA 验签
    RsaVerify {
        /// 公钥文件路径 (PEM)
        #[arg(short, long)]
        key: PathBuf,

        /// 输入数据
        #[arg(short, long)]
        data: String,

        /// 签名值 (hex/base64)
        #[arg(short, long)]
        signature: String,

        /// 算法: sha256
        #[arg(long, default_value = "sha256")]
        algorithm: String,
    },

    /// 随机 User-Agent
    RandomUa {
        /// 浏览器类型: chrome, firefox, safari, edge, all
        #[arg(short, long, default_value = "all")]
        browser: String,

        /// 生成数量
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// 随机字符串
    RandomStr {
        /// 长度
        #[arg(short, long, default_value = "16")]
        length: usize,

        /// 数量
        #[arg(short, long, default_value = "1")]
        count: usize,
    },
}

pub fn execute(args: &CryptoArgs) -> anyhow::Result<()> {
    match &args.action {
        CryptoAction::Hash { algo, input, salt, output } => {
            crate::crypto::hash(algo, input, salt.as_deref(), output)
        }
        CryptoAction::Hmac { algo, key, data, output } => {
            crate::crypto::hmac_sign(algo, key, data, output)
        }
        CryptoAction::Encrypt { algo, key, iv, data, input_format, output_format } => {
            crate::crypto::encrypt(algo, key, iv.as_deref(), data, input_format, output_format)
        }
        CryptoAction::Decrypt { algo, key, iv, data, input_format, output_format } => {
            crate::crypto::decrypt(algo, key, iv.as_deref(), data, input_format, output_format)
        }
        CryptoAction::RsaEncrypt { key, data } => crate::crypto::rsa_encrypt(key, data),
        CryptoAction::RsaDecrypt { key, data } => crate::crypto::rsa_decrypt(key, data),
        CryptoAction::RsaSign { key, data, algorithm } => crate::crypto::rsa_sign(key, data, algorithm),
        CryptoAction::RsaVerify { key, data, signature, algorithm } => {
            crate::crypto::rsa_verify(key, data, signature, algorithm)
        }
        CryptoAction::RandomUa { browser, count } => crate::crypto::random_ua(browser, *count),
        CryptoAction::RandomStr { length, count } => crate::crypto::random_str(*length, *count),
    }
}
