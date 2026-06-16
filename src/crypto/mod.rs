use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use rand::Rng;

use crate::core;

pub fn hash(algo: &str, input: &str, salt: Option<&str>, output: &str) -> Result<()> {
    use md5::Digest;
    use sha1::Sha1;
    use sha2::{Sha256, Sha512};

    let data = match salt {
        Some(s) => format!("{}{}", input, s),
        None => input.to_string(),
    };

    let result = match algo {
        "md5" => {
            let mut hasher = <md5::Md5 as Digest>::new();
            Digest::update(&mut hasher, data.as_bytes());
            hex::encode(hasher.finalize())
        }
        "md5-salt" => {
            let s = salt.ok_or_else(|| anyhow!("md5-salt 需要 --salt 参数"))?;
            let mut hasher = <md5::Md5 as Digest>::new();
            Digest::update(&mut hasher, format!("{}{}", s, input).as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha1" => {
            let mut hasher = Sha1::new();
            Digest::update(&mut hasher, data.as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha256" => {
            let mut hasher = Sha256::new();
            Digest::update(&mut hasher, data.as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            Digest::update(&mut hasher, data.as_bytes());
            hex::encode(hasher.finalize())
        }
        _ => return Err(anyhow!("不支持的哈希算法: {}", algo)),
    };

    let display = match output {
        "base64" => general_purpose::STANDARD.encode(hex::decode(&result)?),
        _ => result,
    };

    println!("{}", display);
    Ok(())
}

pub fn hmac_sign(algo: &str, key: &str, data: &str, output: &str) -> Result<()> {
    use hmac::{Hmac, Mac};

    type HmacMd5 = Hmac<md5::Md5>;
    type HmacSha1 = Hmac<sha1::Sha1>;
    type HmacSha256 = Hmac<sha2::Sha256>;
    type HmacSha512 = Hmac<sha2::Sha512>;

    let result = match algo {
        "md5" => {
            let mut mac = HmacMd5::new_from_slice(key.as_bytes())?;
            mac.update(data.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        }
        "sha1" => {
            let mut mac = HmacSha1::new_from_slice(key.as_bytes())?;
            mac.update(data.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        }
        "sha256" => {
            let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
            mac.update(data.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        }
        "sha512" => {
            let mut mac = HmacSha512::new_from_slice(key.as_bytes())?;
            mac.update(data.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        }
        _ => return Err(anyhow!("不支持的 HMAC 算法: {}", algo)),
    };

    let display = match output {
        "base64" => general_purpose::STANDARD.encode(hex::decode(&result)?),
        _ => result,
    };

    println!("{}", display);
    Ok(())
}

fn parse_input(data: &str, format: &str) -> Result<Vec<u8>> {
    match format {
        "hex" => Ok(hex::decode(data)?),
        "base64" => Ok(general_purpose::STANDARD.decode(data)?),
        "text" => Ok(data.as_bytes().to_vec()),
        _ => Err(anyhow!("不支持的格式: {}", format)),
    }
}

fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let pad_len = block_size - (data.len() % block_size);
    let mut buf = data.to_vec();
    buf.extend(std::iter::repeat(pad_len as u8).take(pad_len));
    buf
}

fn pkcs7_unpad(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow!("数据为空"));
    }
    let pad_len = *data.last().unwrap() as usize;
    if pad_len == 0 || pad_len > 16 || pad_len > data.len() {
        return Err(anyhow!("无效�?PKCS7 填充"));
    }
    for &b in &data[data.len() - pad_len..] {
        if b as usize != pad_len {
            return Err(anyhow!("无效�?PKCS7 填充"));
        }
    }
    Ok(data[..data.len() - pad_len].to_vec())
}

pub fn encrypt(algo: &str, key: &str, iv: Option<&str>, data: &str, input_fmt: &str, output_fmt: &str) -> Result<()> {
    let key_bytes = parse_input(key, "text")?;
    let data_bytes = parse_input(data, input_fmt)?;

    let result = match algo {
        "aes-cbc" => {
            use aes::Aes128;
            use cbc::cipher::{BlockEncryptMut, KeyIvInit, generic_array::GenericArray};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 16]);
            let mut cipher = cbc::Encryptor::<Aes128>::new(key_bytes[..16].into(), iv_bytes[..16].into());
            let padded = pkcs7_pad(&data_bytes, 16);
            let mut out = padded.clone();
            for chunk in out.chunks_mut(16) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.encrypt_block_mut(block);
            }
            out
        }
        "aes-ecb" => {
            use aes::Aes128;
            use ecb::cipher::{BlockEncryptMut, generic_array::GenericArray};
            use aes::cipher::KeyInit;
            let mut cipher = ecb::Encryptor::<Aes128>::new_from_slice(&key_bytes[..16])?;
            let padded = pkcs7_pad(&data_bytes, 16);
            let mut out = padded.clone();
            for chunk in out.chunks_mut(16) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.encrypt_block_mut(block);
            }
            out
        }
        "aes-gcm" => {
            use aes_gcm::{aead::Aead, Aes128Gcm, KeyInit, Nonce};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 12]);
            let cipher = Aes128Gcm::new_from_slice(&key_bytes[..16])?;
            let nonce = Nonce::from_slice(&iv_bytes[..12]);
            cipher.encrypt(nonce, data_bytes.as_ref()).map_err(|e| anyhow!("AES-GCM 加密错误: {}", e))?
        }
        "des" => {
            use des::Des;
            use cbc::cipher::{BlockEncryptMut, KeyIvInit, generic_array::GenericArray};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 8]);
            let mut cipher = cbc::Encryptor::<Des>::new(key_bytes[..8].into(), iv_bytes[..8].into());
            let padded = pkcs7_pad(&data_bytes, 8);
            let mut out = padded.clone();
            for chunk in out.chunks_mut(8) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.encrypt_block_mut(block);
            }
            out
        }
        "3des" => {
            use des::TdesEde3;
            use cbc::cipher::{BlockEncryptMut, KeyIvInit, generic_array::GenericArray};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 8]);
            let mut cipher = cbc::Encryptor::<TdesEde3>::new(key_bytes[..24].into(), iv_bytes[..8].into());
            let padded = pkcs7_pad(&data_bytes, 8);
            let mut out = padded.clone();
            for chunk in out.chunks_mut(8) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.encrypt_block_mut(block);
            }
            out
        }
        "rc4" => {
            use rc4::{Rc4, StreamCipher};
            use rc4::cipher::KeyInit;
            let mut cipher = Rc4::<rc4::cipher::generic_array::typenum::U16>::new_from_slice(&key_bytes)?;
            let mut buf = data_bytes.clone();
            cipher.apply_keystream(&mut buf);
            buf
        }
        _ => return Err(anyhow!("不支持的加密算法: {}", algo)),
    };

    let display = match output_fmt {
        "base64" => general_purpose::STANDARD.encode(&result),
        _ => hex::encode(&result),
    };

    println!("{}", display);
    Ok(())
}

pub fn decrypt(algo: &str, key: &str, iv: Option<&str>, data: &str, input_fmt: &str, output_fmt: &str) -> Result<()> {
    let key_bytes = parse_input(key, "text")?;
    let data_bytes = parse_input(data, input_fmt)?;

    let result = match algo {
        "aes-cbc" => {
            use aes::Aes128;
            use cbc::cipher::{BlockDecryptMut, KeyIvInit, generic_array::GenericArray};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 16]);
            let mut cipher = cbc::Decryptor::<Aes128>::new(key_bytes[..16].into(), iv_bytes[..16].into());
            let mut buf = data_bytes.clone();
            for chunk in buf.chunks_mut(16) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.decrypt_block_mut(block);
            }
            pkcs7_unpad(&buf)?
        }
        "aes-ecb" => {
            use aes::Aes128;
            use ecb::cipher::{BlockDecryptMut, generic_array::GenericArray};
            use aes::cipher::KeyInit;
            let mut cipher = ecb::Decryptor::<Aes128>::new_from_slice(&key_bytes[..16])?;
            let mut buf = data_bytes.clone();
            for chunk in buf.chunks_mut(16) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.decrypt_block_mut(block);
            }
            pkcs7_unpad(&buf)?
        }
        "aes-gcm" => {
            use aes_gcm::{aead::Aead, Aes128Gcm, KeyInit, Nonce};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 12]);
            let cipher = Aes128Gcm::new_from_slice(&key_bytes[..16])?;
            let nonce = Nonce::from_slice(&iv_bytes[..12]);
            cipher.decrypt(nonce, data_bytes.as_ref()).map_err(|e| anyhow!("AES-GCM 解密错误: {}", e))?
        }
        "des" => {
            use des::Des;
            use cbc::cipher::{BlockDecryptMut, KeyIvInit, generic_array::GenericArray};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 8]);
            let mut cipher = cbc::Decryptor::<Des>::new(key_bytes[..8].into(), iv_bytes[..8].into());
            let mut buf = data_bytes.clone();
            for chunk in buf.chunks_mut(8) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.decrypt_block_mut(block);
            }
            pkcs7_unpad(&buf)?
        }
        "3des" => {
            use des::TdesEde3;
            use cbc::cipher::{BlockDecryptMut, KeyIvInit, generic_array::GenericArray};
            let iv_bytes = iv.map(|v| parse_input(v, "text")).transpose()?.unwrap_or_else(|| vec![0u8; 8]);
            let mut cipher = cbc::Decryptor::<TdesEde3>::new(key_bytes[..24].into(), iv_bytes[..8].into());
            let mut buf = data_bytes.clone();
            for chunk in buf.chunks_mut(8) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.decrypt_block_mut(block);
            }
            pkcs7_unpad(&buf)?
        }
        "rc4" => {
            use rc4::{Rc4, StreamCipher};
            use rc4::cipher::KeyInit;
            let mut cipher = Rc4::<rc4::cipher::generic_array::typenum::U16>::new_from_slice(&key_bytes)?;
            let mut buf = data_bytes.clone();
            cipher.apply_keystream(&mut buf);
            buf
        }
        _ => return Err(anyhow!("不支持的解密算法: {}", algo)),
    };

    let display = match output_fmt {
        "hex" => hex::encode(&result),
        "base64" => general_purpose::STANDARD.encode(&result),
        _ => String::from_utf8_lossy(&result).to_string(),
    };

    println!("{}", display);
    Ok(())
}

pub fn rsa_encrypt(_key_path: &std::path::Path, _data: &str) -> Result<()> {
    core::warn("RSA 加密需要密钥对, 请使�?openssl 生成密钥");
    core::info("openssl genrsa -out private.pem 2048");
    core::info("openssl rsa -in private.pem -pubout -out public.pem");
    Ok(())
}

pub fn rsa_decrypt(_key_path: &std::path::Path, _data: &str) -> Result<()> {
    core::warn("RSA 解密需要私�? 请使�?openssl 生成密钥");
    Ok(())
}

pub fn rsa_sign(_key_path: &std::path::Path, _data: &str, _algorithm: &str) -> Result<()> {
    core::warn("RSA 签名需要密钥对, 请使�?openssl 生成密钥");
    Ok(())
}

pub fn rsa_verify(_key_path: &std::path::Path, _data: &str, _signature: &str, _algorithm: &str) -> Result<()> {
    core::warn("RSA 验签需要公�? 请使�?openssl 生成密钥");
    Ok(())
}

pub fn random_ua(browser: &str, count: usize) -> Result<()> {
    let chrome_ua = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    ];
    let firefox_ua = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
    ];
    let safari_ua = [
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    ];

    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let ua = match browser {
            "chrome" => chrome_ua[rng.gen_range(0..chrome_ua.len())],
            "firefox" => firefox_ua[0],
            "safari" => safari_ua[0],
            _ => {
                let mut all = Vec::new();
                all.extend_from_slice(&chrome_ua);
                all.extend_from_slice(&firefox_ua);
                all.extend_from_slice(&safari_ua);
                all[rng.gen_range(0..all.len())]
            }
        };
        println!("{}", ua);
    }
    Ok(())
}

pub fn random_str(length: usize, count: usize) -> Result<()> {
    use rand::Rng;
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let s: String = (0..length).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
        println!("{}", s);
    }
    Ok(())
}
