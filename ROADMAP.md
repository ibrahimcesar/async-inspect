# async-inspect Development Roadmap

## 📊 Current Status: Phase 2 Complete! ✅

**Version:** 0.1.0-alpha
**Last Updated:** 2025-11-20

---

## ✅ Completed Phases

### **Phase 1: Foundation & Core Infrastructure** ✅ (100%)

**Duration:** Initial development
**Status:** COMPLETE

**Achievements:**
- ✅ Core task data structures (TaskId, TaskInfo, TaskState)
- ✅ Comprehensive event system (8 event types)
- ✅ Thread-safe Inspector singleton
- ✅ Manual instrumentation macros
- ✅ Beautiful terminal reporting
- ✅ 16 unit tests passing
- ✅ Working examples

**Files Created:**
- `src/task/mod.rs` (195 lines)
- `src/timeline/mod.rs` (253 lines)
- `src/inspector/mod.rs` (322 lines)
- `src/instrument/mod.rs` (247 lines)
- `src/reporter/mod.rs` (221 lines)
- `examples/simple_test.rs`

**Key Features:**
```rust
use async_inspect::prelude::*;

async fn my_task() {
    let _guard = TaskGuard::new("my_task");
    inspect_point!("checkpoint");
}

// Generate reports
Reporter::global().print_summary();
```

---

### **Phase 2: Tokio Runtime Integration** ✅ (100%)

**Duration:** Current session
**Status:** COMPLETE

**Achievements:**
- ✅ `spawn_tracked()` function - automatic task spawning
- ✅ `TrackedFuture<F>` wrapper - automatic poll tracking
- ✅ `InspectExt` trait - `.inspect()` syntax
- ✅ `spawn_local_tracked()` for !Send futures
- ✅ 4 new integration tests (20 total tests passing)
- ✅ Full Tokio integration example
- ✅ Zero overhead when disabled

**Files Created:**
- `src/runtime/mod.rs`
- `src/runtime/tokio.rs` (265 lines)
- `examples/tokio_integration.rs`

**Key Features:**
```rust
use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};

// Option 1: Drop-in spawn replacement
spawn_tracked("task_name", async {
    // Automatically tracked!
});

// Option 2: Extension method
let result = fetch_data()
    .inspect("fetch_data")
    .await;
```

**Test Results:**
- 20 tests passing ✅
- Example tracked 26 tasks with 186 events
- Full poll tracking with timing

---

## 🚧 In Progress

**None** - Ready for next phase!

---

## 📋 Remaining Phases

### **Phase 3: State Machine Introspection** ⏳ (0%)

**Priority:** HIGH
**Estimated Effort:** 2-3 weeks
**Complexity:** ⭐⭐⭐⭐⭐ Very Complex

**Goals:**
- Label each `.await` point in async functions
- Extract current state from Future state machines
- Show which exact `.await` is blocked
- Capture variable values at await points

**Technical Approach:**

**Option A: Proc Macro (Recommended)**
```rust
#[async_inspect::trace]
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;  // <- Labeled "await_1"
    let posts = fetch_posts(id).await;       // <- Labeled "await_2"
    User { profile, posts }
}

// Transforms to:
async fn fetch_user(id: u64) -> User {
    __inspect_await!("fetch_profile", fetch_profile(id)).await;
    __inspect_await!("fetch_posts", fetch_posts(id)).await;
    User { profile, posts }
}
```

**Tasks:**
- [ ] Create proc-macro crate `async-inspect-macros`
- [ ] Parse async functions with `syn`
- [ ] Transform each `.await` to add tracking
- [ ] Generate unique labels for await points
- [ ] Integrate with existing `AwaitGuard`
- [ ] Add source location tracking
- [ ] Handle error propagation (`?`)
- [ ] Support try blocks and async blocks

**Deliverables:**
- `#[trace]` attribute macro
- Automatic await point labeling
- Enhanced reports showing exact blocked location

---

### **Phase 4: Enhanced Visualization** ⏳ (Partially Complete)

**Priority:** MEDIUM
**Estimated Effort:** 1-2 weeks
**Complexity:** ⭐⭐⭐ Moderate

**Goals:**
- ✅ Basic timeline (DONE)
- Enhanced timeline with concurrency view
- Export formats (Chrome DevTools, Perfetto)
- Dependency graph visualization

**Tasks:**
- [x] Basic event timeline
- [ ] Concurrency timeline (Gantt chart style)
- [ ] Export to Chrome `chrome://tracing` format
- [ ] Export to Perfetto format
- [ ] Dependency graph (DOT/Graphviz)
- [ ] Flamegraph generation
- [ ] ASCII art dependency tree

**Example Output:**
```
Timeline (showing concurrent execution):
0ms     100ms    200ms    300ms
|-------|--------|--------|
Task1:  █████████████████████ (completed)
Task2:    ████████ (completed)
Task3:       ███████████████ (blocked)
           ^
           waiting here
```

---

### **Phase 5: Deadlock Detection** ⏳ (0%)

**Priority:** HIGH
**Estimated Effort:** 1-2 weeks
**Complexity:** ⭐⭐⭐⭐ Complex

**Goals:**
- Detect circular wait conditions
- Track lock acquisitions and waiters
- Generate deadlock reports with suggestions

**Technical Approach:**
- Build wait-for graph: `Task -> Resource -> Task`
- Use Tarjan's algorithm for cycle detection
- Track `tokio::sync::Mutex`, `RwLock`, `Semaphore`

**Tasks:**
- [ ] Resource tracking (locks, channels, semaphores)
- [ ] Wait-for graph construction
- [ ] Cycle detection algorithm
- [ ] Deadlock report generation
- [ ] Lock ordering violation detection
- [ ] Integration with common sync primitives
- [ ] Suggestions for fixes

**Example Output:**
```
💀 DEADLOCK DETECTED!

Task #42 → Mutex<Data> @ 0x7f8a → Task #89
Task #89 → Mutex<State> @ 0x7f9b → Task #42

Circular dependency detected!

Suggestion:
  • Acquire locks in consistent order
  • Use try_lock() with timeout
```

---

### **Phase 6: Performance Profiling** ⏳ (0%)

**Priority:** MEDIUM
**Estimated Effort:** 1-2 weeks
**Complexity:** ⭐⭐⭐ Moderate

**Goals:**
- Identify slow operations and hot paths
- Measure lock contention
- Generate performance recommendations

**Tasks:**
- [ ] Poll duration statistics (P50, P95, P99)
- [ ] Lock contention metrics
- [ ] Hot path identification
- [ ] Slow operation detection
- [ ] Performance recommendations
- [ ] Comparison between runs
- [ ] Regression detection

**Example Output:**
```
Performance Report:

Slowest Operations:
  1. fetch_posts() - avg 2.3s (P99: 5.1s)
     Called: 450x
     Suggestion: Add caching or batch requests

  2. acquire_db_lock() - avg 340ms
     Contention: 50 tasks waiting
     Suggestion: Reduce lock scope
```

---

### **Phase 7: TUI Interface** ⏳ (0%)

**Priority:** MEDIUM
**Estimated Effort:** 1-2 weeks
**Complexity:** ⭐⭐⭐ Moderate

**Goals:**
- Real-time interactive terminal dashboard
- Multiple views (tasks, timeline, graph, logs)
- Keyboard navigation

**Technical Approach:**
- Use `ratatui` for terminal UI
- Real-time updates via channels
- Multi-pane layout

**Tasks:**
- [ ] Basic ratatui application structure
- [ ] Task list view with filtering
- [ ] Timeline view with scrolling
- [ ] Dependency graph view
- [ ] Log/event view
- [ ] Keyboard shortcuts
- [ ] Mouse support
- [ ] Real-time refresh
- [ ] Search and filter

**Mockup:**
```
┌─ Tasks (23) ────────┬─ Timeline ─────────────────────┐
│ #1 ✅ task_a        │ 0s    1s    2s    3s          │
│ #2 ⏳ task_b        │ |-----|-----|-----|           │
│ #3 🏃 task_c        │ #1: ████████                  │
│ #4 💀 task_d        │ #2:   ██████████              │
│                     │ #3:     █████                 │
├─────────────────────┴───────────────────────────────┤
│ Events (live):                                      │
│ [2.3s] Task #2: Blocked at fetch_data              │
│ [2.1s] Task #3: Poll started                       │
└─────────────────────────────────────────────────────┘
[h]elp [q]uit [t]imeline [g]raph [f]ilter
```

---

### **Phase 8: Production Ready** ⏳ (0%)

**Priority:** HIGH (for release)
**Estimated Effort:** 1 week
**Complexity:** ⭐⭐ Easy

**Goals:**
- Complete documentation
- Example gallery
- Performance benchmarks
- Release preparation

**Tasks:**
- [ ] Complete API documentation
- [ ] User guide with tutorials
- [ ] Architecture documentation
- [ ] Example gallery (10+ examples)
- [ ] Performance benchmarks
- [ ] CI/CD setup (GitHub Actions)
- [ ] Crates.io publication
- [ ] README badges and shields
- [ ] CHANGELOG.md
- [ ] Release v0.1.0

---

### **Phase 9: Ecosystem Expansion** ⏳ (0%)

**Priority:** LOW (post-release)
**Estimated Effort:** Ongoing
**Complexity:** ⭐⭐⭐⭐ Complex

**Goals:**
- Support more async runtimes
- IDE integration
- Additional language bindings

**Tasks:**
- [ ] async-std runtime support
- [ ] smol runtime support
- [ ] VS Code extension
- [ ] IntelliJ IDEA plugin
- [ ] Language Server Protocol (LSP) integration
- [ ] Web dashboard (separate binary)
- [ ] Cloud deployment monitoring
- [ ] Distributed tracing integration

---

## 📊 Progress Summary

| Phase | Status | Progress | Priority | Complexity |
|-------|--------|----------|----------|------------|
| 1. Foundation | ✅ Done | 100% | ✅ Critical | ⭐⭐⭐ |
| 2. Tokio Integration | ✅ Done | 100% | ✅ High | ⭐⭐⭐ |
| 3. State Machine | ⏳ Planned | 0% | 🔥 High | ⭐⭐⭐⭐⭐ |
| 4. Visualization | 🔄 Partial | 30% | 🟡 Medium | ⭐⭐⭐ |
| 5. Deadlock Detection | ⏳ Planned | 0% | 🔥 High | ⭐⭐⭐⭐ |
| 6. Profiling | ⏳ Planned | 0% | 🟡 Medium | ⭐⭐⭐ |
| 7. TUI | ⏳ Planned | 0% | 🟡 Medium | ⭐⭐⭐ |
| 8. Production | ⏳ Planned | 0% | 🔥 High | ⭐⭐ |
| 9. Ecosystem | ⏳ Planned | 0% | 🟢 Low | ⭐⭐⭐⭐ |

**Overall Progress: 22% complete** (2 of 9 phases done)

---

## 🎯 Recommended Next Steps

### **Immediate Next: Phase 3 - State Machine Introspection**

**Why this next?**
1. **High Value** - Shows exact `.await` blocking point
2. **Differentiator** - Feature tokio-console doesn't have
3. **User Impact** - Solves the core debugging problem

**Alternative: Phase 5 - Deadlock Detection**

**Why this instead?**
1. **Easier to implement** - Well-defined algorithm
2. **Immediate value** - Catches a common bug class
3. **Less risky** - Doesn't require proc macros

**Recommendation:** Start with **Phase 5 (Deadlock Detection)** to build momentum, then tackle **Phase 3 (State Machine)** for the big feature.

---

## 📈 Metrics

### Current Stats
- **Total Lines of Code:** ~2,300
- **Modules:** 7 core modules
- **Tests:** 20 tests, all passing ✅
- **Examples:** 3 working examples
- **Dependencies:** 9 direct dependencies
- **Features:** 2 features (cli, tokio)

### Performance
- **Overhead:** Low (~100 bytes per task)
- **Thread Safety:** Full concurrent access
- **Test Time:** <0.05s for all tests

---

## 🤝 How to Contribute

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on:
- Setting up development environment
- Running tests
- Submitting pull requests
- Architecture decisions

---

## 📅 Release Timeline

**Tentative:**
- **v0.1.0** - Basic functionality (Phase 1-2) ✅ **CURRENT**
- **v0.2.0** - State machine introspection (Phase 3)
- **v0.3.0** - Deadlock detection (Phase 5)
- **v0.4.0** - TUI interface (Phase 7)
- **v1.0.0** - Production ready (Phase 8)

---

**Last Updated:** 2025-11-20
**Status:** Phase 2 Complete, Ready for Phase 3 or 5
