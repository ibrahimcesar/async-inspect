//! async-inspect CLI
//!
//! Command-line interface for inspecting and monitoring async Rust applications.

use async_inspect::config::Config;
use async_inspect::export::{CsvExporter, JsonExporter};
use async_inspect::inspector::Inspector;
use async_inspect::reporter::Reporter;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[cfg(feature = "cli")]
use async_inspect::tui::run_tui;

/// async-inspect - X-ray vision for async Rust
#[derive(Parser, Debug)]
#[command(name = "async-inspect")]
#[command(author, version)]
#[command(about = "[async-inspect] X-ray vision for async Rust")]
#[command(long_about = None)]
#[command(arg_required_else_help = true)]
#[command(
    after_help = "[INFO] For detailed information, run: async-inspect info\n[TIP] Quick start guide, examples, and documentation available with 'info' command"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch interactive TUI monitor
    #[cfg(feature = "cli")]
    Monitor {
        /// Update interval in milliseconds
        #[arg(short, long, default_value = "100")]
        interval: u64,
    },

    /// Export task data to various formats
    Export {
        /// Output format
        #[arg(short, long, value_enum)]
        format: ExportFormat,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Export events separately (CSV only)
        #[arg(long)]
        with_events: bool,
    },

    /// Show current statistics
    Stats {
        /// Show detailed performance metrics
        #[arg(short, long)]
        detailed: bool,
    },

    /// Configure production settings
    Config {
        /// Configuration mode
        #[arg(value_enum)]
        mode: ConfigMode,

        /// Custom sampling rate (1 in N tasks)
        #[arg(short, long)]
        sampling_rate: Option<usize>,

        /// Maximum events to retain
        #[arg(short = 'e', long)]
        max_events: Option<usize>,

        /// Maximum tasks to track
        #[arg(short = 't', long)]
        max_tasks: Option<usize>,
    },

    /// Show configuration and overhead information
    Info,

    /// Show version information
    Version,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ExportFormat {
    /// Export as JSON
    Json,
    /// Export as CSV
    Csv,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ConfigMode {
    /// Production mode (1% sampling, minimal tracking)
    Production,
    /// Development mode (full tracking)
    Development,
    /// Debug mode (unlimited tracking)
    Debug,
    /// Custom configuration
    Custom,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!(
            "{} - Verbose mode enabled\n",
            "[async-inspect]".on_purple().white().bold()
        );
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // This shouldn't happen due to arg_required_else_help, but handle it anyway
            eprintln!("No command specified. Use --help to see available commands.");
            std::process::exit(1);
        }
    };

    match command {
        #[cfg(feature = "cli")]
        Commands::Monitor { interval } => {
            println!("╔════════════════════════════════════════════════════════════╗");
            println!(
                "║  {} - TUI Monitor                               ║",
                "[async-inspect]".on_purple().white().bold()
            );
            println!("╚════════════════════════════════════════════════════════════╝\n");
            println!("[>] Launching TUI (update interval: {}ms)...\n", interval);

            let inspector = Inspector::global().clone();

            // Note: The TUI will display any tasks that get registered.
            // In a library context, tasks are tracked when using #[async_inspect::trace]
            // or spawn_tracked() in your application code.

            run_tui(inspector)?;

            println!("\n[OK] Monitor closed.");
            Ok(())
        }

        Commands::Export {
            format,
            output,
            with_events,
        } => {
            let inspector = Inspector::global();
            let stats = inspector.stats();

            if stats.total_tasks == 0 {
                println!("[WARN] No tasks tracked yet. Use #[async_inspect::trace] in your code.");
                return Ok(());
            }

            println!(
                "{} Exporting {} tasks and {} events...",
                "[*]".on_yellow().white().bold(),
                stats.total_tasks,
                stats.total_events
            );

            match format {
                ExportFormat::Json => {
                    JsonExporter::export_to_file(inspector, &output)?;
                    println!("[OK] Exported to JSON: {}", output.display());
                }
                ExportFormat::Csv => {
                    CsvExporter::export_tasks_to_file(inspector, &output)?;
                    println!("[OK] Exported tasks to CSV: {}", output.display());

                    if with_events {
                        let mut events_path = output.clone();
                        events_path.set_file_name(format!(
                            "{}_events.csv",
                            output.file_stem().unwrap().to_string_lossy()
                        ));
                        CsvExporter::export_events_to_file(inspector, &events_path)?;
                        println!("[OK] Exported events to CSV: {}", events_path.display());
                    }
                }
            }

            Ok(())
        }

        Commands::Stats { detailed } => {
            let inspector = Inspector::global();
            let reporter = Reporter::global();
            let stats = inspector.stats();

            if stats.total_tasks == 0 {
                println!("[WARN] No tasks tracked yet. Use #[async_inspect::trace] in your code.");
                return Ok(());
            }

            println!("╔════════════════════════════════════════════════════════════╗");
            println!(
                "║  {} - Statistics                                ║",
                "[async-inspect]".on_purple().white().bold()
            );
            println!("╚════════════════════════════════════════════════════════════╝\n");

            reporter.print_summary();

            if detailed {
                println!("\n[STATS] Performance Metrics\n");
                let profiler = inspector.build_profiler();
                let perf_reporter = async_inspect::profile::PerformanceReporter::new(&profiler);
                perf_reporter.print_report();
            }

            Ok(())
        }

        Commands::Config {
            mode,
            sampling_rate,
            max_events,
            max_tasks,
        } => {
            let config = Config::global();

            println!(
                "{} {}\n",
                "[CONFIG]".on_cyan().white().bold(),
                "Configuring [async-inspect]...".bright_white()
            );

            match mode {
                ConfigMode::Production => {
                    config.production_mode();
                    println!("[OK] Applied production mode:");
                    println!("   • 1% sampling (1 in 100 tasks)");
                    println!("   • 1,000 event limit");
                    println!("   • 500 task limit");
                    println!("   • Await tracking disabled");
                    println!("   • HTML reports disabled");
                }
                ConfigMode::Development => {
                    config.development_mode();
                    println!("[OK] Applied development mode:");
                    println!("   • Full sampling (all tasks)");
                    println!("   • 10,000 event limit");
                    println!("   • 1,000 task limit");
                    println!("   • Await tracking enabled");
                    println!("   • HTML reports enabled");
                }
                ConfigMode::Debug => {
                    config.debug_mode();
                    println!("[OK] Applied debug mode:");
                    println!("   • Full sampling (all tasks)");
                    println!("   • Unlimited events");
                    println!("   • Unlimited tasks");
                    println!("   • Await tracking enabled");
                    println!("   • HTML reports enabled");
                }
                ConfigMode::Custom => {
                    if let Some(rate) = sampling_rate {
                        config.set_sampling_rate(rate);
                        println!("[OK] Set sampling rate: 1 in {}", rate);
                    }
                    if let Some(events) = max_events {
                        config.set_max_events(events);
                        println!("[OK] Set max events: {}", events);
                    }
                    if let Some(tasks) = max_tasks {
                        config.set_max_tasks(tasks);
                        println!("[OK] Set max tasks: {}", tasks);
                    }
                    println!("\n[OK] Applied custom configuration");
                }
            }

            println!("\n[CONFIG] Current Configuration:");
            print_config(config);

            Ok(())
        }

        Commands::Info => {
            let config = Config::global();
            let inspector = Inspector::global();
            let stats = inspector.stats();

            println!("╔════════════════════════════════════════════════════════════╗");
            println!(
                "║  {} - Information                               ║",
                "[async-inspect]".on_purple().white().bold()
            );
            println!("╚════════════════════════════════════════════════════════════╝\n");

            println!(
                "{} Version: {}",
                "[*]".on_yellow().white().bold(),
                env!("CARGO_PKG_VERSION")
            );
            println!(
                "{} Description: {}\n",
                "[*]".on_yellow().white().bold(),
                env!("CARGO_PKG_DESCRIPTION")
            );

            println!("[CONFIG] Configuration:");
            print_config(config);

            println!("\n[STATS] Current State:");
            println!("  Total tasks:     {}", stats.total_tasks);
            println!("  Running tasks:   {}", stats.running_tasks);
            println!("  Completed tasks: {}", stats.completed_tasks);
            println!("  Failed tasks:    {}", stats.failed_tasks);
            println!("  Total events:    {}", stats.total_events);
            println!(
                "  Duration:        {:.2}s",
                stats.timeline_duration.as_secs_f64()
            );

            let overhead = config.overhead_stats();
            if overhead.calls > 0 {
                println!("\n[PERF] Overhead Statistics:");
                println!("  Total overhead:        {:.2}ms", overhead.total_ms());
                println!("  Instrumentation calls: {}", overhead.calls);
                println!("  Average per call:      {:.2}µs", overhead.avg_us());
            }

            println!("\n[INFO] Features:");
            println!("  • Task tracking and inspection");
            println!("  • Automatic instrumentation (#[async_inspect::trace])");
            println!("  • Deadlock detection");
            println!("  • Performance profiling");
            #[cfg(feature = "cli")]
            println!("  • Real-time TUI monitoring");
            println!("  • JSON/CSV export");
            println!("  • Production-ready configuration");

            println!("\n{}", "[*] Links:".on_yellow().white().bold());
            println!(
                "  {} {}",
                "Homepage:     ".bright_white(),
                env!("CARGO_PKG_HOMEPAGE").bright_blue()
            );
            println!(
                "  {} {}",
                "Repository:   ".bright_white(),
                env!("CARGO_PKG_REPOSITORY").bright_blue()
            );
            println!(
                "  {} {}",
                "Documentation:".bright_white(),
                "https://docs.rs/async-inspect".bright_blue()
            );

            println!("\n{}", "[*] Quick Start:".on_yellow().white().bold());
            println!("  1. Add to your Cargo.toml:");
            println!("     async-inspect = \"{}\"", env!("CARGO_PKG_VERSION"));
            println!("\n  2. Annotate async functions:");
            println!("     #[async_inspect::trace]");
            println!("     async fn my_function() {{ ... }}");
            println!("\n  3. Launch TUI monitor:");
            println!("     async-inspect monitor");
            println!("\n  4. Export data:");
            println!("     async-inspect export -f json -o trace.json");

            Ok(())
        }

        Commands::Version => {
            println!(
                "{} {}",
                "[async-inspect]".on_purple().white().bold(),
                env!("CARGO_PKG_VERSION")
            );
            println!("X-ray vision for async Rust\n");

            println!("Features enabled:");
            #[cfg(feature = "cli")]
            println!("  • cli (TUI support)");
            #[cfg(feature = "tokio")]
            println!("  • tokio");

            println!("\nAuthors: {}", env!("CARGO_PKG_AUTHORS"));
            println!("License: {}", env!("CARGO_PKG_LICENSE"));

            Ok(())
        }
    }
}

fn print_config(config: &Config) {
    println!("  Sampling rate:   1 in {}", config.sampling_rate());
    println!(
        "  Max events:      {}",
        if config.max_events() == 0 {
            "unlimited".to_string()
        } else {
            config.max_events().to_string()
        }
    );
    println!(
        "  Max tasks:       {}",
        if config.max_tasks() == 0 {
            "unlimited".to_string()
        } else {
            config.max_tasks().to_string()
        }
    );
    println!("  Track awaits:    {}", config.track_awaits());
    println!("  Track polls:     {}", config.track_polls());
    println!("  Enable HTML:     {}", config.enable_html());
}
