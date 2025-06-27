mod chains;
mod methods;
mod rpc;
mod stats;

use anyhow::Result;
use clap::Parser;
use log::{info, warn, error};
use std::path::PathBuf;
use url::Url;

#[derive(Parser, Debug)]
#[clap(
    name = "eth-rpc-check",
    about = "测试以太坊和BSC RPC方法的延迟",
    version
)]
struct Cli {
    /// 以太坊 RPC URL (HTTP/HTTPS)
    #[clap(short = 'e', long, default_value = "https://ethereum.publicnode.com")]
    eth_rpc: String,

    /// BSC RPC URL (HTTP/HTTPS)
    #[clap(short = 'b', long, default_value = "https://bsc-dataseed1.binance.org")]
    bsc_rpc: String,
    
    /// 以太坊 WebSocket URL (WS/WSS)
    #[clap(long)]
    eth_ws: Option<String>,

    /// BSC WebSocket URL (WS/WSS)
    #[clap(long)]
    bsc_ws: Option<String>,

    /// 每个方法测试次数 (1-100)
    #[clap(short = 'c', long, default_value = "10")]
    count: usize,

    /// CSV 输出文件路径
    #[clap(short = 'o', long, default_value = "rpc-metrics.csv")]
    output: PathBuf,

    /// 指定要测试的方法，用逗号分隔
    #[clap(short = 'm', long)]
    methods: Option<String>,

    /// 使用基础测试方法集 (快速测试)
    #[clap(long, conflicts_with = "methods")]
    basic: bool,

    /// 使用扩展测试方法集 (更全面的测试)
    #[clap(long, conflicts_with = "methods")]
    extended: bool,

    /// 日志级别 (error, warn, info, debug)
    #[clap(long, default_value = "info")]
    log_level: String,
}

/// 验证命令行参数
fn validate_args(cli: &Cli) -> Result<()> {
    // 验证测试次数
    if cli.count == 0 || cli.count > 100 {
        anyhow::bail!("测试次数必须在1-100之间，当前值: {}", cli.count);
    }

    // 验证URL格式
    let urls_to_check = vec![
        ("ETH RPC", &cli.eth_rpc),
        ("BSC RPC", &cli.bsc_rpc),
    ];
    
    for (name, url) in urls_to_check {
        if let Err(e) = Url::parse(url) {
            anyhow::bail!("{} URL格式无效: {} (错误: {})", name, url, e);
        }
    }

    // 验证可选的WebSocket URL
    if let Some(ref eth_ws) = cli.eth_ws {
        if let Err(e) = Url::parse(eth_ws) {
            anyhow::bail!("ETH WebSocket URL格式无效: {} (错误: {})", eth_ws, e);
        }
        if !eth_ws.starts_with("ws://") && !eth_ws.starts_with("wss://") {
            anyhow::bail!("ETH WebSocket URL必须以ws://或wss://开头: {}", eth_ws);
        }
    }

    if let Some(ref bsc_ws) = cli.bsc_ws {
        if let Err(e) = Url::parse(bsc_ws) {
            anyhow::bail!("BSC WebSocket URL格式无效: {} (错误: {})", bsc_ws, e);
        }
        if !bsc_ws.starts_with("ws://") && !bsc_ws.starts_with("wss://") {
            anyhow::bail!("BSC WebSocket URL必须以ws://或wss://开头: {}", bsc_ws);
        }
    }

    // 验证输出目录存在
    if let Some(parent) = cli.output.parent() {
        if !parent.exists() {
            anyhow::bail!("输出目录不存在: {}", parent.display());
        }
    }

    Ok(())
}

/// 初始化日志系统
fn init_logger(log_level: &str) -> Result<()> {
    let level = match log_level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        _ => {
            anyhow::bail!("无效的日志级别: {}。支持的级别: error, warn, info, debug", log_level);
        }
    };

    env_logger::Builder::from_default_env()
        .filter_level(level)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();
    
    // 初始化日志系统
    init_logger(&cli.log_level)?;
    
    // 验证参数
    validate_args(&cli)?;
    
    // 初始化方法列表
    let method_list = if let Some(methods_str) = cli.methods {
        let filtered_methods = methods::get_filtered_methods(&methods_str);
        if filtered_methods.is_empty() {
            error!("没有找到匹配的方法: {}", methods_str);
            anyhow::bail!("没有有效的测试方法");
        }
        info!("使用自定义方法列表: {} 个方法", filtered_methods.len());
        filtered_methods
    } else if cli.basic {
        info!("使用基础测试方法集 (快速测试)");
        methods::get_basic_methods()
    } else if cli.extended {
        info!("使用扩展测试方法集 (全面测试)");
        methods::get_extended_methods()
    } else {
        info!("使用完整方法列表");
        methods::get_all_methods()
    };
    
    // 打印启动信息
    println!("ETH/BSC RPC 测试工具启动");
    println!("ETH HTTP RPC: {}", cli.eth_rpc);
    println!("BSC HTTP RPC: {}", cli.bsc_rpc);
    
    if let Some(eth_ws) = &cli.eth_ws {
        println!("ETH WebSocket: {}", eth_ws);
    }
    
    if let Some(bsc_ws) = &cli.bsc_ws {
        println!("BSC WebSocket: {}", bsc_ws);
    }
    
    println!("测试方法数: {}", method_list.len());
    println!("每个方法测试次数: {}", cli.count);
    println!("输出文件: {}", cli.output.display());
    println!("-----------------------------");
    
    // 创建链配置
    let mut chains = vec![
        chains::Chain::new("ETH-HTTP", &cli.eth_rpc),
        chains::Chain::new("BSC-HTTP", &cli.bsc_rpc),
    ];
    
    // 添加WebSocket链配置
    if let Some(eth_ws) = &cli.eth_ws {
        chains.push(chains::Chain::new("ETH-WS", eth_ws));
    }
    
    if let Some(bsc_ws) = &cli.bsc_ws {
        chains.push(chains::Chain::new("BSC-WS", bsc_ws));
    }
    
    info!("开始执行RPC测试");
    
    // 执行测试
    let results = rpc::test_all_methods(&chains, &method_list, cli.count).await?;
    
    // 计算统计数据
    let stats = stats::calculate_stats(&results);
    
    // 输出到CSV
    stats::write_to_csv(&stats, &cli.output)?;
    
    // 控制台输出
    stats::print_stats(&stats);
    
    println!("\n测试完成！结果已保存到: {}", cli.output.display());
    
    Ok(())
}
