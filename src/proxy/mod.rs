use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::core::{self, Mode};

#[allow(dead_code)]
pub struct ProxyConfig {
    pub port: u16,
    pub filter: Option<String>,
    pub filter_mode: String,
    pub callback_url: Option<String>,
    pub callback_timeout: u64,
    pub callback_retries: u32,
    pub hook_fetch: bool,
    pub hook_function: Option<String>,
    pub no_static: bool,
    pub mode: Mode,
}

#[allow(dead_code)]
struct TlsManager {
    ca_cert: rcgen::Certificate,
    ca_key: rcgen::KeyPair,
    cert_cache: HashMap<String, Vec<u8>>,
}

impl TlsManager {
    fn new() -> Result<Self> {
        let cert_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".reptool").join("certs");
        fs::create_dir_all(&cert_dir)?;

        let ca_cert_path = cert_dir.join("ca.pem");
        let ca_key_path = cert_dir.join("ca-key.pem");

        let (ca_cert, ca_key) = if ca_cert_path.exists() && ca_key_path.exists() {
            // 尝试加载已有 CA
            let key_pem = fs::read_to_string(&ca_key_path)?;
            let key_pair = rcgen::KeyPair::from_pem(&key_pem)?;
            let mut params = rcgen::CertificateParams::new(vec!["Reptool Proxy CA".to_string()])?;
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            let cert = params.self_signed(&key_pair)?;
            (cert, key_pair)
        } else {
            let mut params = rcgen::CertificateParams::new(vec!["Reptool Proxy CA".to_string()])?;
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            let key_pair = rcgen::KeyPair::generate()?;
            let cert = params.self_signed(&key_pair)?;

            fs::write(&ca_cert_path, cert.pem())?;
            fs::write(&ca_key_path, key_pair.serialize_pem())?;

            core::info("CA 证书已生成");
            (cert, key_pair)
        };

        Ok(Self {
            ca_cert,
            ca_key,
            cert_cache: HashMap::new(),
        })
    }

    fn get_cert_for_domain(&mut self, domain: &str) -> Result<Vec<u8>> {
        if let Some(cert_pem) = self.cert_cache.get(domain) {
            return Ok(cert_pem.clone());
        }

        let mut params = rcgen::CertificateParams::new(vec![domain.to_string()])?;
        params.is_ca = rcgen::IsCa::NoCa;
        let key_pair = rcgen::KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        let cert_pem = cert.pem().into_bytes();
        self.cert_cache.insert(domain.to_string(), cert_pem.clone());

        Ok(cert_pem)
    }
}

pub fn start(config: ProxyConfig) -> Result<()> {
    core::info(&format!("启动 MITM 代理监听 0.0.0.0:{}", config.port));
    core::info(&format!("模式: {}", config.mode));

    if let Some(ref filter) = config.filter {
        core::info(&format!("URL 过滤 ({}) : {}", config.filter_mode, filter));
    }

    let rules = load_rules()?;
    let sessions: Arc<tokio::sync::RwLock<Vec<SessionEntry>>> = Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let tls_manager = Arc::new(tokio::sync::Mutex::new(TlsManager::new()?));

    let filter = config.filter.clone();
    let filter_mode = config.filter_mode.clone();
    let no_static = config.no_static;
    let hook_fetch = config.hook_fetch;
    let hook_function = config.hook_function.clone();
    let port = config.port;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        core::success(&format!("代理已启动, 监听 0.0.0.0:{}", port));
        core::info("按 Ctrl+C 停止");

        loop {
            let (stream, _addr) = listener.accept().await?;
            let filter = filter.clone();
            let filter_mode = filter_mode.clone();
            let rules = rules.clone();
            let sessions = sessions.clone();
            let tls_manager = tls_manager.clone();
            let hook_fetch = hook_fetch;
            let hook_function = hook_function.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(
                    stream, &filter, &filter_mode, no_static, &rules, &sessions, &tls_manager, hook_fetch, hook_function.as_deref(),
                ).await {
                    tracing::debug!("连接处理错误: {}", e);
                }
            });
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

pub fn stop() -> Result<()> {
    core::info("停止代理...");
    core::success("代理已停止");
    Ok(())
}

pub fn export_cert(output: &Path) -> Result<()> {
    let cert_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".reptool").join("certs");
    let ca_cert_path = cert_dir.join("ca.pem");

    if !ca_cert_path.exists() {
        core::warn("CA 证书不存在, 请先启动一次代理生成证书");
        return Ok(());
    }

    fs::create_dir_all(output.parent().unwrap_or(Path::new(".")))?;
    fs::copy(&ca_cert_path, output)?;

    core::success(&format!("CA 证书已导出到 {}", output.display()));
    println!("\n{}", "安装证书到系统信任存储:".yellow().bold());
    println!("  Windows: certutil -addstore Root \"{}\"", output.display());
    println!("  macOS:   sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/SystemKeychain \"{}\"", output.display());
    println!("  Linux:   sudo cp \"{}\" /usr/local/share/ca-certificates/reptool-ca.crt && sudo update-ca-certificates", output.display());

    Ok(())
}

pub fn add_rule(rule: &str) -> Result<()> {
    let mut rules = load_rules()?;
    rules.push(rule.to_string());
    save_rules(&rules)?;
    core::success(&format!("规则已添加: {}", rule));
    Ok(())
}

pub fn list_rules() -> Result<()> {
    let rules = load_rules()?;
    if rules.is_empty() {
        core::info("无篡改规则");
    } else {
        for (i, rule) in rules.iter().enumerate() {
            println!("[{}] {}", i, rule);
        }
    }
    Ok(())
}

pub fn remove_rule(index: usize) -> Result<()> {
    let mut rules = load_rules()?;
    if index >= rules.len() {
        return Err(anyhow!("索引 {} 超出范围", index));
    }
    rules.remove(index);
    save_rules(&rules)?;
    core::success("规则已删除");
    Ok(())
}

pub fn clear_rules() -> Result<()> {
    save_rules(&[])?;
    core::success("所有规则已清空");
    Ok(())
}

pub fn show_sessions(filter: Option<&str>, limit: usize) -> Result<()> {
    let _re = filter.map(|f| regex::Regex::new(f)).transpose()?;
    let sessions_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".reptool").join("sessions");

    if !sessions_dir.exists() {
        core::info("无抓包会话");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&sessions_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
        .collect();

    entries.sort();
    entries.reverse();

    let mut count = 0;
    for entry in entries.iter().take(limit) {
        if let Ok(content) = fs::read_to_string(entry) {
            if let Ok(session) = serde_json::from_str::<serde_json::Value>(&content) {
                let url = session.get("url").and_then(|v| v.as_str()).unwrap_or("?");
                let method = session.get("method").and_then(|v| v.as_str()).unwrap_or("?");
                let status = session.get("status").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("{} {} → {}", method, url, status);
                count += 1;
            }
        }
    }

    core::info(&format!("显示 {} 条会话", count));
    Ok(())
}

#[allow(dead_code)]
struct SessionEntry {
    method: String,
    url: String,
    status: u16,
}

fn rules_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".reptool").join("proxy_rules.json")
}

fn load_rules() -> Result<Vec<String>> {
    let path = rules_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

fn save_rules(rules: &[String]) -> Result<()> {
    let path = rules_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, serde_json::to_string(rules)?)?;
    Ok(())
}

fn build_hook_js() -> String {
    r#"
(function() {
    // Hook XMLHttpRequest
    const origOpen = XMLHttpRequest.prototype.open;
    const origSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function(method, url) {
        this._hookMethod = method;
        this._hookUrl = url;
        return origOpen.apply(this, arguments);
    };
    XMLHttpRequest.prototype.send = function(body) {
        if (body) {
            console.log('%c[XHR] ' + this._hookMethod + ' ' + this._hookUrl, 'color: #ff9900');
            console.log('%c[XHR Body]', 'color: #ff9900', body);
        }
        return origSend.apply(this, arguments);
    };

    // Hook fetch
    const origFetch = window.fetch;
    window.fetch = function() {
        const args = arguments;
        console.log('%c[FETCH]', 'color: #cc00ff', args[0]?.url || args[0], args[1]);
        return origFetch.apply(this, arguments).then(function(resp) {
            resp.clone().text().then(function(t) {
                console.log('%c[FETCH Response]', 'color: #cc00ff', t);
            });
            return resp;
        });
    };

    console.log('%c[Reptool] Hook 已激活', 'color: #00ff00; font-weight: bold; font-size: 14px');
})();
"#.to_string()
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    filter: &Option<String>,
    filter_mode: &str,
    no_static: bool,
    rules: &[String],
    sessions: &Arc<tokio::sync::RwLock<Vec<SessionEntry>>>,
    tls_manager: &Arc<tokio::sync::Mutex<TlsManager>>,
    hook_fetch: bool,
    hook_function: Option<&str>,
) -> Result<()> {
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or("");

    if first_line.starts_with("CONNECT") {
        handle_connect(&mut stream, first_line, filter, filter_mode, rules, sessions, tls_manager, hook_fetch, hook_function).await?;
    } else {
        handle_http(&mut stream, &buf[..n], filter, filter_mode, no_static, rules, sessions, hook_fetch, hook_function).await?;
    }

    Ok(())
}

async fn handle_connect(
    stream: &mut tokio::net::TcpStream,
    first_line: &str,
    filter: &Option<String>,
    filter_mode: &str,
    _rules: &[String],
    sessions: &Arc<tokio::sync::RwLock<Vec<SessionEntry>>>,
    tls_manager: &Arc<tokio::sync::Mutex<TlsManager>>,
    hook_fetch: bool,
    _hook_function: Option<&str>,
) -> Result<()> {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let host_port = parts.get(1).unwrap_or(&"");
    let host = host_port.split(':').next().unwrap_or("");

    if let Some(ref f) = filter {
        let matched = match regex::Regex::new(f) {
            Ok(re) => re.is_match(host_port),
            Err(_) => host_port.contains(f.as_str()),
        };
        if (filter_mode == "whitelist") != matched {
            stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
            return Ok(());
        }
    }

    // 通知客户端隧道已建立
    stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;

    // 获取该域名的证书
    let domain = host.to_string();
    let _cert_pem = {
        let mut mgr = tls_manager.lock().await;
        mgr.get_cert_for_domain(&domain)?
    };

    // 读取 CA 私钥
    let ca_key_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".reptool").join("certs").join("ca-key.pem");
    let ca_key_pem = fs::read_to_string(&ca_key_path)?;
    let ca_key = rcgen::KeyPair::from_pem(&ca_key_pem)?;

    // 为该域名生成证书
    let mut params = rcgen::CertificateParams::new(vec![domain.clone()])?;
    params.is_ca = rcgen::IsCa::NoCa;
    let domain_key = rcgen::KeyPair::generate()?;
    let domain_cert = params.self_signed(&ca_key)?;

    let cert_der = domain_cert.der().to_vec();
    let key_der = domain_key.serialize_der();

    // 构建 rustls ServerConfig
    let cert_chain = vec![rustls_pki_types::CertificateDer::from(cert_der)];
    let key = rustls_pki_types::PrivatePkcs1KeyDer::from(key_der);
    let key = rustls_pki_types::PrivateKeyDer::Pkcs1(key);

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;

    let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));

    // TLS 握手
    let tls_stream = match tls_acceptor.accept(stream).await {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("TLS 握手失败 {}: {}", domain, e);
            return Ok(());
        }
    };

    // 分离读写
    let (mut reader, mut writer) = tokio::io::split(tls_stream);

    // 读取客户端发送的 HTTP 请求
    let mut client_buf = vec![0u8; 8192];
    let n = reader.read(&mut client_buf).await?;
    if n == 0 {
        return Ok(());
    }

    let client_request = String::from_utf8_lossy(&client_buf[..n]);
    let first_line = client_request.lines().next().unwrap_or("");
    let req_parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = req_parts.first().unwrap_or(&"GET");
    let path = req_parts.get(1).copied().unwrap_or("/");

    let full_url = format!("https://{}{}", host, path);

    // 检查是否需要注入 hook
    let mut response_body = Vec::new();

    // 连接目标服务器
    let target_addr = format!("{}:443", host);
    let target_stream = match tokio::net::TcpStream::connect(&target_addr).await {
        Ok(s) => s,
        Err(e) => {
            let err_resp = format!("HTTP/1.1 502 Bad Gateway\r\n\r\n{}", e);
            let _ = writer.write_all(err_resp.as_bytes()).await;
            return Ok(());
        }
    };

    // TLS 连接到目标服务器
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name = rustls_pki_types::ServerName::try_from(host.to_string())?;
    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
    let mut target_tls = connector.connect(server_name, target_stream).await?;

    // 转发客户端请求到目标服务器
    let _ = target_tls.write_all(&client_buf[..n]).await;

    // 读取目标服务器响应
    let mut target_buf = vec![0u8; 8192];
    loop {
        match target_tls.read(&mut target_buf).await {
            Ok(0) => break,
            Ok(n) => {
                response_body.extend_from_slice(&target_buf[..n]);

                // 检查是否是 HTML 响应，需要注入 hook
                if hook_fetch {
                    let resp_str = String::from_utf8_lossy(&response_body);
                    if resp_str.contains("text/html") || resp_str.contains("<head>") || resp_str.contains("<body>") {
                        // 注入 hook 脚本
                        let hook_js = build_hook_js();
                        let inject_tag = format!("<script>{}</script>", hook_js);

                        if let Some(pos) = resp_str.find("</head>") {
                            let mut modified = response_body.clone();
                            let _head_end = pos + 7;
                            // 找到 </head> 在原始字节中的位置
                            let head_tag = b"</head>";
                            if let Some(byte_pos) = find_subsequence(&modified, head_tag) {
                                modified.splice(byte_pos..byte_pos, inject_tag.bytes());
                                let _ = writer.write_all(&modified).await;
                                continue;
                            }
                        }
                    }
                }

                let _ = writer.write_all(&target_buf[..n]).await;
            }
            Err(_) => break,
        }
    }

    // 记录会话
    let status_line = String::from_utf8_lossy(&response_body)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();

    let status_code: u16 = status_line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    sessions.write().await.push(SessionEntry {
        method: method.to_string(),
        url: full_url,
        status: status_code,
    });

    Ok(())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

async fn handle_http(
    stream: &mut tokio::net::TcpStream,
    raw: &[u8],
    filter: &Option<String>,
    filter_mode: &str,
    no_static: bool,
    _rules: &[String],
    sessions: &Arc<tokio::sync::RwLock<Vec<SessionEntry>>>,
    _hook_fetch: bool,
    _hook_function: Option<&str>,
) -> Result<()> {
    let request = String::from_utf8_lossy(raw);
    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = parts.first().unwrap_or(&"GET");
    let url = parts.get(1).unwrap_or(&"/");

    if let Some(ref f) = filter {
        let matched = match regex::Regex::new(f) {
            Ok(re) => re.is_match(url),
            Err(_) => url.contains(f.as_str()),
        };
        if (filter_mode == "whitelist") != matched {
            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await?;
            return Ok(());
        }
    }

    if no_static {
        if url.ends_with(".css") || url.ends_with(".js") || url.ends_with(".png") || url.ends_with(".jpg") || url.ends_with(".gif") {
            stream.write_all(b"HTTP/1.1 204 No Content\r\n\r\n").await?;
            return Ok(());
        }
    }

    let parsed_url = url::Url::parse(url).or_else(|_| url::Url::parse(&format!("http://{}", url)))?;
    let host = parsed_url.host_str().unwrap_or("localhost");
    let port = parsed_url.port_or_known_default().unwrap_or(80);

    let target_addr = format!("{}:{}", host, port);
    let mut target_stream = tokio::net::TcpStream::connect(&target_addr).await?;

    let _ = target_stream.write_all(raw).await;

    let mut response_buf = Vec::new();
    let mut temp = vec![0u8; 8192];
    loop {
        match target_stream.read(&mut temp).await {
            Ok(0) => break,
            Ok(n) => response_buf.extend_from_slice(&temp[..n]),
            Err(_) => break,
        }
    }

    let _ = stream.write_all(&response_buf).await;

    let status_line = String::from_utf8_lossy(&response_buf)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();

    let status_code: u16 = status_line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    sessions.write().await.push(SessionEntry {
        method: method.to_string(),
        url: url.to_string(),
        status: status_code,
    });

    Ok(())
}
