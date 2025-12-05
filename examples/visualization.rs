//! Visualization example
//!
//! This example demonstrates both terminal and HTML visualization outputs.
//! It creates several concurrent tasks with different patterns to show
//! how async-inspect can visualize complex async workflows.

use async_inspect::prelude::*;
use async_inspect::runtime::tokio::spawn_tracked;
use colored::Colorize;
use std::time::Duration;
use tokio::time::sleep;

/// Simulates fetching user data
async fn fetch_user_data(user_id: u32) -> String {
    sleep(Duration::from_millis(100 + (user_id * 20) as u64)).await;
    format!("User {} data", user_id)
}

/// Simulates processing data with variable duration
async fn process_data(data: String, complexity: u32) -> String {
    sleep(Duration::from_millis(50 + (complexity * 15) as u64)).await;
    format!("Processed: {}", data)
}

/// Simulates saving to database
async fn save_to_db(_data: String) {
    sleep(Duration::from_millis(80)).await;
}

/// Complex workflow with multiple concurrent operations
async fn complex_workflow() {
    println!("🚀 Starting complex workflow with concurrent tasks...\n");

    // Scenario 1: Parallel data fetching
    let fetch_tasks: Vec<_> = (1..=5)
        .map(|id| {
            spawn_tracked(format!("fetch_user_{}", id), async move {
                let data = fetch_user_data(id).await;
                inspect_point!("data_fetched", format!("Fetched: {}", data));
                data
            })
        })
        .collect();

    // Scenario 2: Sequential processing pipeline
    let pipeline_task = spawn_tracked("pipeline", async {
        let data1 = fetch_user_data(10).await;
        inspect_point!("stage_1_complete");

        let processed = process_data(data1, 3).await;
        inspect_point!("stage_2_complete");

        save_to_db(processed).await;
        inspect_point!("stage_3_complete");
    });

    // Scenario 3: Race condition - first to finish wins
    let race_task = spawn_tracked("race_winner", async {
        tokio::select! {
            _ = async {
                sleep(Duration::from_millis(150)).await;
                "Fast path"
            } => {
                inspect_point!("race_result", "Fast path won!");
            }
            _ = async {
                sleep(Duration::from_millis(200)).await;
                "Slow path"
            } => {
                inspect_point!("race_result", "Slow path won!");
            }
        }
    });

    // Scenario 4: Retry logic
    let retry_task = spawn_tracked("retry_logic", async {
        for attempt in 1..=3 {
            inspect_point!("retry_attempt", format!("Attempt {}", attempt));

            sleep(Duration::from_millis(40)).await;

            if attempt == 3 {
                inspect_point!("retry_success", "Operation succeeded!");
                break;
            }
        }
    });

    // Scenario 5: Timeout handling
    let timeout_task = spawn_tracked("timeout_handler", async {
        let result = tokio::time::timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(150)).await;
            "Completed"
        })
        .await;

        match result {
            Ok(_) => inspect_point!("timeout_result", "Completed in time"),
            Err(_) => inspect_point!("timeout_result", "Timed out!"),
        }
    });

    // Wait for all tasks
    for task in fetch_tasks {
        let _ = task.await;
    }
    let _ = pipeline_task.await;
    let _ = race_task.await;
    let _ = retry_task.await;
    let _ = timeout_task.await;

    // Small delay to ensure all events are recorded
    sleep(Duration::from_millis(50)).await;
}

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!(
        "║  {} - Visualization Example                    ║",
        "[async-inspect]".on_purple().white().bold()
    );
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Run the complex workflow
    complex_workflow().await;

    println!(
        "\n{}\n",
        "[OK] Workflow completed!".on_green().white().bold()
    );

    // Get the reporter
    let reporter = Reporter::global();

    // Print terminal visualizations
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ 1. Summary Report                                          │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_summary();

    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ 2. Gantt Timeline (Terminal)                               │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_gantt_timeline();

    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ 3. Event Timeline                                          │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_timeline();

    // Generate HTML report
    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ 4. Generating HTML Report                                  │");
    println!("└────────────────────────────────────────────────────────────┘");

    let html_reporter = HtmlReporter::global();
    let html_path = "async_inspect_report.html";

    match html_reporter.save_to_file(html_path) {
        Ok(_) => {
            println!(
                "\n{} HTML report saved to: {}",
                "[OK]".on_green().white().bold(),
                html_path
            );
            println!(
                "\n{} {}",
                "[WEB]".on_truecolor(139, 69, 19).white().bold(),
                "Open it in your browser to explore:".bright_white()
            );
            println!(
                "   {} {}",
                "-".bright_cyan(),
                "Interactive timeline with hover details".bright_white()
            );
            println!(
                "   {} {}",
                "-".bright_cyan(),
                "Click on tasks to see full event history".bright_white()
            );
            println!(
                "   {} {}",
                "-".bright_cyan(),
                "Visual state machine flow".bright_white()
            );
            println!(
                "   {} {}",
                "-".bright_cyan(),
                "Expandable task details".bright_white()
            );
            let url = format!(
                "file://{}/{}",
                std::env::current_dir().unwrap().display(),
                html_path
            );
            println!(
                "\n   {} {}",
                "Open:".bright_yellow().bold(),
                url.bright_blue().underline()
            );
        }
        Err(e) => {
            println!(
                "\n{} Failed to save HTML report: {}",
                "[FAIL]".on_red().white().bold(),
                e
            );
        }
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Summary                                                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let stats = Inspector::global().stats();
    println!(
        "{} Analyzed {} tasks with {} events",
        "[STATS]".on_blue().white().bold(),
        stats.total_tasks,
        stats.total_events
    );
    println!(
        "{} Total execution time: {:.2}s",
        "[TIME]".on_truecolor(204, 119, 34).white().bold(),
        stats.timeline_duration.as_secs_f64()
    );
    println!(
        "{} Completed: {}",
        "[OK]".on_green().white().bold(),
        stats.completed_tasks
    );
    println!(
        "{} Failed: {}",
        "[FAIL]".on_red().white().bold(),
        stats.failed_tasks
    );

    println!("\n{}", "[>] Next steps:".bright_cyan().bold());
    println!(
        "   {} {}",
        "1.".bright_yellow(),
        "Open the HTML report in your browser".bright_white()
    );
    println!(
        "   {} {}",
        "2.".bright_yellow(),
        "Examine the Gantt timeline above".bright_white()
    );
    println!(
        "   {} {}",
        "3.".bright_yellow(),
        "Look for bottlenecks and optimization opportunities".bright_white()
    );
    println!(
        "   {} {}",
        "4.".bright_yellow(),
        "Use [async-inspect] in your own async code!".bright_white()
    );

    println!();
}
