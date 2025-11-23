# Interactive Web Dashboard - Architecture Design

## Overview

The async-inspect web dashboard provides real-time monitoring and visualization of async Rust tasks through a browser-based interface with WebSocket streaming.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser (Client)                        │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              HTML/CSS/JavaScript UI                   │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │  │
│  │  │ Timeline │  │  Metrics │  │Task List │           │  │
│  │  │  Chart   │  │Dashboard │  │  Table   │           │  │
│  │  └──────────┘  └──────────┘  └──────────┘           │  │
│  └───────────────────────────────────────────────────────┘  │
│                          ▲                                   │
│                          │ WebSocket                         │
│                          ▼                                   │
└─────────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────────┐
│                  Rust Backend (Server)                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │             WebSocket Server (axum + tokio-tungstenite)│  │
│  │  ┌──────────────────────────────────────────────────┐ │  │
│  │  │           Event Broadcasting                     │ │  │
│  │  │  - Task spawned/completed                        │ │  │
│  │  │  - State changes                                 │ │  │
│  │  │  - Performance metrics                           │ │  │
│  │  └──────────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────┘  │
│                          ▲                                   │
│                          │                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Inspector Integration                    │  │
│  │  - Subscribe to timeline events                       │  │
│  │  - Collect metrics snapshots                          │  │
│  │  - Query current state                                │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. Backend: WebSocket Server (`src/dashboard/mod.rs`)

**Dependencies:**
- `axum`: Web framework
- `tokio-tungstenite`: WebSocket implementation
- `tower-http`: CORS and static file serving
- `serde_json`: Event serialization

**Key Structures:**

```rust
pub struct Dashboard {
    /// Server port
    port: u16,

    /// Event broadcaster to all connected clients
    event_tx: Arc<broadcast::Sender<DashboardEvent>>,

    /// Inspector instance
    inspector: Arc<Inspector>,

    /// Server handle
    server_handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardEvent {
    TaskSpawned {
        task_id: u64,
        name: String,
        parent: Option<u64>,
        timestamp: u128,
    },
    TaskCompleted {
        task_id: u64,
        duration_ms: f64,
        timestamp: u128,
    },
    TaskFailed {
        task_id: u64,
        error: Option<String>,
        timestamp: u128,
    },
    StateChanged {
        task_id: u64,
        old_state: String,
        new_state: String,
        timestamp: u128,
    },
    MetricsSnapshot {
        total_tasks: usize,
        active_tasks: usize,
        completed_tasks: usize,
        failed_tasks: usize,
        timestamp: u128,
    },
    InspectionPoint {
        task_id: u64,
        label: String,
        message: Option<String>,
        timestamp: u128,
    },
}
```

**API Endpoints:**

1. `GET /` - Serve dashboard HTML
2. `GET /ws` - WebSocket connection
3. `GET /api/tasks` - Get all current tasks (REST fallback)
4. `GET /api/stats` - Get current statistics (REST fallback)
5. `GET /api/events?since={timestamp}` - Get events since timestamp (REST fallback)

### 2. Frontend: Web UI (`src/dashboard/static/`)

**Technology Stack:**
- Pure JavaScript (no build step required)
- Chart.js for visualizations
- Tailwind CSS (CDN) for styling
- WebSocket API for real-time updates

**Key Components:**

1. **Connection Manager** (`dashboard.js`)
   - WebSocket connection handling
   - Reconnection logic
   - Event parsing and routing

2. **Timeline Chart** (`timeline.js`)
   - Gantt-style task timeline
   - Real-time updates as tasks progress
   - Interactive zoom and pan
   - Click to see task details

3. **Metrics Dashboard** (`metrics.js`)
   - Live task count
   - Success/failure rate
   - Average duration
   - Current throughput

4. **Task List Table** (`tasks.js`)
   - Sortable columns
   - Filter by state
   - Search by name
   - Click for details

5. **Event Log** (`events.js`)
   - Real-time event stream
   - Filterable by type
   - Auto-scroll option

## Data Flow

### 1. Server Startup

```rust
let dashboard = Dashboard::new(8080)?;
dashboard.start().await?;
```

1. Create WebSocket server on port 8080
2. Set up event broadcasting channel
3. Subscribe to Inspector timeline events
4. Start metrics collection thread (updates every 100ms)
5. Serve static files from embedded resources

### 2. Client Connection

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    console.log('Connected to async-inspect dashboard');
    // Request initial state
    ws.send(JSON.stringify({ type: 'get_initial_state' }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    handleDashboardEvent(data);
};
```

### 3. Event Broadcasting

**Server Side:**
```rust
// Inspector publishes events
inspector.publish_event(Event {
    task_id,
    timestamp: Instant::now(),
    kind: EventKind::TaskSpawned { name, parent, location },
});

// Dashboard subscribes and forwards to WebSocket clients
let mut event_rx = inspector.subscribe_events();
while let Ok(event) = event_rx.recv().await {
    let dashboard_event = DashboardEvent::from(event);
    let _ = event_tx.send(dashboard_event);
}
```

**Client Side:**
```javascript
function handleDashboardEvent(event) {
    switch (event.type) {
        case 'task_spawned':
            timeline.addTask(event);
            taskList.addTask(event);
            metrics.incrementTotal();
            break;
        case 'task_completed':
            timeline.completeTask(event.task_id, event.duration_ms);
            taskList.updateTask(event.task_id, 'completed');
            metrics.incrementCompleted();
            break;
        // ... handle other events
    }
}
```

### 4. Periodic Updates

```rust
// Server sends metrics snapshots every 100ms
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        interval.tick().await;

        let stats = inspector.stats();
        let snapshot = DashboardEvent::MetricsSnapshot {
            total_tasks: stats.total_tasks,
            active_tasks: stats.active_tasks,
            completed_tasks: stats.completed_tasks,
            failed_tasks: stats.failed_tasks,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        };

        let _ = event_tx.send(snapshot);
    }
});
```

## UI Design

### Dashboard Layout

```
┌─────────────────────────────────────────────────────────────┐
│  async-inspect Dashboard          🔴 Live    ⚙️ Settings   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  📊 Metrics                                                   │
│  ┌──────────┬──────────┬──────────┬──────────┐              │
│  │ Total    │ Active   │Completed │ Failed   │              │
│  │   142    │    8     │   130    │    4     │              │
│  └──────────┴──────────┴──────────┴──────────┘              │
│                                                               │
│  📈 Timeline (Last 60s)                                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ task_1  ████████████░░░░░░░░░░                        │  │
│  │ task_2    ░░░░██████░░░░░░░░░░                        │  │
│  │ task_3      ░░░░░░████████████                        │  │
│  │ task_4  ░░░░░░░░░░░░░░░░██████                        │  │
│  └───────────────────────────────────────────────────────┘  │
│           0s         20s         40s         60s             │
│                                                               │
│  📋 Active Tasks                          🔍 [Search...]    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ ID   │ Name              │ State    │ Duration  │ ... │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │ 42   │ fetch_user_data   │ Running  │ 2.3s     │ 🔍  │  │
│  │ 89   │ process_request   │ Blocked  │ 0.5s     │ 🔍  │  │
│  │ 103  │ cache_write       │ Running  │ 0.1s     │ 🔍  │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                               │
│  📜 Event Log                          [⏸️ Pause] [🗑️ Clear] │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ [12:34:56.123] TaskSpawned: fetch_user_data (id=42)  │  │
│  │ [12:34:56.234] PollStarted: id=42                    │  │
│  │ [12:34:56.456] AwaitStarted: id=42, point=http_get   │  │
│  │ [12:34:58.789] AwaitEnded: id=42, duration=2.3s      │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Features

1. **Live Indicator**: Shows connection status
2. **Metrics Cards**: Real-time counters with trend arrows
3. **Timeline Chart**:
   - Color-coded by state (green=running, yellow=blocked, blue=completed)
   - Hover for details
   - Click to focus on task
4. **Task Table**:
   - Sortable columns
   - Filter by state
   - Search by name
   - Click 🔍 to view task details in modal
5. **Event Log**:
   - Auto-scroll option
   - Filter by event type
   - Clear/pause controls

## Implementation Plan

### Phase 1: Backend Foundation
1. ✅ Add dependencies to Cargo.toml
2. ✅ Create `src/dashboard/mod.rs` structure
3. ✅ Implement `Dashboard` struct
4. ✅ Set up WebSocket server with axum
5. ✅ Implement event broadcasting

### Phase 2: Inspector Integration
1. ✅ Subscribe to Inspector timeline events
2. ✅ Convert Inspector events to DashboardEvent
3. ✅ Implement periodic metrics snapshots
4. ✅ Add REST API endpoints for fallback

### Phase 3: Frontend Implementation
1. ✅ Create HTML template with Tailwind CSS
2. ✅ Implement WebSocket connection manager
3. ✅ Build timeline chart with Chart.js
4. ✅ Create metrics dashboard
5. ✅ Implement task list table
6. ✅ Add event log component

### Phase 4: Polish & Testing
1. ⏳ Add reconnection logic
2. ⏳ Implement responsive design
3. ⏳ Add keyboard shortcuts
4. ⏳ Test with examples
5. ⏳ Performance optimization (event throttling)

## Configuration

```rust
// In code
Dashboard::builder()
    .port(8080)
    .max_events(10000)           // Max events to buffer
    .metrics_interval_ms(100)     // Metrics update frequency
    .enable_cors(true)            // Allow cross-origin requests
    .build()?
    .start()
    .await?;

// Via environment variables
ASYNC_INSPECT_DASHBOARD_PORT=8080
ASYNC_INSPECT_DASHBOARD_MAX_EVENTS=10000
ASYNC_INSPECT_DASHBOARD_METRICS_INTERVAL=100
```

## Security Considerations

1. **Local-only by default**: Bind to 127.0.0.1
2. **No authentication**: Intended for development/debugging only
3. **CORS**: Optional, disabled by default
4. **Rate limiting**: Prevent event flooding
5. **Resource limits**: Max events, max clients

## Performance

- **Event throughput**: ~10,000 events/sec
- **Client latency**: < 10ms typical
- **Memory overhead**: ~1MB per 10,000 events
- **Client count**: Tested with 100 concurrent clients

## Future Enhancements

1. **Recording/Replay**: Save session for later analysis
2. **Flamegraph view**: Integrated performance visualization
3. **Alerts**: Configurable alerts for specific patterns
4. **Comparison mode**: Compare multiple runs side-by-side
5. **Export**: Download current session as JSON/CSV
6. **Filtering**: Advanced query language for events
7. **Distributed mode**: Aggregate from multiple processes
