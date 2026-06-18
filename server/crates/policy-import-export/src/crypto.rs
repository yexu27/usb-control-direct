//! SM4/SM3/SM2 国密算法封装。
//!
//! 对 `smcrypto` crate 的 SM4-CBC 加解密、SM3 摘要和 SM2 签名验签
//! 进行薄封装，统一错误处理和返回类型。

use rand::Rng;
use smcrypto::sm2;
use smcrypto::sm3;
use smcrypto::sm4;

use crate::error::PolicyError;

/// IV 长度（字节）。
pub const IV_LEN: usize = 16;

/// SM4-CBC 加密。
///
/// 参数:
/// - `key`: SM4 密钥（16 字节）。
/// - `iv`: 初始化向量（16 字节）。
/// - `plaintext`: 待加密明文。
///
/// 返回:
/// - 加密后的密文字节序列。
pub fn sm4_cbc_encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let cipher = sm4::CryptSM4CBC::new(key, iv);
    cipher.encrypt_cbc(plaintext)
}

/// SM4-CBC 解密。
///
/// 参数:
/// - `key`: SM4 密钥（16 字节）。
/// - `iv`: 初始化向量（16 字节）。
/// - `ciphertext`: 待解密密文。
///
/// 返回:
/// - 成功时返回解密后的明文；失败时返回 [`PolicyError::DecryptError`]。
pub fn sm4_cbc_decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, PolicyError> {
    if ciphertext.is_empty() {
        return Err(PolicyError::DecryptError("密文为空".into()));
    }
    let cipher = sm4::CryptSM4CBC::new(key, iv);
    // catch_unwind 防止底层库在填充异常时 panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cipher.decrypt_cbc(ciphertext)
    }));
    match result {
        Ok(plaintext) => {
            if plaintext.is_empty() {
                return Err(PolicyError::DecryptError("解密结果为空，密文可能已损坏".into()));
            }
            Ok(plaintext)
        }
        Err(_) => Err(PolicyError::DecryptError("解密过程异常".into())),
    }
}

/// SM3 摘要计算。
///
/// 参数:
/// - `data`: 待计算摘要的数据。
///
/// 返回:
/// - 成功时返回 32 字节摘要值；失败时返回 [`PolicyError::FormatError`]。
pub fn sm3_hash(data: &[u8]) -> Result<Vec<u8>, PolicyError> {
    let hex_str = sm3::sm3_hash(data);
    hex_decode(&hex_str)
}

/// SM2 签名。
///
/// 参数:
/// - `private_key_hex`: SM2 私钥（hex 编码字符串）。
/// - `data`: 待签名数据。
///
/// 返回:
/// - 成功时返回签名字节序列；失败时返回 [`PolicyError`]。
pub fn sm2_sign(private_key_hex: &str, data: &[u8]) -> Result<Vec<u8>, PolicyError> {
    let signer = sm2::Sign::new(private_key_hex);
    let signature = signer.sign(data);
    if signature.is_empty() {
        return Err(PolicyError::SignatureError);
    }
    Ok(signature)
}

/// SM2 验签。
///
/// 参数:
/// - `public_key_hex`: SM2 公钥（hex 编码字符串）。
/// - `data`: 原始数据。
/// - `signature`: 待验证的签名。
///
/// 返回:
/// - 成功时返回验证结果（true/false）；失败时返回 [`PolicyError`]。
pub fn sm2_verify(public_key_hex: &str, data: &[u8], signature: &[u8]) -> Result<bool, PolicyError> {
    let verifier = sm2::Verify::new(public_key_hex);
    Ok(verifier.verify(data, signature))
}

/// 生成 16 字节随机 IV。
///
/// 返回:
/// - 16 字节随机初始化向量。
pub fn generate_random_iv() -> [u8; IV_LEN] {
    let mut iv = [0u8; IV_LEN];
    rand::thread_rng().fill(&mut iv);
    iv
}

/// 将 hex 字符串解码为字节数组。
///
/// 输入必须为偶数长度的合法十六进制字符串（来自 sm3_hash 输出）。
///
/// 返回:
/// - 成功时返回解码后的字节序列；失败时返回 [`PolicyError::FormatError`]。
fn hex_decode(hex: &str) -> Result<Vec<u8>, PolicyError> {
    if hex.len() % 2 != 0 {
        return Err(PolicyError::FormatError(format!(
            "hex 字符串长度必须为偶数，实际长度: {}",
            hex.len()
        )));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| {
                PolicyError::FormatError(format!("非法十六进制字符: {:?}", &hex[i..i + 2]))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sm4_encrypt_decrypt_roundtrip() {
        let key = b"1234567890abcdef";
        let iv = b"abcdef1234567890";
        let plaintext = b"hello world test data for sm4 encryption";

        let ciphertext = sm4_cbc_encrypt(key, iv, plaintext);
        assert!(!ciphertext.is_empty());
        assert_ne!(ciphertext, plaintext);

        let decrypted = sm4_cbc_decrypt(key, iv, &ciphertext).expect("解密失败");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn sm3_hash_produces_32_bytes() {
        let data = b"test data for sm3 hash";
        let digest = sm3_hash(data).expect("sm3_hash 失败");
        assert_eq!(digest.len(), 32);
    }

    #[test]
    fn sm3_hash_deterministic() {
        let data = b"deterministic test";
        let digest1 = sm3_hash(data).expect("sm3_hash 第一次失败");
        let digest2 = sm3_hash(data).expect("sm3_hash 第二次失败");
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn sm2_sign_verify_roundtrip() {
        let (private_key, public_key) = sm2::gen_keypair();
        let data = b"test data for sm2 signing";

        let signature = sm2_sign(&private_key, data).expect("签名失败");
        assert!(!signature.is_empty());

        let valid = sm2_verify(&public_key, data, &signature).expect("验签失败");
        assert!(valid, "签名验证应通过");
    }

    #[test]
    fn sm2_verify_rejects_tampered_data() {
        let (private_key, public_key) = sm2::gen_keypair();
        let data = b"original data";

        let signature = sm2_sign(&private_key, data).expect("签名失败");

        let tampered = b"tampered data";
        let valid = sm2_verify(&public_key, tampered, &signature).expect("验签失败");
        assert!(!valid, "篡改数据后签名验证应失败");
    }

    #[test]
    fn generate_random_iv_unique() {
        let iv1 = generate_random_iv();
        let iv2 = generate_random_iv();
        assert_ne!(iv1, iv2, "两次生成的 IV 应不同");
    }

    #[test]
    fn sm4_decrypt_empty_ciphertext_returns_error() {
        let key = b"1234567890abcdef";
        let iv = b"abcdef1234567890";
        let result = sm4_cbc_decrypt(key, iv, &[]);
        assert!(result.is_err());
    }
}
