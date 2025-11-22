//! Performance report generation

use super::Profiler;
use std::fmt::Write as FmtWrite;

/// Performance report generator
pub struct PerformanceReporter<'a> {
    profiler: &'a Profiler,
}

impl<'a> PerformanceReporter<'a> {
    /// Create a new performance reporter
    pub fn new(profiler: &'a Profiler) -> Self {
        Self { profiler }
    }

    /// Print a comprehensive performance report
    pub fn print_report(&self) {
        self.print_header();
        self.print_overall_stats();
        self.print_bottlenecks();
        self.print_hot_paths();
        self.print_slowest_tasks();
        self.print_await_stats();
        self.print_efficiency_analysis();
    }

    /// Print report header
    fn print_header(&self) {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║           async-inspect - Performance Report              ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
    }

    /// Print overall statistics
    fn print_overall_stats(&self) {
        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Overall Statistics                                         │");
        println!("└────────────────────────────────────────────────────────────┘");

        let stats = self.profiler.calculate_stats();
        let all_metrics = self.profiler.all_metrics();

        println!("  Total Tasks:     {}", all_metrics.len());
        println!(
            "  Completed:       {}",
            all_metrics.iter().filter(|m| m.completed).count()
        );
        println!();
        println!("  Duration Stats:");
        println!(
            "    Min:           {:.2}ms",
            stats.min.as_secs_f64() * 1000.0
        );
        println!(
            "    Max:           {:.2}ms",
            stats.max.as_secs_f64() * 1000.0
        );
        println!(
            "    Mean:          {:.2}ms",
            stats.mean.as_secs_f64() * 1000.0
        );
        println!(
            "    Median (p50):  {:.2}ms",
            stats.median.as_secs_f64() * 1000.0
        );
        println!(
            "    p95:           {:.2}ms",
            stats.p95.as_secs_f64() * 1000.0
        );
        println!(
            "    p99:           {:.2}ms",
            stats.p99.as_secs_f64() * 1000.0
        );
        println!("    Std Dev:       {:.2}ms", stats.std_dev * 1000.0);
        println!();
    }

    /// Print bottleneck analysis
    fn print_bottlenecks(&self) {
        let bottlenecks = self.profiler.identify_bottlenecks();

        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Bottleneck Analysis                                        │");
        println!("└────────────────────────────────────────────────────────────┘");

        if bottlenecks.is_empty() {
            println!("  ✅ No bottlenecks detected\n");
            return;
        }

        println!(
            "  ⚠️  Found {} potential bottleneck(s):\n",
            bottlenecks.len()
        );

        for (i, metrics) in bottlenecks.iter().enumerate().take(10) {
            println!(
                "  {}. {} (#{}) - {:.2}ms",
                i + 1,
                metrics.name,
                metrics.task_id.as_u64(),
                metrics.total_duration.as_secs_f64() * 1000.0
            );
            println!(
                "     Running: {:.2}ms | Blocked: {:.2}ms | Efficiency: {:.1}%",
                metrics.running_time.as_secs_f64() * 1000.0,
                metrics.blocked_time.as_secs_f64() * 1000.0,
                metrics.efficiency() * 100.0
            );
        }
        println!();
    }

    /// Print hot path analysis
    fn print_hot_paths(&self) {
        let hot_paths = self.profiler.get_hot_paths();

        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Hot Paths (Most Frequently Executed)                      │");
        println!("└────────────────────────────────────────────────────────────┘");

        if hot_paths.is_empty() {
            println!("  No hot paths identified\n");
            return;
        }

        println!("  Top execution paths:\n");

        for (i, path) in hot_paths.iter().enumerate().take(10) {
            println!("  {}. {} ", i + 1, path.path);
            println!(
                "     Executions: {} | Total: {:.2}ms | Avg: {:.2}ms",
                path.execution_count,
                path.total_time.as_secs_f64() * 1000.0,
                path.avg_time.as_secs_f64() * 1000.0
            );
        }
        println!();
    }

    /// Print slowest tasks
    fn print_slowest_tasks(&self) {
        let slowest = self.profiler.slowest_tasks(10);

        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Slowest Tasks                                              │");
        println!("└────────────────────────────────────────────────────────────┘");

        if slowest.is_empty() {
            println!("  No tasks to analyze\n");
            return;
        }

        for (i, metrics) in slowest.iter().enumerate() {
            println!(
                "  {}. {} (#{}) - {:.2}ms",
                i + 1,
                metrics.name,
                metrics.task_id.as_u64(),
                metrics.total_duration.as_secs_f64() * 1000.0
            );
            println!(
                "     Polls: {} | Awaits: {} | Avg poll: {:.2}ms",
                metrics.poll_count,
                metrics.await_count,
                metrics.avg_poll_duration.as_secs_f64() * 1000.0
            );
        }
        println!();
    }

    /// Print await point statistics
    fn print_await_stats(&self) {
        let stats = self.profiler.await_stats();

        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Await Point Analysis                                       │");
        println!("└────────────────────────────────────────────────────────────┘");

        if stats.count == 0 {
            println!("  No await points recorded\n");
            return;
        }

        println!("  Total Await Points: {}", stats.count);
        println!();
        println!("  Await Duration Stats:");
        println!(
            "    Min:           {:.2}ms",
            stats.min.as_secs_f64() * 1000.0
        );
        println!(
            "    Max:           {:.2}ms",
            stats.max.as_secs_f64() * 1000.0
        );
        println!(
            "    Mean:          {:.2}ms",
            stats.mean.as_secs_f64() * 1000.0
        );
        println!(
            "    Median (p50):  {:.2}ms",
            stats.median.as_secs_f64() * 1000.0
        );
        println!(
            "    p95:           {:.2}ms",
            stats.p95.as_secs_f64() * 1000.0
        );
        println!(
            "    p99:           {:.2}ms",
            stats.p99.as_secs_f64() * 1000.0
        );
        println!();
    }

    /// Print efficiency analysis
    fn print_efficiency_analysis(&self) {
        let least_efficient = self.profiler.least_efficient_tasks(5);

        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Efficiency Analysis (Least Efficient Tasks)               │");
        println!("└────────────────────────────────────────────────────────────┘");

        if least_efficient.is_empty() {
            println!("  No tasks to analyze\n");
            return;
        }

        println!("  Tasks with highest blocked time ratio:\n");

        for (i, metrics) in least_efficient.iter().enumerate() {
            let efficiency_pct = metrics.efficiency() * 100.0;
            let blocked_pct =
                (metrics.blocked_time.as_secs_f64() / metrics.total_duration.as_secs_f64()) * 100.0;

            println!(
                "  {}. {} (#{}) - {:.1}% efficient",
                i + 1,
                metrics.name,
                metrics.task_id.as_u64(),
                efficiency_pct
            );
            println!(
                "     Total: {:.2}ms | Running: {:.2}ms ({:.1}%) | Blocked: {:.2}ms ({:.1}%)",
                metrics.total_duration.as_secs_f64() * 1000.0,
                metrics.running_time.as_secs_f64() * 1000.0,
                efficiency_pct,
                metrics.blocked_time.as_secs_f64() * 1000.0,
                blocked_pct
            );
        }
        println!();
    }

    /// Generate a compact performance summary
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        let stats = self.profiler.calculate_stats();
        let bottlenecks = self.profiler.identify_bottlenecks();

        writeln!(summary, "Performance Summary:").unwrap();
        writeln!(summary, "  Tasks: {}", self.profiler.all_metrics().len()).unwrap();
        writeln!(
            summary,
            "  Mean duration: {:.2}ms",
            stats.mean.as_secs_f64() * 1000.0
        )
        .unwrap();
        writeln!(
            summary,
            "  p95 duration: {:.2}ms",
            stats.p95.as_secs_f64() * 1000.0
        )
        .unwrap();
        writeln!(summary, "  Bottlenecks: {}", bottlenecks.len()).unwrap();

        summary
    }

    /// Print recommendations based on profiling data
    pub fn print_recommendations(&self) {
        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Optimization Recommendations                               │");
        println!("└────────────────────────────────────────────────────────────┘");

        let bottlenecks = self.profiler.identify_bottlenecks();
        let least_efficient = self.profiler.least_efficient_tasks(3);
        let busiest = self.profiler.busiest_tasks(3);

        let mut recommendations = Vec::new();

        if !bottlenecks.is_empty() {
            recommendations.push(format!(
                "⚠️  {} bottleneck(s) detected - consider optimizing slow tasks",
                bottlenecks.len()
            ));
        }

        if !least_efficient.is_empty() {
            let avg_efficiency: f64 = least_efficient.iter().map(|m| m.efficiency()).sum::<f64>()
                / least_efficient.len() as f64;

            if avg_efficiency < 0.5 {
                recommendations.push(
                    "⚡ Low efficiency detected - tasks spending too much time blocked".to_string(),
                );
                recommendations.push(
                    "   → Consider reducing await dependencies or using timeouts".to_string(),
                );
            }
        }

        if !busiest.is_empty() {
            let max_polls = busiest[0].poll_count;
            if max_polls > 100 {
                recommendations.push(format!(
                    "🔄 Task with {} polls detected - possible busy loop or fine-grained awaits",
                    max_polls
                ));
                recommendations.push(
                    "   → Consider batching operations or using coarser-grained awaits".to_string(),
                );
            }
        }

        let hot_paths = self.profiler.get_hot_paths();
        if let Some(hottest) = hot_paths.first() {
            if hottest.execution_count > 100 {
                recommendations.push(format!(
                    "🔥 Hot path detected: '{}' executed {} times",
                    hottest.path, hottest.execution_count
                ));
                recommendations
                    .push("   → Consider caching or memoization if appropriate".to_string());
            }
        }

        if recommendations.is_empty() {
            println!("  ✅ No major performance issues detected!");
            println!("  ✨ Your async code looks well-optimized.");
        } else {
            for rec in recommendations {
                println!("  {}", rec);
            }
        }

        println!();
    }
}
