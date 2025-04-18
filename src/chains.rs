/// RPC 连接类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    /// HTTP/HTTPS 连接
    Http,
    /// WebSocket 连接
    WebSocket,
}

/// Chain 表示一条区块链及其 RPC 端点
#[derive(Debug, Clone)]
pub struct Chain {
    /// 链的名称 (例如 "ETH", "BSC")
    pub name: String,
    /// RPC 端点 URL
    pub rpc_url: String,
    /// 连接类型 (HTTP 或 WebSocket)
    pub connection_type: ConnectionType,
}

impl Chain {
    /// 创建一个新的链配置
    pub fn new(name: &str, rpc_url: &str) -> Self {
        // 根据URL自动确定连接类型
        let connection_type = if rpc_url.starts_with("ws://") || rpc_url.starts_with("wss://") {
            println!("[调试] 检测到WebSocket URL: {}", rpc_url);
            ConnectionType::WebSocket
        } else {
            println!("[调试] 检测到HTTP URL: {}", rpc_url);
            ConnectionType::Http
        };

        let chain = Self {
            name: name.to_string(),
            rpc_url: rpc_url.to_string(),
            connection_type,
        };
        
        println!("[调试] 创建链 {}: {} (连接类型: {:?})", 
                 name, rpc_url, chain.connection_type);
                 
        chain
    }

    /// 创建一个新的HTTP链配置
    pub fn new_http(name: &str, rpc_url: &str) -> Self {
        Self {
            name: name.to_string(),
            rpc_url: rpc_url.to_string(),
            connection_type: ConnectionType::Http,
        }
    }

    /// 创建一个新的WebSocket链配置
    pub fn new_ws(name: &str, rpc_url: &str) -> Self {
        Self {
            name: name.to_string(),
            rpc_url: rpc_url.to_string(),
            connection_type: ConnectionType::WebSocket,
        }
    }
}