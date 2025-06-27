use crate::chains::{Chain, ConnectionType};
use crate::methods::RpcMethod;
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use futures::future::{self, Either};
use futures::{pin_mut, SinkExt, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use tokio::net::TcpStream;
use std::collections::HashMap;
use log::{debug, info, warn, error};

/// 配置常量
pub struct Config {
    pub http_timeout_secs: u64,
    pub ws_timeout_secs: u64,
    pub request_delay_ms: u64,
    pub max_concurrent_requests: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http_timeout_secs: 10,
            ws_timeout_secs: 15,
            request_delay_ms: 100,
            max_concurrent_requests: 10,
        }
    }
}

/// 自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("网络连接错误: {0}")]
    NetworkError(String),
    #[error("JSON-RPC错误: {0}")]
    JsonRpcError(String),
    #[error("WebSocket连接错误: {0}")]
    WebSocketError(String),
    #[error("超时错误: {0}")]
    TimeoutError(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
}

/// RPC 调用的结果
#[derive(Debug, Clone)]
pub struct RpcResult {
    /// 链名称
    pub chain: String,
    /// 端点URL
    pub endpoint: String,
    /// 方法名称
    pub method: String,
    /// 调用是否成功
    pub success: bool,
    /// 调用延迟（毫秒）
    pub latency_ms: f64,
    /// 如果调用失败，则包含错误信息
    pub error: Option<String>,
    /// 时间戳
    pub timestamp: chrono::DateTime<Utc>,
}

/// WebSocket连接管理器
pub struct WebSocketManager {
    connections: HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>,
    config: Config,
}

impl WebSocketManager {
    pub fn new(config: Config) -> Self {
        Self {
            connections: HashMap::new(),
            config,
        }
    }

    /// 获取或创建WebSocket连接
    async fn get_connection(&mut self, url: &str) -> Result<&mut WebSocketStream<MaybeTlsStream<TcpStream>>, RpcError> {
        if !self.connections.contains_key(url) {
            debug!("创建新的WebSocket连接: {}", url);
            let ws_url = Url::parse(url)
                .map_err(|e| RpcError::ConfigError(format!("无效的WebSocket URL: {}", e)))?;
            
            let (ws_stream, _) = connect_async(ws_url).await
                .map_err(|e| RpcError::WebSocketError(format!("连接失败: {}", e)))?;
            
            self.connections.insert(url.to_string(), ws_stream);
            info!("WebSocket连接已建立: {}", url);
        }
        
        Ok(self.connections.get_mut(url).unwrap())
    }

    /// 发送WebSocket RPC请求
    pub async fn send_request(
        &mut self,
        url: &str,
        method: &str,
        params: &[Value],
    ) -> Result<(bool, f64, Option<String>, Value), RpcError> {
        let start = Instant::now();
        
        // 先获取超时配置，避免借用冲突
        let ws_timeout_secs = self.config.ws_timeout_secs;
        
        // 获取连接
        let ws_stream = self.get_connection(url).await?;
        
        // 创建请求
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        
        // 发送请求
        debug!("发送WebSocket请求: {} 到 {}", method, url);
        let request_message = Message::Text(request_body.to_string());
        ws_stream.send(request_message).await
            .map_err(|e| RpcError::WebSocketError(format!("发送消息失败: {}", e)))?;
        
        // 等待响应
        let timeout_duration = Duration::from_secs(ws_timeout_secs);
        let timeout = tokio::time::sleep(timeout_duration);
        let response_future = ws_stream.next();
        
        pin_mut!(response_future);
        pin_mut!(timeout);
        
        let response = match future::select(response_future, timeout).await {
            Either::Left((Some(Ok(response)), _)) => {
                debug!("收到WebSocket响应");
                response
            },
            Either::Left((Some(Err(e)), _)) => {
                return Err(RpcError::WebSocketError(format!("响应错误: {}", e)));
            },
            Either::Left((None, _)) => {
                return Err(RpcError::WebSocketError("连接已关闭".to_string()));
            },
            Either::Right((_, _)) => {
                return Err(RpcError::TimeoutError(format!("请求超时({}秒)", ws_timeout_secs)));
            },
        };
        
        let latency = start.elapsed().as_secs_f64() * 1000.0;
        
        // 处理响应
        let response_text = match response {
            Message::Text(text) => text,
            _ => return Err(RpcError::WebSocketError("收到非文本响应".to_string())),
        };
        
        let response_body: Value = serde_json::from_str(&response_text)
            .map_err(|e| RpcError::JsonRpcError(format!("解析响应失败: {}", e)))?;
        
        let success = response_body.get("error").is_none();
        let error = if !success {
            response_body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };
        
        Ok((success, latency, error, response_body))
    }

    /// 关闭所有连接
    pub async fn close_all(&mut self) {
        for (url, mut stream) in self.connections.drain() {
            debug!("关闭WebSocket连接: {}", url);
            let _ = stream.send(Message::Close(None)).await;
        }
    }
}

/// RPC客户端管理器
pub struct RpcManager {
    http_client: Client,
    ws_manager: WebSocketManager,
    config: Config,
}

impl RpcManager {
    pub fn new(config: Config) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.http_timeout_secs))
            .build()
            .expect("创建HTTP客户端失败");
        
        let ws_manager = WebSocketManager::new(Config::default());
        
        Self {
            http_client,
            ws_manager,
            config,
        }
    }

    /// 发送HTTP RPC请求
    async fn send_http_request(
        &self,
        rpc_url: &str,
        method: &str,
        params: &[Value],
    ) -> Result<(bool, f64, Option<String>, Value), RpcError> {
        let start = Instant::now();
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        
        debug!("发送HTTP请求: {} 到 {}", method, rpc_url);
        
        let response = self.http_client
            .post(rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| RpcError::NetworkError(format!("HTTP请求失败: {}", e)))?;
        
        let latency = start.elapsed().as_secs_f64() * 1000.0;
        
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| RpcError::JsonRpcError(format!("解析HTTP响应失败: {}", e)))?;
        
        let success = response_body.get("error").is_none();
        let error = if !success {
            response_body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };
        
        Ok((success, latency, error, response_body))
    }

    /// 测试单个RPC方法
    pub async fn test_method(&mut self, chain: &Chain, method: &RpcMethod) -> RpcResult {
        let result = match chain.connection_type {
            ConnectionType::Http => {
                self.send_http_request(&chain.rpc_url, &method.name, &method.params).await
            },
            ConnectionType::WebSocket => {
                self.ws_manager.send_request(&chain.rpc_url, &method.name, &method.params).await
            }
        };

        match result {
            Ok((success, latency_ms, error, _)) => {
                RpcResult {
                    chain: chain.name.clone(),
                    endpoint: chain.rpc_url.clone(),
                    method: method.name.clone(),
                    success,
                    latency_ms,
                    error,
                    timestamp: Utc::now(),
                }
            },
            Err(e) => {
                error!("RPC调用失败: {}", e);
                RpcResult {
                    chain: chain.name.clone(),
                    endpoint: chain.rpc_url.clone(),
                    method: method.name.clone(),
                    success: false,
                    latency_ms: 0.0,
                    error: Some(e.to_string()),
                    timestamp: Utc::now(),
                }
            }
        }
    }

    /// 关闭所有连接
    pub async fn close(&mut self) {
        self.ws_manager.close_all().await;
    }
}

/// 测试所有方法
pub async fn test_all_methods(
    chains: &[Chain],
    methods: &[RpcMethod],
    count_per_method: usize,
) -> Result<Vec<RpcResult>> {
    let config = Config::default();
    let mut rpc_manager = RpcManager::new(config);
    let mut all_results = Vec::new();
    
    info!("开始测试 {} 个链上的 {} 个方法", chains.len(), methods.len());
    
    for (chain_idx, chain) in chains.iter().enumerate() {
        println!("测试链[{}/{}]: {} ({}) - 端点: {}", 
                 chain_idx + 1, chains.len(), chain.name, 
                 if chain.connection_type == ConnectionType::WebSocket { "WebSocket" } else { "HTTP" },
                 chain.rpc_url);
        
        for (i, method) in methods.iter().enumerate() {
            print!("[{}/{}] 测试方法: {} ... ", i + 1, methods.len(), method.name);
            
            let mut method_results = Vec::with_capacity(count_per_method);
            let mut error_occurred = false;
            let mut last_error = String::new();
            
            for attempt in 0..count_per_method {
                let result = rpc_manager.test_method(chain, method).await;
                
                if !result.success {
                    error_occurred = true;
                    if let Some(ref error) = result.error {
                        last_error = error.clone();
                    }
                    
                    // 对于WebSocket连接，如果第一次请求失败，输出详细错误并终止后续尝试
                    if attempt == 0 && chain.connection_type == ConnectionType::WebSocket {
                        debug!("WebSocket请求失败: {:?}", result.error);
                        method_results.push(result);
                        break;
                    }
                } else if attempt == 0 && chain.connection_type == ConnectionType::WebSocket {
                    debug!("WebSocket请求成功");
                }
                
                method_results.push(result);
                
                // 添加短暂延迟，避免过度请求
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            let success_count = method_results.iter().filter(|r| r.success).count();
            let avg_latency = if success_count > 0 {
                method_results.iter().filter(|r| r.success).map(|r| r.latency_ms).sum::<f64>() / success_count as f64
            } else {
                0.0
            };
            
            if error_occurred && chain.connection_type == ConnectionType::WebSocket {
                println!("完成 ({}/{}成功, 平均: {:.2}ms) - 错误: {}", 
                         success_count, count_per_method, avg_latency, last_error);
            } else {
                println!("完成 ({}/{}成功, 平均: {:.2}ms)", 
                         success_count, count_per_method, avg_latency);
            }
            
            all_results.extend(method_results);
            
            // 如果这是WebSocket链且第一个方法就失败，那么跳过该链的其他测试
            if i == 0 && success_count == 0 && chain.connection_type == ConnectionType::WebSocket {
                debug!("WebSocket链连接无法建立，跳过其他测试");
                break;
            }
        }
    }
    
    // 关闭所有连接
    rpc_manager.close().await;
    
    Ok(all_results)
}

