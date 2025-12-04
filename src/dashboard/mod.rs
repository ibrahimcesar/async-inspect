//! Web dashboard for real-time async inspection
//!
//! This module provides a browser-based dashboard for monitoring async tasks in real-time.
//! It uses WebSockets to stream events and metrics to connected clients.
//!
//! # Features
//!
//! - Real-time task monitoring
//! - Interactive timeline visualization
//! - Live metrics dashboard
//! - Event log with filtering
//! - RESTful API fallback
//!
//! # Example
//!
//! ```no_run
//! use async_inspect::dashboard::Dashboard;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start dashboard on port 8080
//!     let dashboard = Dashboard::new(8080);
//!     let handle = dashboard.start().await?;
//!
//!     println!("Dashboard running at http://localhost:8080");
//!
//!     // Keep running
//!     handle.await??;
//!     Ok(())
//! }
//! ```

#[cfg(feature = "dashboard")]
use crate::inspector::Inspector;

#[cfg(feature = "dashboard")]
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State as AxumState, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
#[cfg(feature = "dashboard")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "dashboard")]
use std::net::SocketAddr;
#[cfg(feature = "dashboard")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(feature = "dashboard")]
use tokio::sync::broadcast;
#[cfg(feature = "dashboard")]
use tokio::task::JoinHandle;
#[cfg(feature = "dashboard")]
use tower_http::cors::CorsLayer;

/// Dashboard event sent to connected clients
#[cfg(feature = "dashboard")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardEvent {
    /// Task was spawned
    TaskSpawned {
        /// Task ID
        task_id: u64,
        /// Task name
        name: String,
        /// Parent task ID
        parent: Option<u64>,
        /// Timestamp in milliseconds
        timestamp: u128,
    },
    /// Task completed successfully
    TaskCompleted {
        /// Task ID
        task_id: u64,
        /// Duration in milliseconds
        duration_ms: f64,
        /// Timestamp
        timestamp: u128,
    },
    /// Task failed
    TaskFailed {
        /// Task ID
        task_id: u64,
        /// Error message
        error: Option<String>,
        /// Timestamp
        timestamp: u128,
    },
    /// Task state changed
    StateChanged {
        /// Task ID
        task_id: u64,
        /// Old state
        old_state: String,
        /// New state
        new_state: String,
        /// Timestamp
        timestamp: u128,
    },
    /// Metrics snapshot
    MetricsSnapshot {
        /// Total tasks
        total_tasks: usize,
        /// Running tasks
        running_tasks: usize,
        /// Completed tasks
        completed_tasks: usize,
        /// Failed tasks
        failed_tasks: usize,
        /// Blocked tasks
        blocked_tasks: usize,
        /// Timestamp
        timestamp: u128,
    },
    /// Await point started
    AwaitStarted {
        /// Task ID
        task_id: u64,
        /// Await point label
        label: String,
        /// Timestamp
        timestamp: u128,
    },
    /// Await point ended
    AwaitEnded {
        /// Task ID
        task_id: u64,
        /// Await point label
        label: String,
        /// Duration in milliseconds
        duration_ms: f64,
        /// Timestamp
        timestamp: u128,
    },
}

/// Dashboard state shared across handlers
#[cfg(feature = "dashboard")]
#[derive(Clone)]
struct DashboardState {
    /// Event broadcaster
    event_tx: broadcast::Sender<DashboardEvent>,
    /// Inspector reference
    inspector: &'static Inspector,
}

/// Web dashboard for real-time monitoring
#[cfg(feature = "dashboard")]
pub struct Dashboard {
    /// Server port
    port: u16,
    /// Event broadcaster
    event_tx: broadcast::Sender<DashboardEvent>,
}

#[cfg(feature = "dashboard")]
impl Dashboard {
    /// Create a new dashboard on the specified port
    #[must_use]
    pub fn new(port: u16) -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self { port, event_tx }
    }

    /// Start the dashboard server
    ///
    /// Returns a join handle for the server task
    pub async fn start(self) -> Result<JoinHandle<Result<(), std::io::Error>>, std::io::Error> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let inspector = Inspector::global();

        let state = DashboardState {
            event_tx: self.event_tx.clone(),
            inspector,
        };

        // Start metrics broadcaster
        let metrics_tx = self.event_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                interval.tick().await;

                let stats = inspector.stats();
                let snapshot = DashboardEvent::MetricsSnapshot {
                    total_tasks: stats.total_tasks,
                    running_tasks: stats.running_tasks,
                    completed_tasks: stats.completed_tasks,
                    failed_tasks: stats.failed_tasks,
                    blocked_tasks: stats.blocked_tasks,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis(),
                };

                let _ = metrics_tx.send(snapshot);
            }
        });

        // Build router
        let app = Router::new()
            .route("/", get(serve_dashboard))
            .route("/ws", get(websocket_handler))
            .route("/api/tasks", get(api_tasks))
            .route("/api/stats", get(api_stats))
            .layer(CorsLayer::permissive())
            .with_state(state);

        // Spawn server
        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await
        });

        Ok(handle)
    }
}

/// Serve the dashboard HTML
#[cfg(feature = "dashboard")]
async fn serve_dashboard() -> Html<&'static str> {
    Html(include_str!("static/index.html"))
}

/// WebSocket upgrade handler
#[cfg(feature = "dashboard")]
async fn websocket_handler(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<DashboardState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
#[cfg(feature = "dashboard")]
async fn handle_websocket(mut socket: WebSocket, state: DashboardState) {
    let mut event_rx = state.event_tx.subscribe();

    // Send initial state
    let tasks = state.inspector.get_all_tasks();
    for task in tasks {
        let event = DashboardEvent::TaskSpawned {
            task_id: task.id.as_u64(),
            name: task.name.clone(),
            parent: task.parent.map(|p| p.as_u64()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        };

        if let Ok(json) = serde_json::to_string(&event) {
            if socket.send(Message::Text(json)).await.is_err() {
                return;
            }
        }
    }

    // Stream events
    while let Ok(event) = event_rx.recv().await {
        if let Ok(json) = serde_json::to_string(&event) {
            if socket.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    }
}

/// REST API: Get all tasks
#[cfg(feature = "dashboard")]
async fn api_tasks(AxumState(state): AxumState<DashboardState>) -> Json<serde_json::Value> {
    let tasks = state.inspector.get_all_tasks();

    let task_list: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|task| {
            serde_json::json!({
                "id": task.id.as_u64(),
                "name": task.name,
                "state": format!("{:?}", task.state),
                "parent": task.parent.map(|p| p.as_u64()),
                "poll_count": task.poll_count,
            })
        })
        .collect();

    Json(serde_json::json!({ "tasks": task_list }))
}

/// REST API: Get statistics
#[cfg(feature = "dashboard")]
async fn api_stats(AxumState(state): AxumState<DashboardState>) -> Json<serde_json::Value> {
    let stats = state.inspector.stats();

    Json(serde_json::json!({
        "total_tasks": stats.total_tasks,
        "running_tasks": stats.running_tasks,
        "completed_tasks": stats.completed_tasks,
        "failed_tasks": stats.failed_tasks,
        "blocked_tasks": stats.blocked_tasks,
    }))
}

#[cfg(not(feature = "dashboard"))]
compile_error!("The dashboard module requires the 'dashboard' feature to be enabled");
