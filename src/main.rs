mod chains;
mod methods;
mod rpc;
mod stats;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

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

    /// 每个方法测试次数
    #[clap(short = 'c', long, default_value = "10")]
    count: usize,

    /// CSV 输出文件路径
    #[clap(short = 'o', long, default_value = "rpc-metrics.csv")]
    output: PathBuf,

    /// 指定要测试的方法，用逗号分隔
    #[clap(short = 'm', long)]
    methods: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();
    
    // 初始化方法列表
    let method_list = if let Some(methods_str) = cli.methods {
        methods::get_filtered_methods(&methods_str)
    } else {
        methods::get_all_methods()
    };
    
    // 打印所有参数，用于调试
    println!("命令行参数调试信息:");
    println!("  ETH HTTP RPC: {}", cli.eth_rpc);
    println!("  BSC HTTP RPC: {}", cli.bsc_rpc);
    println!("  ETH WebSocket: {:?}", cli.eth_ws);
    println!("  BSC WebSocket: {:?}", cli.bsc_ws);
    
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
