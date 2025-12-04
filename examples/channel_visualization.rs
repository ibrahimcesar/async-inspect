//! Channel Visualization Example
//!
//! Demonstrates how async-inspect's tracked channels automatically
//! record message flow metrics without any manual instrumentation.

use async_inspect::channel::{broadcast, mpsc, oneshot};
use colored::Colorize;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!(
        "\n{}\n",
        " [async-inspect] Channel Visualization Demo "
            .white()
            .bold()
            .on_purple()
    );

    // Demo 1: MPSC channel tracking
    demo_mpsc().await;

    // Demo 2: Oneshot channel tracking
    demo_oneshot().await;

    // Demo 3: Broadcast channel tracking
    demo_broadcast().await;

    // Demo 4: Complex pipeline
    demo_pipeline().await;

    println!(
        "\n{}\n",
        " [async-inspect] Demo Complete! "
            .white()
            .bold()
            .on_purple()
    );
}

async fn demo_mpsc() {
    println!("\n{} MPSC Channel Tracking\n", "[*]".cyan().bold());

    let (tx, mut rx) = mpsc::channel::<i32>(10, "work_queue");

    // Producer task
    let producer = tokio::spawn({
        let tx = tx.clone();
        async move {
            for i in 0..10 {
                tx.send(i).await.unwrap();
                println!("    Producer sent: {}", i);
            }
        }
    });

    // Consumer task with slight delay
    let consumer = tokio::spawn(async move {
        while let Some(value) = rx.recv().await {
            println!("    Consumer received: {}", value);
            sleep(Duration::from_millis(5)).await;
        }
        rx.metrics()
    });

    producer.await.unwrap();
    drop(tx); // Close channel

    let metrics = consumer.await.unwrap();

    println!("\n{} MPSC Channel Metrics:", "[STATS]".green().bold());
    println!("    Messages sent:     {}", metrics.sent);
    println!("    Messages received: {}", metrics.received);
    println!(
        "    Send block rate:   {:.1}%",
        metrics.send_block_rate() * 100.0
    );
    println!(
        "    Recv block rate:   {:.1}%",
        metrics.recv_block_rate() * 100.0
    );
    println!("    Total recv wait:   {:?}", metrics.total_recv_wait_time);
}

async fn demo_oneshot() {
    println!("\n{} Oneshot Channel Tracking\n", "[*]".cyan().bold());

    // Multiple oneshot channels for request-response pattern
    let mut handles = vec![];

    for i in 0..5 {
        let (tx, rx) = oneshot::channel::<String>(format!("request_{}", i));

        // Simulate async request-response
        handles.push(tokio::spawn(async move {
            sleep(Duration::from_millis(10 * i as u64)).await;
            tx.send(format!("Response #{}", i)).unwrap();
        }));

        handles.push(tokio::spawn(async move {
            let response = rx.await.unwrap();
            println!("    Received: {}", response);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    println!(
        "\n{} Oneshot channels completed (5 request-response pairs)",
        "[OK]".green().bold()
    );
}

async fn demo_broadcast() {
    println!("\n{} Broadcast Channel Tracking\n", "[*]".cyan().bold());

    let (tx, mut rx1) = broadcast::channel::<String>(16, "events");
    let mut rx2 = tx.subscribe();
    let mut rx3 = tx.subscribe();

    println!(
        "    Created broadcast channel with {} receivers",
        tx.receiver_count()
    );

    // Send events
    tx.send("event_1".into()).unwrap();
    tx.send("event_2".into()).unwrap();
    tx.send("event_3".into()).unwrap();

    // Receive on all subscribers
    let r1 = tokio::spawn(async move {
        let mut count = 0;
        while let Ok(msg) = rx1.recv().await {
            println!("    Subscriber 1 received: {}", msg);
            count += 1;
            if count >= 3 {
                break;
            }
        }
    });

    let r2 = tokio::spawn(async move {
        let mut count = 0;
        while let Ok(msg) = rx2.recv().await {
            println!("    Subscriber 2 received: {}", msg);
            count += 1;
            if count >= 3 {
                break;
            }
        }
    });

    let r3 = tokio::spawn(async move {
        let mut count = 0;
        while let Ok(msg) = rx3.recv().await {
            println!("    Subscriber 3 received: {}", msg);
            count += 1;
            if count >= 3 {
                break;
            }
        }
    });

    r1.await.unwrap();
    r2.await.unwrap();
    r3.await.unwrap();

    let metrics = tx.metrics();
    println!("\n{} Broadcast Metrics:", "[STATS]".green().bold());
    println!("    Messages broadcast: {}", metrics.sent);
    println!(
        "    Total deliveries:   {} (3 msgs x 3 receivers)",
        metrics.sent * 3
    );
}

async fn demo_pipeline() {
    println!(
        "\n{} Pipeline Pattern (Producer -> Processor -> Consumer)\n",
        "[*]".cyan().bold()
    );

    // Stage 1: Producer -> Processor channel
    let (producer_tx, mut processor_rx) = mpsc::channel::<i32>(5, "raw_data");

    // Stage 2: Processor -> Consumer channel
    let (processor_tx, mut consumer_rx) = mpsc::channel::<i32>(5, "processed_data");

    // Producer: generates raw data
    let producer = tokio::spawn(async move {
        for i in 1..=10 {
            producer_tx.send(i).await.unwrap();
            println!("    [Producer] Generated: {}", i);
        }
        producer_tx.metrics()
    });

    // Processor: transforms data (doubles it)
    let processor = tokio::spawn(async move {
        while let Some(value) = processor_rx.recv().await {
            let processed = value * 2;
            processor_tx.send(processed).await.unwrap();
            println!("    [Processor] {} -> {}", value, processed);
        }
        (processor_rx.metrics(), processor_tx.metrics())
    });

    // Consumer: receives processed data
    let consumer = tokio::spawn(async move {
        let mut sum = 0;
        while let Some(value) = consumer_rx.recv().await {
            println!("    [Consumer] Received: {}", value);
            sum += value;
        }
        (sum, consumer_rx.metrics())
    });

    let producer_metrics = producer.await.unwrap();
    // processor_rx is moved into the processor task, no need to drop here

    let (stage1_rx_metrics, stage2_tx_metrics) = processor.await.unwrap();
    let (total_sum, consumer_metrics) = consumer.await.unwrap();

    println!("\n{} Pipeline Metrics:", "[STATS]".green().bold());
    println!("  Stage 1 (raw_data):");
    println!(
        "    Sent: {}, Received: {}",
        producer_metrics.sent, stage1_rx_metrics.received
    );
    println!("  Stage 2 (processed_data):");
    println!(
        "    Sent: {}, Received: {}",
        stage2_tx_metrics.sent, consumer_metrics.received
    );
    println!(
        "\n{} Final sum of processed values: {}",
        "[OK]".green().bold(),
        total_sum
    );
}
