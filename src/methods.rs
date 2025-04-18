use serde_json::json;

/// 表示一个 RPC 方法及其参数
#[derive(Debug, Clone)]
pub struct RpcMethod {
    /// 方法名称 (例如 "eth_blockNumber")
    pub name: String,
    /// 方法参数列表
    pub params: Vec<serde_json::Value>,
    /// 方法描述
    pub description: String,
}

impl RpcMethod {
    /// 创建一个新的 RPC 方法
    pub fn new(name: &str, params: Vec<serde_json::Value>, description: &str) -> Self {
        Self {
            name: name.to_string(),
            params,
            description: description.to_string(),
        }
    }
}

/// 获取所有支持的 RPC 方法
pub fn get_all_methods() -> Vec<RpcMethod> {
    let mut methods = Vec::new();

    // Web3 方法
    methods.push(RpcMethod::new(
        "web3_clientVersion",
        vec![],
        "获取客户端版本",
    ));
    methods.push(RpcMethod::new(
        "web3_sha3",
        vec![json!("0x68656c6c6f20776f726c64")],
        "计算 Keccak-256 哈希",
    ));

    // Net 方法
    methods.push(RpcMethod::new(
        "net_version",
        vec![],
        "获取网络 ID",
    ));
    methods.push(RpcMethod::new(
        "net_listening",
        vec![],
        "检查节点是否正在监听网络连接",
    ));
    methods.push(RpcMethod::new(
        "net_peerCount",
        vec![],
        "获取已连接的对等节点数量",
    ));

    // 标准 ETH 方法
    methods.push(RpcMethod::new(
        "eth_protocolVersion",
        vec![],
        "获取以太坊协议版本",
    ));
    methods.push(RpcMethod::new(
        "eth_syncing",
        vec![],
        "检查节点是否正在同步",
    ));
    methods.push(RpcMethod::new(
        "eth_coinbase",
        vec![],
        "获取节点挖矿账户",
    ));
    methods.push(RpcMethod::new(
        "eth_mining",
        vec![],
        "检查节点是否正在挖矿",
    ));
    methods.push(RpcMethod::new(
        "eth_hashrate",
        vec![],
        "获取节点挖矿哈希率",
    ));
    methods.push(RpcMethod::new(
        "eth_gasPrice",
        vec![],
        "获取当前 gas 价格",
    ));
    methods.push(RpcMethod::new(
        "eth_accounts",
        vec![],
        "获取节点控制的账户列表",
    ));
    methods.push(RpcMethod::new(
        "eth_blockNumber",
        vec![],
        "获取最新区块号",
    ));
    methods.push(RpcMethod::new(
        "eth_chainId",
        vec![],
        "获取链 ID",
    ));

    // 需要参数的方法
    methods.push(RpcMethod::new(
        "eth_getBalance",
        vec![json!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"), json!("latest")],
        "获取账户余额 (Vitalik 的地址)",
    ));
    methods.push(RpcMethod::new(
        "eth_getTransactionCount",
        vec![json!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"), json!("latest")],
        "获取账户交易数量 (Vitalik 的地址)",
    ));
    methods.push(RpcMethod::new(
        "eth_getBlockByNumber",
        vec![json!("latest"), json!(false)],
        "获取最新区块信息",
    ));
    methods.push(RpcMethod::new(
        "eth_getBlockTransactionCountByNumber",
        vec![json!("latest")],
        "获取最新区块的交易数量",
    ));
    methods.push(RpcMethod::new(
        "eth_getUncleCountByBlockNumber",
        vec![json!("latest")],
        "获取最新区块的叔块数量",
    ));
    methods.push(RpcMethod::new(
        "eth_getCode",
        vec![json!("0x6b175474e89094c44da98b954eedeac495271d0f"), json!("latest")],
        "获取合约代码 (DAI 合约)",
    ));
    methods.push(RpcMethod::new(
        "eth_call",
        vec![
            json!({
                "to": "0x6b175474e89094c44da98b954eedeac495271d0f",
                "data": "0x06fdde03"  // name()
            }),
            json!("latest")
        ],
        "调用合约方法 (DAI 合约的 name 方法)",
    ));
    methods.push(RpcMethod::new(
        "eth_estimateGas",
        vec![
            json!({
                "to": "0x6b175474e89094c44da98b954eedeac495271d0f",
                "data": "0x06fdde03"  // name()
            })
        ],
        "估计调用合约方法的 gas 消耗",
    ));
    methods.push(RpcMethod::new(
        "eth_feeHistory",
        vec![json!("0x1"), json!("latest"), json!([25, 50, 75])],
        "获取近期 gas 费用历史",
    ));

    // 不太可能使用的方法（只是为了完整性而添加）
    methods.push(RpcMethod::new(
        "eth_getStorageAt",
        vec![json!("0x6b175474e89094c44da98b954eedeac495271d0f"), json!("0x0"), json!("latest")],
        "获取存储位置的值",
    ));
    methods.push(RpcMethod::new(
        "eth_getLogs",
        vec![
            json!({
                "fromBlock": "latest",
                "toBlock": "latest",
                "address": "0x6b175474e89094c44da98b954eedeac495271d0f"
            })
        ],
        "获取指定区块范围和地址的日志",
    ));

    methods
}

/// 根据逗号分隔的方法名字符串获取过滤后的方法列表
pub fn get_filtered_methods(methods_str: &str) -> Vec<RpcMethod> {
    let method_names: Vec<&str> = methods_str.split(',').map(|s| s.trim()).collect();
    let all_methods = get_all_methods();
    
    all_methods
        .into_iter()
        .filter(|method| method_names.contains(&method.name.as_str()))
        .collect()
} 