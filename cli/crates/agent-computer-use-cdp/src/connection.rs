use crate::protocol::{CdpRequest, CdpResponse, TargetInfo, VersionInfo};
use agent_computer_use_core::Error;
use futures_util::{SinkExt, StreamExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
const WS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub struct CdpConnection {
    ws: Mutex<WsStream>,
    next_id: AtomicU64,
    pub port: u16,
}

impl CdpConnection {
    pub async fn connect(port: u16) -> agent_computer_use_core::Result<Self> {
        if let Some(cached_url) = load_cached_ws_url(port) {
            if let Ok(conn) = Self::connect_ws(&cached_url, port).await {
                return Ok(conn);
            }
        }

        let ws_url = Self::discover_ws_url(port).await?;
        save_ws_url_cache(port, &ws_url);
        Self::connect_ws(&ws_url, port).await
    }

    async fn connect_ws(ws_url: &str, port: u16) -> agent_computer_use_core::Result<Self> {
        tracing::debug!("connecting to CDP at {ws_url}");

        let (ws, _) = tokio::time::timeout(WS_TIMEOUT, tokio_tungstenite::connect_async(ws_url))
            .await
            .map_err(|_| Error::PlatformError {
                message: format!("CDP WebSocket connection timed out (port {port})"),
            })?
            .map_err(|e| Error::PlatformError {
                message: format!("CDP WebSocket connection failed: {e}"),
            })?;

        Ok(Self {
            ws: Mutex::new(ws),
            next_id: AtomicU64::new(1),
            port,
        })
    }

    async fn discover_ws_url(port: u16) -> agent_computer_use_core::Result<String> {
        if let Ok(targets) = Self::http_get_json::<Vec<TargetInfo>>(port, "/json").await {
            if let Some(target) = targets
                .iter()
                .find(|t| t.target_type == "page")
                .or_else(|| targets.first())
            {
                if let Some(ref url) = target.web_socket_debugger_url {
                    return Ok(url.clone());
                }
            }
        }

        let version = Self::http_get_json::<VersionInfo>(port, "/json/version").await?;
        version
            .web_socket_debugger_url
            .ok_or_else(|| Error::PlatformError {
                message: format!("CDP on port {port} did not provide a WebSocket URL"),
            })
    }

    async fn http_get_json<T: serde::de::DeserializeOwned>(
        port: u16,
        path: &str,
    ) -> agent_computer_use_core::Result<T> {
        let url = format!("http://127.0.0.1:{port}{path}");

        let stream = tokio::time::timeout(
            HTTP_TIMEOUT,
            tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")),
        )
        .await
        .map_err(|_| Error::PlatformError {
            message: format!("CDP connection timed out on port {port}"),
        })?
        .map_err(|e| Error::PlatformError {
            message: format!("cannot connect to CDP port {port}: {e}"),
        })?;

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = stream;
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| Error::PlatformError {
                message: format!("CDP HTTP write failed: {e}"),
            })?;

        let mut buf = Vec::with_capacity(4096);
        match tokio::time::timeout(HTTP_TIMEOUT, stream.read_to_end(&mut buf)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                return Err(Error::PlatformError {
                    message: format!("CDP HTTP read failed: {e}"),
                })
            }
            Err(_) if !buf.is_empty() => {
                tracing::debug!("CDP HTTP read timed out, using {} bytes", buf.len());
            }
            Err(_) => {
                return Err(Error::PlatformError {
                    message: format!("CDP HTTP read timed out on {url}"),
                })
            }
        }

        let response = String::from_utf8_lossy(&buf);
        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .ok_or_else(|| Error::PlatformError {
                message: format!("invalid HTTP response from {url}"),
            })?;

        serde_json::from_str(body).map_err(|e| Error::PlatformError {
            message: format!("invalid JSON from {url}: {e}"),
        })
    }

    pub async fn send(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> agent_computer_use_core::Result<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = CdpRequest {
            id,
            method: method.to_string(),
            params,
        };

        let msg = serde_json::to_string(&request).map_err(|e| Error::PlatformError {
            message: format!("CDP serialize error: {e}"),
        })?;

        let mut ws = self.ws.lock().await;
        ws.send(Message::Text(msg))
            .await
            .map_err(|e| Error::PlatformError {
                message: format!("CDP send error: {e}"),
            })?;

        let deadline = tokio::time::Instant::now() + WS_TIMEOUT;

        loop {
            let msg = tokio::time::timeout_at(deadline, ws.next())
                .await
                .map_err(|_| Error::PlatformError {
                    message: format!("CDP response timed out for {method}"),
                })?
                .ok_or_else(|| Error::PlatformError {
                    message: "CDP connection closed".into(),
                })?
                .map_err(|e| Error::PlatformError {
                    message: format!("CDP read error: {e}"),
                })?;

            let text = match msg {
                Message::Text(t) => t.to_string(),
                Message::Binary(b) => String::from_utf8_lossy(&b).to_string(),
                Message::Close(_) => {
                    return Err(Error::PlatformError {
                        message: "CDP connection closed".into(),
                    })
                }
                _ => continue,
            };

            let resp: CdpResponse =
                serde_json::from_str(&text).map_err(|e| Error::PlatformError {
                    message: format!("CDP parse error: {e}"),
                })?;

            if resp.id != Some(id) {
                continue;
            }

            if let Some(err) = resp.error {
                return Err(Error::PlatformError {
                    message: format!("CDP error: {} ({})", err.message, err.code),
                });
            }

            return Ok(resp.result.unwrap_or(serde_json::Value::Null));
        }
    }

    pub async fn evaluate(
        &self,
        expression: &str,
    ) -> agent_computer_use_core::Result<serde_json::Value> {
        let result = self
            .send(
                "Runtime.evaluate",
                Some(serde_json::json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": true,
                })),
            )
            .await?;

        if result.get("exceptionDetails").is_some() {
            let desc = result
                .pointer("/exceptionDetails/exception/description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown JS error");
            return Err(Error::PlatformError {
                message: format!("JS error: {desc}"),
            });
        }

        Ok(result
            .pointer("/result/value")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }
}

fn cache_path(port: u16) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(format!("{home}/.agent-cu/cdp-{port}.json"))
}

fn load_cached_ws_url(port: u16) -> Option<String> {
    let path = cache_path(port);
    let contents = std::fs::read_to_string(&path).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&contents).ok()?;

    let ts = cache.get("ts")?.as_u64()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now - ts > 300 {
        let _ = std::fs::remove_file(&path);
        return None;
    }

    cache.get("url")?.as_str().map(String::from)
}

fn save_ws_url_cache(port: u16, url: &str) {
    let path = cache_path(port);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let cache = serde_json::json!({ "url": url, "ts": now });
    let _ = std::fs::write(&path, serde_json::to_string(&cache).unwrap_or_default());
}

#[cfg(test)]
mod pid_tests {
    use super::*;
    use crate::{CdpConfig, ElectronAwarePlatform};
    use agent_computer_use_core::action::{Action, ActionResult};
    use agent_computer_use_core::platform::{AppInfo, WindowInfo};
    use agent_computer_use_core::{AccessibilityNode, Platform, Result, Selector};
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::{atomic::AtomicUsize, Arc};

    struct Native;

    #[async_trait]
    impl Platform for Native {
        async fn tree(&self, _: Option<&str>, _: Option<u32>) -> Result<AccessibilityNode> {
            unreachable!()
        }
        async fn find(&self, _: &Selector) -> Result<Vec<AccessibilityNode>> {
            unreachable!()
        }
        async fn perform(&self, action: &Action) -> Result<ActionResult> {
            assert_eq!(
                serde_json::to_value(action).unwrap(),
                json!({"type":"type", "text":"hello", "app":"pid:42", "submit":false})
            );
            Ok(ActionResult {
                success: true,
                message: Some("native PID typing".into()),
                path: None,
                data: None,
            })
        }
        async fn focused(&self) -> Result<AccessibilityNode> {
            unreachable!()
        }
        async fn applications(&self) -> Result<Vec<AppInfo>> {
            unreachable!()
        }
        async fn windows(&self, _: Option<&str>) -> Result<Vec<WindowInfo>> {
            unreachable!()
        }
        async fn text(&self, _: Option<&str>) -> Result<String> {
            unreachable!()
        }
        async fn check_permissions(&self) -> Result<bool> {
            unreachable!()
        }
        fn platform_name(&self) -> &'static str {
            "fixture"
        }
    }

    #[tokio::test]
    async fn pid_typing_does_not_reuse_another_apps_cached_connection() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::new(AtomicUsize::new(0));
        let seen = requests.clone();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(socket).await.unwrap();
            while let Some(Ok(Message::Text(message))) = ws.next().await {
                let request: serde_json::Value = serde_json::from_str(&message).unwrap();
                assert_eq!(request["method"], "Input.insertText");
                assert_eq!(request["params"], json!({"text":"hello"}));
                seen.fetch_add(1, Ordering::SeqCst);
                ws.send(Message::Text(
                    json!({"id":request["id"], "result":{}}).to_string(),
                ))
                .await
                .unwrap();
            }
        });
        let connection = CdpConnection::connect_ws(&format!("ws://127.0.0.1:{port}"), port)
            .await
            .unwrap();
        let platform = ElectronAwarePlatform::new(Native, CdpConfig::default());
        platform
            .connections
            .lock()
            .unwrap()
            .insert("Other app".into(), Arc::new(connection));
        let action = Action::Type {
            text: "hello".into(),
            app: None,
            selector: None,
            submit: false,
        };
        let legacy = platform.perform(&action).await.unwrap();
        assert_eq!(
            legacy.message.as_deref(),
            Some("typed 5 characters via CDP")
        );
        let targeted = Action::Type {
            text: "hello".into(),
            app: Some("pid:42".into()),
            selector: None,
            submit: false,
        };
        let result = platform.perform(&targeted).await.unwrap();
        assert_eq!(result.message.as_deref(), Some("native PID typing"));
        assert_eq!(requests.load(Ordering::SeqCst), 1);
        server.abort();
    }
}
