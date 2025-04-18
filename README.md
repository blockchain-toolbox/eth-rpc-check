# ETH/BSC RPC 测试工具

这是一个 Rust 开发的命令行工具，用于批量测试以太坊（ETH）和币安智能链（BSC）RPC 的各个常用方法调用，并统计每个方法的延迟指标。

## 功能特点

- 支持自定义 RPC URL（ETH 和 BSC）
- 支持多种以太坊标准 JSON-RPC 方法
- 多次测试每个方法并计算详细的统计指标
- 输出结果到 CSV 文件和控制台表格
- 具备良好的可扩展性，便于添加其他链或方法

## 安装

确保已安装 Rust 和 Cargo。

```bash
# 克隆仓库
git clone https://github.com/yourusername/eth-rpc-check.git
cd eth-rpc-check

# 编译
cargo build --release

# 运行
./target/release/eth-rpc-check
```

## 使用方法

```bash
# 使用默认配置
cargo run

# 指定自定义 RPC URL
cargo run -- --eth-rpc https://ethereum.publicnode.com --bsc-rpc https://bsc-dataseed1.binance.org

# 指定测试次数
cargo run -- --count 5

# 指定输出文件
cargo run -- --output my-results.csv

# 只测试特定方法
cargo run -- --methods eth_blockNumber,eth_gasPrice,eth_chainId
```

### 命令行参数

```
OPTIONS:
    -e, --eth-rpc <URL>       以太坊 RPC URL [默认: https://ethereum.publicnode.com]
    -b, --bsc-rpc <URL>       BSC RPC URL [默认: https://bsc-dataseed1.binance.org]
    -c, --count <NUM>         每个方法测试次数 [默认: 10]
    -o, --output <FILE>       CSV 输出文件路径 [默认: rpc-metrics.csv]
    -m, --methods <METHODS>   指定要测试的方法，逗号分隔
    -h, --help                打印帮助信息
```

## 支持的 RPC 方法

工具支持以下以太坊标准 JSON-RPC 方法：

### Web3
- web3_clientVersion
- web3_sha3

### Net
- net_version
- net_listening
- net_peerCount

### ETH
- eth_protocolVersion
- eth_syncing
- eth_coinbase
- eth_mining
- eth_hashrate
- eth_gasPrice
- eth_accounts
- eth_blockNumber
- eth_chainId
- eth_getBalance
- eth_getTransactionCount
- eth_getBlockByNumber
- eth_getBlockTransactionCountByNumber
- eth_getUncleCountByBlockNumber
- eth_getCode
- eth_call
- eth_estimateGas
- eth_feeHistory
- eth_getStorageAt
- eth_getLogs

## 输出示例

### 控制台输出

```
ETH/BSC RPC 测试工具启动
ETH RPC: https://ethereum.publicnode.com
BSC RPC: https://bsc-dataseed1.binance.org
测试方法数: 25
每个方法测试次数: 10
输出文件: rpc-metrics.csv
-----------------------------
测试链: ETH
[1/25] 测试方法: web3_clientVersion ... 完成 (10/10成功, 平均: 78.32ms)
[2/25] 测试方法: eth_blockNumber ... 完成 (10/10成功, 平均: 67.81ms)
...

测试链: BSC
[1/25] 测试方法: web3_clientVersion ... 完成 (10/10成功, 平均: 58.65ms)
...

测试完成！结果已保存到: rpc-metrics.csv
```

### CSV 输出

CSV 文件将包含以下字段：

- chain: 链名称 (ETH 或 BSC)
- method: RPC 方法名称
- call_count: 调用次数
- success_count: 成功调用次数
- min_latency_ms: 最小延迟（毫秒）
- max_latency_ms: 最大延迟（毫秒）
- avg_latency_ms: 平均延迟（毫秒）
- median_latency_ms: 中位数延迟（毫秒）
- p95_latency_ms: 95 百分位延迟（毫秒）
- success_rate: 成功率

## 许可证

MIT 