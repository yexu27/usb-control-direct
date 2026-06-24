//! TLS 配置加载。
//!
//! 基于 tokio-rustls，单向 TLS（服务端证书）。
//! 证书和私钥从文件路径加载。

use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use sha2::{Digest, Sha256};

use crate::error::GatewayError;

/// 计算 DER 编码证书的 SHA-256 指纹。
fn compute_fingerprint(cert_der: &[u8]) -> String {
    let hash = Sha256::digest(cert_der);
    hash.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

/// 从 PEM 文件加载证书链。
fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, GatewayError> {
    let file = fs::File::open(path).map_err(|e| {
        GatewayError::TlsConfig(format!("无法打开证书文件 {}: {}", path.display(), e))
    })?;
    let mut reader = BufReader::new(file);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| GatewayError::TlsConfig(format!("解析证书失败: {}", e)))?;
    if certs.is_empty() {
        return Err(GatewayError::TlsConfig("证书文件为空".into()));
    }
    Ok(certs)
}

/// 从 PEM 文件加载私钥。
fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>, GatewayError> {
    let file = fs::File::open(path).map_err(|e| {
        GatewayError::TlsConfig(format!("无法打开私钥文件 {}: {}", path.display(), e))
    })?;
    let mut reader = BufReader::new(file);
    let key = rustls_pemfile::private_key(&mut reader)
        .map_err(|e| GatewayError::TlsConfig(format!("解析私钥失败: {}", e)))?
        .ok_or_else(|| GatewayError::TlsConfig("私钥文件为空".into()))?;
    Ok(key)
}

/// 创建 TLS acceptor。
///
/// 参数:
///   - `cert_path`: 证书 PEM 文件路径。
///   - `key_path`: 私钥 PEM 文件路径。
///
/// 返回:
///   - `TlsAcceptor` 和叶证书的 SHA-256 指纹（冒号分隔十六进制）。
pub fn create_tls_acceptor(
    cert_path: &Path,
    key_path: &Path,
) -> Result<(TlsAcceptor, String), GatewayError> {
    let certs = load_certs(cert_path)?;
    let fingerprint = compute_fingerprint(certs[0].as_ref());
    let key = load_private_key(key_path)?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| GatewayError::TlsConfig(format!("TLS 配置构建失败: {}", e)))?;

    Ok((TlsAcceptor::from(Arc::new(config)), fingerprint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn generate_self_signed() -> (NamedTempFile, NamedTempFile) {
        let key_pair = rcgen::KeyPair::generate().unwrap();
        let cert = rcgen::CertificateParams::new(vec!["localhost".into()])
            .unwrap()
            .self_signed(&key_pair)
            .unwrap();

        let mut cert_file = NamedTempFile::new().unwrap();
        cert_file.write_all(cert.pem().as_bytes()).unwrap();

        let mut key_file = NamedTempFile::new().unwrap();
        key_file.write_all(key_pair.serialize_pem().as_bytes()).unwrap();

        (cert_file, key_file)
    }

    #[test]
    fn create_tls_acceptor_with_valid_cert() {
        let (cert_file, key_file) = generate_self_signed();
        let result = create_tls_acceptor(cert_file.path(), key_file.path());
        assert!(result.is_ok());
        let (_acceptor, fingerprint) = result.unwrap();
        // SHA-256 指纹格式: 32 字节 × 3 - 1 = 95 字符（XX:XX:...）
        assert_eq!(fingerprint.len(), 95);
        assert!(fingerprint.contains(':'));
    }

    #[test]
    fn create_tls_acceptor_missing_cert_fails() {
        let key_file = NamedTempFile::new().unwrap();
        let result = create_tls_acceptor(Path::new("/nonexistent/cert.pem"), key_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn create_tls_acceptor_empty_cert_fails() {
        let cert_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();
        let result = create_tls_acceptor(cert_file.path(), key_file.path());
        assert!(result.is_err());
    }
}
