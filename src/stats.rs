use crate::rpc::RpcResult;
use anyhow::Result;
use itertools::Itertools;
use prettytable::{format, Cell, Row, Table};
use statrs::statistics::Statistics;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// 一个方法调用的统计指标
#[derive(Debug, Clone)]
pub struct MethodStats {
    /// 链名称
    pub chain: String,
    /// 端点URL
    pub endpoint: String,
    /// 方法名称
    pub method: String,
    /// 调用次数
    pub call_count: usize,
    /// 成功调用次数
    pub success_count: usize,
    /// 最小延迟（毫秒）
    pub min_latency: f64,
    /// 最大延迟（毫秒）
    pub max_latency: f64,
    /// 平均延迟（毫秒）
    pub avg_latency: f64,
    /// 中位数延迟（毫秒）
    pub median_latency: f64,
    /// 95 百分位延迟（毫秒）
    pub p95_latency: f64,
    /// 成功率
    pub success_rate: f64,
}

/// 根据 RPC 调用结果计算统计指标
pub fn calculate_stats(results: &[RpcResult]) -> Vec<MethodStats> {
    let mut stats_map: HashMap<(String, String, String), Vec<&RpcResult>> = HashMap::new();
    
    // 按链、端点和方法分组结果
    for result in results {
        let key = (result.chain.clone(), result.endpoint.clone(), result.method.clone());
        stats_map.entry(key).or_default().push(result);
    }
    
    // 计算每个组的统计指标
    stats_map
        .into_iter()
        .map(|((chain, endpoint, method), group_results)| {
            let call_count = group_results.len();
            let success_results: Vec<_> = group_results.iter().filter(|r| r.success).cloned().collect();
            let success_count = success_results.len();
            let success_rate = success_count as f64 / call_count as f64;
            
            // 如果没有成功的结果，返回全零的统计数据
            if success_count == 0 {
                return MethodStats {
                    chain,
                    endpoint,
                    method,
                    call_count,
                    success_count,
                    min_latency: 0.0,
                    max_latency: 0.0,
                    avg_latency: 0.0,
                    median_latency: 0.0,
                    p95_latency: 0.0,
                    success_rate,
                };
            }
            
            // 收集延迟数据
            let latencies: Vec<f64> = success_results
                .iter()
                .map(|r| r.latency_ms)
                .collect();
            
            // 计算统计指标
            let min_latency = latencies.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
            let max_latency = latencies.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
            let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
            
            // 排序计算中位数和百分位数
            let mut sorted_latencies = latencies.clone();
            sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median_latency = if sorted_latencies.is_empty() {
                0.0
            } else if sorted_latencies.len() % 2 == 0 {
                let mid = sorted_latencies.len() / 2;
                (sorted_latencies[mid - 1] + sorted_latencies[mid]) / 2.0
            } else {
                sorted_latencies[sorted_latencies.len() / 2]
            };
            
            let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
            let p95_latency = if sorted_latencies.is_empty() || p95_index >= sorted_latencies.len() {
                if let Some(last) = sorted_latencies.last() {
                    *last
                } else {
                    0.0
                }
            } else {
                sorted_latencies[p95_index]
            };
            
            MethodStats {
                chain,
                endpoint,
                method,
                call_count,
                success_count,
                min_latency,
                max_latency,
                avg_latency,
                median_latency,
                p95_latency,
                success_rate,
            }
        })
        .sorted_by(|a, b| {
            a.chain.cmp(&b.chain).then_with(|| {
                // 先按成功率排序（降序）
                b.success_rate.partial_cmp(&a.success_rate).unwrap_or(std::cmp::Ordering::Equal)
                // 如果成功率相同，按平均延迟排序（升序）
                .then_with(|| a.avg_latency.partial_cmp(&b.avg_latency).unwrap_or(std::cmp::Ordering::Equal))
            })
        })
        .collect()
}

/// 将统计数据写入 CSV 文件
pub fn write_to_csv(stats: &[MethodStats], output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(file);
    
    // 写入 CSV 头
    wtr.write_record(&[
        "chain",
        "endpoint",
        "method",
        "call_count",
        "success_count",
        "min_latency_ms",
        "max_latency_ms",
        "avg_latency_ms",
        "median_latency_ms",
        "p95_latency_ms",
        "success_rate_percent",
    ])?;
    
    // 写入每个方法的统计数据
    for stat in stats {
        wtr.write_record(&[
            &stat.chain,
            &stat.endpoint,
            &stat.method,
            &stat.call_count.to_string(),
            &stat.success_count.to_string(),
            &format!("{:.2}", stat.min_latency),
            &format!("{:.2}", stat.max_latency),
            &format!("{:.2}", stat.avg_latency),
            &format!("{:.2}", stat.median_latency),
            &format!("{:.2}", stat.p95_latency),
            &format!("{:.2}", stat.success_rate * 100.0),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// 在控制台中打印统计数据
pub fn print_stats(stats: &[MethodStats]) {
    // 创建并格式化表格
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    
    // 添加表头
    table.add_row(Row::new(vec![
        Cell::new("链"),
        Cell::new("方法"),
        Cell::new("调用次数"),
        Cell::new("成功次数"),
        Cell::new("成功率"),
        Cell::new("最小延迟(ms)"),
        Cell::new("最大延迟(ms)"),
        Cell::new("平均延迟(ms)"),
        Cell::new("中位数延迟(ms)"),
        Cell::new("P95延迟(ms)"),
    ]));
    
    // 分链显示
    let chain_groups = stats.iter().group_by(|s| &s.chain);
    
    for (chain, group) in &chain_groups {
        let chain_stats: Vec<_> = group.collect();
        let chain_avg_success_rate = chain_stats.iter().map(|s| s.success_rate).sum::<f64>() / chain_stats.len() as f64;
        let chain_avg_latency = chain_stats.iter().filter(|s| s.success_count > 0).map(|s| s.avg_latency).sum::<f64>() 
            / chain_stats.iter().filter(|s| s.success_count > 0).count() as f64;
        
        // 添加链的标题行
        table.add_row(Row::new(vec![
            Cell::new(&format!("== {} 总结 ==", chain)).style_spec("FgBrightCyan"),
            Cell::new(&format!("方法总数: {}", chain_stats.len())).style_spec("FgBrightCyan"),
            Cell::new("").style_spec("FgBrightCyan"),
            Cell::new("").style_spec("FgBrightCyan"),
            Cell::new(&format!("平均成功率: {:.2}%", chain_avg_success_rate * 100.0)).style_spec("FgBrightCyan"),
            Cell::new("").style_spec("FgBrightCyan"),
            Cell::new("").style_spec("FgBrightCyan"),
            Cell::new(&format!("平均延迟: {:.2}ms", chain_avg_latency)).style_spec("FgBrightCyan"),
            Cell::new("").style_spec("FgBrightCyan"),
            Cell::new("").style_spec("FgBrightCyan"),
        ]));
        
        // 添加此链的所有方法
        for stat in chain_stats {
            // 根据成功率设置颜色
            let success_rate_color = if stat.success_rate >= 0.9 {
                "Fg=Green"
            } else if stat.success_rate >= 0.5 {
                "Fg=Yellow"
            } else {
                "Fg=Red"
            };
            
            // 添加方法行
            table.add_row(Row::new(vec![
                Cell::new(&stat.chain),
                Cell::new(&stat.method),
                Cell::new(&stat.call_count.to_string()),
                Cell::new(&stat.success_count.to_string()),
                Cell::new(&format!("{:.2}%", stat.success_rate * 100.0)).style_spec(success_rate_color),
                Cell::new(&format!("{:.2}", stat.min_latency)),
                Cell::new(&format!("{:.2}", stat.max_latency)),
                Cell::new(&format!("{:.2}", stat.avg_latency)),
                Cell::new(&format!("{:.2}", stat.median_latency)),
                Cell::new(&format!("{:.2}", stat.p95_latency)),
            ]));
        }
        
        // 添加空行分隔不同链
        table.add_row(Row::new(vec![
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
        ]));
    }
    
    // 打印表格
    table.printstd();
} 