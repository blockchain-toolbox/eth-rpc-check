use crate::chains::{Chain, ConnectionType};
use crate::methods::RpcMethod;
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use futures::future::{self, Either};
use futures::{pin_mut, SinkExt, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

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

/// 通过HTTP发送 RPC 请求并返回结果
async fn send_http_rpc_request(
    client: &Client,
    rpc_url: &str,
    method: &str,
    params: &[Value],
) -> Result<(bool, f64, Option<String>, Value)> {
    let start = Instant::now();
    
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });
    
    let response = client
        .post(rpc_url)
        .json(&request_body)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("发送 HTTP RPC 请求失败")?;
    
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    
    let response_body: Value = response
        .json()
        .await
        .context("解析 HTTP RPC 响应失败")?;
    
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

/// 通过WebSocket发送 RPC 请求并返回结果
async fn send_ws_rpc_request(
    rpc_url: &str,
    method: &str,
    params: &[Value],
) -> Result<(bool, f64, Option<String>, Value)> {
    let start = Instant::now();
    
    // 解析WebSocket URL
    let url = Url::parse(rpc_url).context("解析WebSocket URL失败")?;
    
    println!("  [调试] 连接WebSocket: {}", rpc_url);
    
    // 连接到WebSocket
    let (mut ws_stream, _) = match connect_async(url).await {
        Ok(stream) => {
            println!("  [调试] WebSocket连接成功");
            stream
        },
        Err(e) => {
            println!("  [调试] WebSocket连接失败: {}", e);
            return Err(anyhow!("连接WebSocket失败: {}", e));
        }
    };
    
    // 创建请求消息
    let request_id = 1;
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params
    });
    
    // 发送请求
    let request_message = Message::Text(request_body.to_string());
    println!("  [调试] 发送WebSocket请求: {}", method);
    if let Err(e) = ws_stream.send(request_message).await {
        println!("  [调试] 发送WebSocket消息失败: {}", e);
        return Err(anyhow!("发送WebSocket消息失败: {}", e));
    }
    
    // 设置超时
    let timeout_duration = Duration::from_secs(15); // 增加超时时间
    let timeout = tokio::time::sleep(timeout_duration);
    
    // 等待响应或超时
    let response_future = ws_stream.next();
    
    // 使用pin_mut宏确保futures被正确pin
    pin_mut!(response_future);
    pin_mut!(timeout);
    
    // 等待响应或超时
    println!("  [调试] 等待WebSocket响应...");
    let response = match future::select(response_future, timeout).await {
        Either::Left((Some(Ok(response)), _)) => {
            println!("  [调试] 收到WebSocket响应");
            response
        },
        Either::Left((Some(Err(e)), _)) => {
            let err_msg = format!("WebSocket响应错误: {}", e);
            println!("  [调试] {}", err_msg);
            return Err(anyhow!(err_msg));
        },
        Either::Left((None, _)) => {
            let err_msg = "WebSocket连接已关闭";
            println!("  [调试] {}", err_msg);
            return Err(anyhow!(err_msg));
        },
        Either::Right((_, _)) => {
            let err_msg = "WebSocket请求超时(15秒)";
            println!("  [调试] {}", err_msg);
            return Err(anyhow!(err_msg));
        },
    };
    
    // 计算延迟
    let latency = start.elapsed().as_secs_f64() * 1000.0;
    
    // 处理响应
    let response_text = match response {
        Message::Text(text) => text,
        _ => {
            let err_msg = "收到非文本WebSocket响应";
            println!("  [调试] {}", err_msg);
            return Err(anyhow!(err_msg));
        }
    };
    
    // 解析响应
    let response_body: Value = match serde_json::from_str(&response_text) {
        Ok(body) => body,
        Err(e) => {
            let err_msg = format!("解析WebSocket响应JSON失败: {}", e);
            println!("  [调试] {}", err_msg);
            return Err(anyhow!(err_msg));
        }
    };
    
    // 检查是否成功
    let success = response_body.get("error").is_none();
    let error = if !success {
        let err_msg = response_body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .map(|s| s.to_string());
            
        if let Some(ref msg) = err_msg {
            println!("  [调试] WebSocket请求错误: {}", msg);
        }
        
        err_msg
    } else {
        None
    };
    
    // 关闭连接
    let close_msg = Message::Close(None);
    if let Err(e) = ws_stream.send(close_msg).await {
        println!("  [调试] 关闭连接失败: {}", e);
    }
    
    Ok((success, latency, error, response_body))
}

/// 测试单个 RPC 方法
async fn test_method(
    client: &Client,
    chain: &Chain,
    method: &RpcMethod,
) -> Result<RpcResult> {
    // 根据连接类型选择不同的请求处理方式
    let (success, latency_ms, error, _) = match chain.connection_type {
        ConnectionType::Http => {
            // 使用HTTP请求
            send_http_rpc_request(
                client,
                &chain.rpc_url,
                &method.name,
                &method.params,
            )
            .await?
        },
        ConnectionType::WebSocket => {
            // 使用WebSocket请求
            send_ws_rpc_request(
                &chain.rpc_url,
                &method.name,
                &method.params,
            )
            .await?
        }
    };

    Ok(RpcResult {
        chain: chain.name.clone(),
        endpoint: chain.rpc_url.clone(),
        method: method.name.clone(),
        success,
        latency_ms,
        error,
        timestamp: Utc::now(),
    })
}

/// 测试所有方法
pub async fn test_all_methods(
    chains: &[Chain],
    methods: &[RpcMethod],
    count_per_method: usize,
) -> Result<Vec<RpcResult>> {
    let client = Client::new();
    let mut all_results = Vec::new();
    
    println!("[调试] 开始测试 {} 个链上的 {} 个方法", chains.len(), methods.len());
    
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
                match test_method(&client, chain, method).await {
                    Ok(result) => {
                        method_results.push(result);
                        if attempt == 0 && chain.connection_type == ConnectionType::WebSocket {
                            println!("\n  [调试] WebSocket请求成功");
                        }
                    },
                    Err(e) => {
                        error_occurred = true;
                        last_error = e.to_string();
                        let error_msg = format!("测试失败: {}", e);
                        method_results.push(RpcResult {
                            chain: chain.name.clone(),
                            endpoint: chain.rpc_url.clone(),
                            method: method.name.clone(),
                            success: false,
                            latency_ms: 0.0,
                            error: Some(error_msg),
                            timestamp: Utc::now(),
                        });
                        
                        // 对于WebSocket连接，如果第一次请求失败，输出详细错误并终止后续尝试
                        if attempt == 0 && chain.connection_type == ConnectionType::WebSocket {
                            println!("\n  [调试] WebSocket请求失败: {}", e);
                            // 对于WebSocket，如果第一个方法失败，可能连接有问题，不再尝试其他次数
                            break;
                        }
                    }
                }
                
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
                println!("[调试] WebSocket链连接无法建立，跳过其他测试");
                break;
            }
        }
    }
    
    Ok(all_results)
}

/// 并发测试所有方法（可选，性能更好但结果可能会受到影响）
#[allow(dead_code)]
pub async fn test_all_methods_concurrent(
    chains: &[Chain],
    methods: &[RpcMethod],
    count_per_method: usize,
) -> Result<Vec<RpcResult>> {
    let client = Client::new();
    let mut all_results = Vec::new();
    
    for chain in chains {
        println!("测试链: {}", chain.name);
        
        let futures = methods.iter().map(|method| {
            let chain = chain.clone();
            let method = method.clone();
            let client = &client;
            
            async move {
                let mut method_results = Vec::with_capacity(count_per_method);
                
                for _ in 0..count_per_method {
                    match test_method(client, &chain, &method).await {
                        Ok(result) => method_results.push(result),
                        Err(e) => {
                            let error_msg = format!("测试失败: {}", e);
                            method_results.push(RpcResult {
                                chain: chain.name.clone(),
                                endpoint: chain.rpc_url.clone(),
                                method: method.name.clone(),
                                success: false,
                                latency_ms: 0.0,
                                error: Some(error_msg),
                                timestamp: Utc::now(),
                            });
                        }
                    }
                    
                    // 添加短暂延迟，避免过度请求
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                
                (method.name.clone(), method_results)
            }
        });
        
        let results = future::join_all(futures).await;
        
        for (method_name, method_results) in results {
            let success_count = method_results.iter().filter(|r| r.success).count();
            let avg_latency = if success_count > 0 {
                method_results.iter().filter(|r| r.success).map(|r| r.latency_ms).sum::<f64>() / success_count as f64
            } else {
                0.0
            };
            
            println!(
                "方法: {} 完成 ({}/{}成功, 平均: {:.2}ms)",
                method_name, success_count, count_per_method, avg_latency
            );
            
            all_results.extend(method_results);
        }
    }
    
    Ok(all_results)
} 