use serde_json::json;

/// 测试用的常见地址常量
pub mod test_addresses {
    /// Vitalik Buterin的以太坊地址
    pub const VITALIK_ADDRESS: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
    /// DAI稳定币合约地址
    pub const DAI_CONTRACT: &str = "0x6b175474e89094c44da98b954eedeac495271d0f";
    /// USDC稳定币合约地址
    pub const USDC_CONTRACT: &str = "0xa0b86a33e6c5bb4d9b0c44a8b5ec23c9b02a1a8a";
}

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
    methods.extend(get_web3_methods());
    
    // Net 方法
    methods.extend(get_net_methods());
    
    // ETH 基础方法
    methods.extend(get_eth_basic_methods());
    
    // ETH 查询方法
    methods.extend(get_eth_query_methods());
    
    // ETH 高级方法
    methods.extend(get_eth_advanced_methods());

    methods
}

/// 获取Web3相关方法
fn get_web3_methods() -> Vec<RpcMethod> {
    vec![
        RpcMethod::new(
            "web3_clientVersion",
            vec![],
            "获取客户端版本信息",
        ),
        RpcMethod::new(
            "web3_sha3",
            vec![json!("0x68656c6c6f20776f726c64")], // "hello world"的十六进制
            "计算输入数据的Keccak-256哈希值",
        ),
    ]
}

/// 获取网络相关方法
fn get_net_methods() -> Vec<RpcMethod> {
    vec![
        RpcMethod::new(
            "net_version",
            vec![],
            "获取当前网络ID",
        ),
        RpcMethod::new(
            "net_listening",
            vec![],
            "检查节点是否正在监听网络连接",
        ),
        RpcMethod::new(
            "net_peerCount",
            vec![],
            "获取已连接的对等节点数量",
        ),
    ]
}

/// 获取以太坊基础方法
fn get_eth_basic_methods() -> Vec<RpcMethod> {
    vec![
        RpcMethod::new(
            "eth_protocolVersion",
            vec![],
            "获取以太坊协议版本",
        ),
        RpcMethod::new(
            "eth_syncing",
            vec![],
            "检查节点同步状态",
        ),
        RpcMethod::new(
            "eth_coinbase",
            vec![],
            "获取节点的挖矿收益地址",
        ),
        RpcMethod::new(
            "eth_mining",
            vec![],
            "检查节点是否正在挖矿",
        ),
        RpcMethod::new(
            "eth_hashrate",
            vec![],
            "获取节点的挖矿哈希率",
        ),
        RpcMethod::new(
            "eth_gasPrice",
            vec![],
            "获取当前推荐的gas价格",
        ),
        RpcMethod::new(
            "eth_accounts",
            vec![],
            "获取节点控制的账户列表",
        ),
        RpcMethod::new(
            "eth_blockNumber",
            vec![],
            "获取最新区块号",
        ),
        RpcMethod::new(
            "eth_chainId",
            vec![],
            "获取链ID",
        ),
    ]
}

/// 获取以太坊查询方法
fn get_eth_query_methods() -> Vec<RpcMethod> {
    use test_addresses::*;
    
    vec![
        RpcMethod::new(
            "eth_getBalance",
            vec![json!(VITALIK_ADDRESS), json!("latest")],
            "获取账户余额 (使用Vitalik的地址作为测试)",
        ),
        RpcMethod::new(
            "eth_getTransactionCount",
            vec![json!(VITALIK_ADDRESS), json!("latest")],
            "获取账户的交易数量 (nonce)",
        ),
        RpcMethod::new(
            "eth_getBlockByNumber",
            vec![json!("latest"), json!(false)],
            "获取最新区块信息 (不包含完整交易)",
        ),
        RpcMethod::new(
            "eth_getBlockTransactionCountByNumber",
            vec![json!("latest")],
            "获取最新区块的交易数量",
        ),
        RpcMethod::new(
            "eth_getUncleCountByBlockNumber",
            vec![json!("latest")],
            "获取最新区块的叔块数量 (ETH专用)",
        ),
        RpcMethod::new(
            "eth_getCode",
            vec![json!(DAI_CONTRACT), json!("latest")],
            "获取合约字节码 (使用DAI合约作为测试)",
        ),
        RpcMethod::new(
            "eth_getStorageAt",
            vec![json!(DAI_CONTRACT), json!("0x0"), json!("latest")],
            "读取合约存储位置的值",
        ),
    ]
}

/// 获取以太坊高级方法
fn get_eth_advanced_methods() -> Vec<RpcMethod> {
    use test_addresses::*;
    
    vec![
        RpcMethod::new(
            "eth_call",
            vec![
                json!({
                    "to": DAI_CONTRACT,
                    "data": "0x06fdde03"  // name() 函数签名
                }),
                json!("latest")
            ],
            "调用合约只读方法 (获取DAI代币名称)",
        ),
        RpcMethod::new(
            "eth_estimateGas",
            vec![
                json!({
                    "to": DAI_CONTRACT,
                    "data": "0x06fdde03"  // name() 函数签名
                })
            ],
            "估算调用合约方法所需的gas",
        ),
        RpcMethod::new(
            "eth_feeHistory",
            vec![json!("0x1"), json!("latest"), json!([25, 50, 75])],
            "获取最近区块的fee历史 (EIP-1559相关)",
        ),
        RpcMethod::new(
            "eth_getLogs",
            vec![
                json!({
                    "fromBlock": "latest",
                    "toBlock": "latest",
                    "address": DAI_CONTRACT,
                    "topics": []
                })
            ],
            "获取事件日志 (限制为最新区块以减少负载)",
        ),
    ]
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

/// 获取基础性能测试方法 (快速测试)
pub fn get_basic_methods() -> Vec<RpcMethod> {
    vec![
        RpcMethod::new("eth_blockNumber", vec![], "获取最新区块号"),
        RpcMethod::new("eth_gasPrice", vec![], "获取当前gas价格"),
        RpcMethod::new("eth_chainId", vec![], "获取链ID"),
        RpcMethod::new("net_version", vec![], "获取网络ID"),
        RpcMethod::new("web3_clientVersion", vec![], "获取客户端版本"),
    ]
}

/// 获取扩展测试方法 (包含更多复杂查询)
pub fn get_extended_methods() -> Vec<RpcMethod> {
    let mut methods = get_basic_methods();
    methods.extend(get_eth_query_methods());
    methods.extend(vec![
        RpcMethod::new("eth_syncing", vec![], "检查同步状态"),
        RpcMethod::new("net_listening", vec![], "检查网络监听状态"),
    ]);
    methods
} 