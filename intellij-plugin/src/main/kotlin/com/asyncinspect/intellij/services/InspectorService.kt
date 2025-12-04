package com.asyncinspect.intellij.services

import com.google.gson.Gson
import com.google.gson.JsonObject
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.Disposable
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.io.BufferedReader
import java.io.InputStreamReader
import java.net.HttpURLConnection
import java.net.URI
import java.net.http.HttpClient
import java.net.http.WebSocket
import java.nio.ByteBuffer
import java.util.concurrent.CompletionStage
import java.util.concurrent.ConcurrentHashMap

/**
 * Application-level service for managing async-inspect connections
 */
@Service(Service.Level.APP)
class InspectorService : Disposable {
    private val logger = Logger.getInstance(InspectorService::class.java)
    private val gson = Gson()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    private var webSocket: WebSocket? = null
    private var httpClient: HttpClient? = null

    private val _connectionState = MutableStateFlow<ConnectionState>(ConnectionState.Disconnected)
    val connectionState: StateFlow<ConnectionState> = _connectionState.asStateFlow()

    private val _tasks = MutableStateFlow<Map<Long, TaskInfo>>(emptyMap())
    val tasks: StateFlow<Map<Long, TaskInfo>> = _tasks.asStateFlow()

    private val _stats = MutableStateFlow<InspectorStats>(InspectorStats())
    val stats: StateFlow<InspectorStats> = _stats.asStateFlow()

    private val listeners = ConcurrentHashMap<String, EventListener>()

    /**
     * Connect to the async-inspect dashboard WebSocket
     */
    fun connect(host: String = "localhost", port: Int = 8080) {
        if (_connectionState.value is ConnectionState.Connected) {
            logger.warn("Already connected to async-inspect dashboard")
            return
        }

        scope.launch {
            try {
                _connectionState.value = ConnectionState.Connecting

                val client = HttpClient.newHttpClient()
                httpClient = client

                val ws = client.newWebSocketBuilder()
                    .buildAsync(URI.create("ws://$host:$port/ws"), InspectorWebSocketListener())
                    .get()

                webSocket = ws
                _connectionState.value = ConnectionState.Connected(host, port)

                logger.info("Connected to async-inspect dashboard at $host:$port")
                notifyInfo("Connected to async-inspect dashboard at $host:$port")

                // Fetch initial state via REST API
                fetchInitialState(host, port)

            } catch (e: Exception) {
                logger.error("Failed to connect to async-inspect dashboard", e)
                _connectionState.value = ConnectionState.Error(e.message ?: "Connection failed")
                notifyError("Failed to connect: ${e.message}")
            }
        }
    }

    /**
     * Disconnect from the dashboard
     */
    fun disconnect() {
        webSocket?.sendClose(WebSocket.NORMAL_CLOSURE, "Client disconnect")
        webSocket = null
        httpClient = null
        _connectionState.value = ConnectionState.Disconnected
        _tasks.value = emptyMap()
        _stats.value = InspectorStats()

        logger.info("Disconnected from async-inspect dashboard")
    }

    /**
     * Clear all tracked tasks
     */
    fun clearTasks() {
        _tasks.value = emptyMap()
        _stats.value = InspectorStats()
    }

    /**
     * Register an event listener
     */
    fun addEventListener(id: String, listener: EventListener) {
        listeners[id] = listener
    }

    /**
     * Unregister an event listener
     */
    fun removeEventListener(id: String) {
        listeners.remove(id)
    }

    /**
     * Fetch initial state from REST API
     */
    private suspend fun fetchInitialState(host: String, port: Int) {
        withContext(Dispatchers.IO) {
            try {
                val url = URI.create("http://$host:$port/api/tasks").toURL()
                val connection = url.openConnection() as HttpURLConnection
                connection.requestMethod = "GET"
                connection.connectTimeout = 5000
                connection.readTimeout = 5000

                val reader = BufferedReader(InputStreamReader(connection.inputStream))
                val response = reader.readText()
                reader.close()

                val json = gson.fromJson(response, JsonObject::class.java)
                val tasksArray = json.getAsJsonArray("tasks")

                val taskMap = mutableMapOf<Long, TaskInfo>()
                tasksArray.forEach { element ->
                    val taskObj = element.asJsonObject
                    val taskInfo = TaskInfo(
                        id = taskObj.get("id").asLong,
                        name = taskObj.get("name").asString,
                        state = taskObj.get("state").asString,
                        parent = taskObj.get("parent")?.asLong,
                        pollCount = taskObj.get("poll_count")?.asInt ?: 0,
                        duration = null
                    )
                    taskMap[taskInfo.id] = taskInfo
                }

                _tasks.value = taskMap

            } catch (e: Exception) {
                logger.error("Failed to fetch initial state", e)
            }
        }
    }

    /**
     * Handle incoming WebSocket events
     */
    private fun handleEvent(event: DashboardEvent) {
        when (event) {
            is DashboardEvent.TaskSpawned -> {
                val tasks = _tasks.value.toMutableMap()
                tasks[event.taskId] = TaskInfo(
                    id = event.taskId,
                    name = event.name,
                    state = "Running",
                    parent = event.parent,
                    pollCount = 0,
                    duration = null
                )
                _tasks.value = tasks

                listeners.values.forEach { it.onTaskSpawned(event) }
            }

            is DashboardEvent.TaskCompleted -> {
                val tasks = _tasks.value.toMutableMap()
                tasks[event.taskId]?.let { task ->
                    tasks[event.taskId] = task.copy(state = "Completed", duration = event.durationMs)
                }
                _tasks.value = tasks

                listeners.values.forEach { it.onTaskCompleted(event) }
            }

            is DashboardEvent.TaskFailed -> {
                val tasks = _tasks.value.toMutableMap()
                tasks[event.taskId]?.let { task ->
                    tasks[event.taskId] = task.copy(state = "Failed")
                }
                _tasks.value = tasks

                listeners.values.forEach { it.onTaskFailed(event) }
            }

            is DashboardEvent.StateChanged -> {
                val tasks = _tasks.value.toMutableMap()
                tasks[event.taskId]?.let { task ->
                    tasks[event.taskId] = task.copy(state = event.newState)
                }
                _tasks.value = tasks

                listeners.values.forEach { it.onStateChanged(event) }
            }

            is DashboardEvent.MetricsSnapshot -> {
                _stats.value = InspectorStats(
                    totalTasks = event.totalTasks,
                    runningTasks = event.runningTasks,
                    completedTasks = event.completedTasks,
                    failedTasks = event.failedTasks,
                    blockedTasks = event.blockedTasks
                )

                listeners.values.forEach { it.onMetricsSnapshot(event) }
            }

            is DashboardEvent.AwaitStarted -> {
                listeners.values.forEach { it.onAwaitStarted(event) }
            }

            is DashboardEvent.AwaitEnded -> {
                listeners.values.forEach { it.onAwaitEnded(event) }
            }
        }
    }

    /**
     * WebSocket listener implementation
     */
    private inner class InspectorWebSocketListener : WebSocket.Listener {
        private val textBuffer = StringBuilder()

        override fun onText(webSocket: WebSocket, data: CharSequence, last: Boolean): CompletionStage<*>? {
            textBuffer.append(data)

            if (last) {
                val json = textBuffer.toString()
                textBuffer.clear()

                try {
                    val event = parseEvent(json)
                    handleEvent(event)
                } catch (e: Exception) {
                    logger.error("Failed to parse event: $json", e)
                }
            }

            webSocket.request(1)
            return null
        }

        override fun onClose(webSocket: WebSocket, statusCode: Int, reason: String): CompletionStage<*>? {
            logger.info("WebSocket closed: $statusCode - $reason")
            _connectionState.value = ConnectionState.Disconnected
            return null
        }

        override fun onError(webSocket: WebSocket, error: Throwable) {
            logger.error("WebSocket error", error)
            _connectionState.value = ConnectionState.Error(error.message ?: "WebSocket error")
            notifyError("Connection error: ${error.message}")
        }
    }

    /**
     * Parse dashboard event from JSON
     */
    private fun parseEvent(json: String): DashboardEvent {
        val obj = gson.fromJson(json, JsonObject::class.java)
        val type = obj.get("type").asString

        return when (type) {
            "task_spawned" -> DashboardEvent.TaskSpawned(
                taskId = obj.get("task_id").asLong,
                name = obj.get("name").asString,
                parent = obj.get("parent")?.asLong,
                timestamp = obj.get("timestamp").asLong
            )

            "task_completed" -> DashboardEvent.TaskCompleted(
                taskId = obj.get("task_id").asLong,
                durationMs = obj.get("duration_ms").asDouble,
                timestamp = obj.get("timestamp").asLong
            )

            "task_failed" -> DashboardEvent.TaskFailed(
                taskId = obj.get("task_id").asLong,
                error = obj.get("error")?.asString,
                timestamp = obj.get("timestamp").asLong
            )

            "state_changed" -> DashboardEvent.StateChanged(
                taskId = obj.get("task_id").asLong,
                oldState = obj.get("old_state").asString,
                newState = obj.get("new_state").asString,
                timestamp = obj.get("timestamp").asLong
            )

            "metrics_snapshot" -> DashboardEvent.MetricsSnapshot(
                totalTasks = obj.get("total_tasks").asInt,
                runningTasks = obj.get("running_tasks").asInt,
                completedTasks = obj.get("completed_tasks").asInt,
                failedTasks = obj.get("failed_tasks").asInt,
                blockedTasks = obj.get("blocked_tasks").asInt,
                timestamp = obj.get("timestamp").asLong
            )

            "await_started" -> DashboardEvent.AwaitStarted(
                taskId = obj.get("task_id").asLong,
                label = obj.get("label").asString,
                timestamp = obj.get("timestamp").asLong
            )

            "await_ended" -> DashboardEvent.AwaitEnded(
                taskId = obj.get("task_id").asLong,
                label = obj.get("label").asString,
                durationMs = obj.get("duration_ms").asDouble,
                timestamp = obj.get("timestamp").asLong
            )

            else -> throw IllegalArgumentException("Unknown event type: $type")
        }
    }

    private fun notifyInfo(message: String) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("Async Inspect Notifications")
            .createNotification(message, NotificationType.INFORMATION)
            .notify(null)
    }

    private fun notifyError(message: String) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("Async Inspect Notifications")
            .createNotification(message, NotificationType.ERROR)
            .notify(null)
    }

    override fun dispose() {
        disconnect()
        scope.cancel()
    }
}

/**
 * Connection state
 */
sealed class ConnectionState {
    object Disconnected : ConnectionState()
    object Connecting : ConnectionState()
    data class Connected(val host: String, val port: Int) : ConnectionState()
    data class Error(val message: String) : ConnectionState()
}

/**
 * Task information
 */
data class TaskInfo(
    val id: Long,
    val name: String,
    val state: String,
    val parent: Long?,
    val pollCount: Int,
    val duration: Double?
)

/**
 * Inspector statistics
 */
data class InspectorStats(
    val totalTasks: Int = 0,
    val runningTasks: Int = 0,
    val completedTasks: Int = 0,
    val failedTasks: Int = 0,
    val blockedTasks: Int = 0
)

/**
 * Dashboard events
 */
sealed class DashboardEvent {
    data class TaskSpawned(
        val taskId: Long,
        val name: String,
        val parent: Long?,
        val timestamp: Long
    ) : DashboardEvent()

    data class TaskCompleted(
        val taskId: Long,
        val durationMs: Double,
        val timestamp: Long
    ) : DashboardEvent()

    data class TaskFailed(
        val taskId: Long,
        val error: String?,
        val timestamp: Long
    ) : DashboardEvent()

    data class StateChanged(
        val taskId: Long,
        val oldState: String,
        val newState: String,
        val timestamp: Long
    ) : DashboardEvent()

    data class MetricsSnapshot(
        val totalTasks: Int,
        val runningTasks: Int,
        val completedTasks: Int,
        val failedTasks: Int,
        val blockedTasks: Int,
        val timestamp: Long
    ) : DashboardEvent()

    data class AwaitStarted(
        val taskId: Long,
        val label: String,
        val timestamp: Long
    ) : DashboardEvent()

    data class AwaitEnded(
        val taskId: Long,
        val label: String,
        val durationMs: Double,
        val timestamp: Long
    ) : DashboardEvent()
}

/**
 * Event listener interface
 */
interface EventListener {
    fun onTaskSpawned(event: DashboardEvent.TaskSpawned) {}
    fun onTaskCompleted(event: DashboardEvent.TaskCompleted) {}
    fun onTaskFailed(event: DashboardEvent.TaskFailed) {}
    fun onStateChanged(event: DashboardEvent.StateChanged) {}
    fun onMetricsSnapshot(event: DashboardEvent.MetricsSnapshot) {}
    fun onAwaitStarted(event: DashboardEvent.AwaitStarted) {}
    fun onAwaitEnded(event: DashboardEvent.AwaitEnded) {}
}
