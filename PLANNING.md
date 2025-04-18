# ETH/BSC RPC 测试工具计划文档

## 项目概述
开发一个 Rust 脚本工具，用于批量测试以太坊（ETH）和币安智能链（BSC）RPC 的各个常用方法调用，并统计每个方法的延迟指标。

## 功能需求
1. 支持自定义 RPC URL（ETH 和 BSC）
2. 支持测试所有标准 JSON-RPC 方法
3. 多次测试每个方法并计算延迟统计数据（最小值、最大值、平均值、中位数）
4. 输出结果到 CSV 文件和控制台
5. 具备良好的可扩展性，便于添加其他链或方法

## 项目结构
```
eth-rpc-check/
├── src/
│   ├── main.rs          # 主入口
│   ├── rpc.rs           # RPC 客户端实现
│   ├── methods.rs       # 所有 RPC 方法定义
│   ├── chains.rs        # 链相关配置
│   └── stats.rs         # 统计计算
├── Cargo.toml           # 项目依赖
└── README.md            # 使用说明
```

## 依赖项
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }  # 异步运行时
reqwest = { version = "0.11", features = ["json"] }  # HTTP 客户端
serde = { version = "1.0", features = ["derive"] }  # 序列化/反序列化
serde_json = "1.0"  # JSON 处理
clap = { version = "4.4", features = ["derive"] }  # 命令行参数解析
csv = "1.2"  # CSV 文件处理
statrs = "0.16"  # 统计计算
prettytable-rs = "0.10"  # 终端表格输出
futures = "0.3"  # 异步工具
chrono = "0.4"  # 时间戳和格式化
```

## RPC 方法列表
将支持以下标准的以太坊 JSON-RPC 方法：

### 标准方法
1. `web3_clientVersion`
2. `web3_sha3`
3. `net_version`
4. `net_listening`
5. `net_peerCount`
6. `eth_protocolVersion`
7. `eth_syncing`
8. `eth_coinbase`
9. `eth_mining`
10. `eth_hashrate`
11. `eth_gasPrice`
12. `eth_accounts`
13. `eth_blockNumber`
14. `eth_getBalance`
15. `eth_getStorageAt`
16. `eth_getTransactionCount`
17. `eth_getBlockTransactionCountByHash`
18. `eth_getBlockTransactionCountByNumber`
19. `eth_getUncleCountByBlockHash`
20. `eth_getUncleCountByBlockNumber`
21. `eth_getCode`
22. `eth_sign`
23. `eth_signTransaction`
24. `eth_sendTransaction`
25. `eth_sendRawTransaction`
26. `eth_call`
27. `eth_estimateGas`
28. `eth_getBlockByHash`
29. `eth_getBlockByNumber`
30. `eth_getTransactionByHash`
31. `eth_getTransactionByBlockHashAndIndex`
32. `eth_getTransactionByBlockNumberAndIndex`
33. `eth_getTransactionReceipt`
34. `eth_getUncleByBlockHashAndIndex`
35. `eth_getUncleByBlockNumberAndIndex`
36. `eth_getLogs`
37. `eth_getWork`
38. `eth_submitWork`
39. `eth_submitHashrate`
40. `eth_chainId`
41. `eth_feeHistory`

## 实现方式

### 命令行参数
```
USAGE:
    eth-rpc-check [OPTIONS]

OPTIONS:
    -e, --eth-rpc <URL>       以太坊 RPC URL [默认: https://ethereum.publicnode.com]
    -b, --bsc-rpc <URL>       BSC RPC URL [默认: https://bsc-dataseed1.binance.org]
    -c, --count <NUM>         每个方法测试次数 [默认: 10]
    -o, --output <FILE>       CSV 输出文件路径 [默认: rpc-metrics.csv]
    -m, --methods <METHODS>   指定要测试的方法，逗号分隔
    -h, --help                打印帮助信息
```

### 数据结构
```rust
struct RpcMethod {
    name: String,
    params: Vec<serde_json::Value>,
    description: String,
}

struct RpcResult {
    chain: String,
    method: String,
    success: bool,
    latency_ms: f64,
    error: Option<String>,
}

struct MethodStats {
    chain: String,
    method: String,
    call_count: usize,
    success_count: usize,
    min_latency: f64,
    max_latency: f64,
    avg_latency: f64,
    median_latency: f64,
    p95_latency: f64,
    success_rate: f64,
}
```

### 输出格式

#### CSV 格式
```
chain,method,call_count,success_count,min_latency_ms,max_latency_ms,avg_latency_ms,median_latency_ms,p95_latency_ms,success_rate
ETH,eth_blockNumber,10,10,56.3,128.7,78.2,72.5,115.6,1.0
ETH,eth_getBalance,10,10,82.1,210.5,132.7,128.9,195.2,1.0
...
BSC,eth_blockNumber,10,10,43.2,98.1,58.9,54.3,89.2,1.0
...
```

#### 控制台输出
使用表格格式在控制台显示相同的统计数据。

### 工作流程
1. 解析命令行参数，获取 RPC URL 和测试配置
2. 初始化要测试的 RPC 方法列表
3. 对每个链（ETH、BSC）的每个方法执行指定次数的调用
4. 计算每个方法的统计指标
5. 将结果输出到 CSV 文件
6. 在控制台以表格形式显示结果摘要

### 错误处理
1. RPC 连接错误
2. 方法调用错误
3. 请求超时处理
4. 文件写入错误

## 后续扩展可能性
1. 支持更多的区块链网络（如 Polygon、Avalanche、Arbitrum 等）
2. 支持自定义 RPC 参数（如区块号、地址等）
3. 实现持续监控模式，定期执行并报告指标
4. 加入图形化报告生成
5. 支持设置断言条件，用于监控和告警系统 