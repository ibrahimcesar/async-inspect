package com.asyncinspect.intellij.toolwindow

import com.asyncinspect.intellij.services.ConnectionState
import com.asyncinspect.intellij.services.InspectorService
import com.asyncinspect.intellij.services.InspectorStats
import com.asyncinspect.intellij.services.TaskInfo
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.table.JBTable
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.collect
import java.awt.BorderLayout
import java.awt.FlowLayout
import javax.swing.*
import javax.swing.table.DefaultTableModel

/**
 * Factory for creating the Async Inspect tool window
 */
class AsyncInspectToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val toolWindowContent = AsyncInspectToolWindowContent(project)
        val content = ContentFactory.getInstance().createContent(toolWindowContent.panel, "", false)
        toolWindow.contentManager.addContent(content)
    }

    override fun shouldBeAvailable(project: Project): Boolean = true
}

/**
 * Main tool window content
 */
class AsyncInspectToolWindowContent(private val project: Project) {
    val panel: JPanel = JPanel(BorderLayout())
    private val service = ApplicationManager.getApplication().getService(InspectorService::class.java)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    // Metrics panel
    private val metricsPanel = JPanel(FlowLayout(FlowLayout.LEFT))
    private val totalTasksLabel = JLabel("Total: 0")
    private val runningTasksLabel = JLabel("Running: 0")
    private val completedTasksLabel = JLabel("Completed: 0")
    private val failedTasksLabel = JLabel("Failed: 0")
    private val blockedTasksLabel = JLabel("Blocked: 0")

    // Connection status
    private val connectionStatusLabel = JLabel("Disconnected")

    // Task table
    private val taskTableModel = DefaultTableModel(
        arrayOf("ID", "Name", "State", "Parent", "Polls", "Duration (ms)"),
        0
    ).apply {
        // Make table non-editable
        override fun isCellEditable(row: Int, column: Int) = false
    }
    private val taskTable = JBTable(taskTableModel).apply {
        setShowGrid(true)
        autoCreateRowSorter = true
    }

    // Event log
    private val eventLogModel = DefaultListModel<String>()
    private val eventLogList = JList(eventLogModel)

    init {
        setupUI()
        observeState()
    }

    private fun setupUI() {
        // Metrics panel
        metricsPanel.apply {
            border = BorderFactory.createTitledBorder("Metrics")
            add(totalTasksLabel)
            add(JSeparator(SwingConstants.VERTICAL))
            add(runningTasksLabel)
            add(JSeparator(SwingConstants.VERTICAL))
            add(completedTasksLabel)
            add(JSeparator(SwingConstants.VERTICAL))
            add(failedTasksLabel)
            add(JSeparator(SwingConstants.VERTICAL))
            add(blockedTasksLabel)
        }

        // Status panel
        val statusPanel = JPanel(FlowLayout(FlowLayout.LEFT)).apply {
            border = BorderFactory.createTitledBorder("Connection")
            add(connectionStatusLabel)
        }

        // Top panel (metrics + status)
        val topPanel = JPanel(BorderLayout()).apply {
            add(metricsPanel, BorderLayout.CENTER)
            add(statusPanel, BorderLayout.SOUTH)
        }

        // Task table panel
        val taskTablePanel = JPanel(BorderLayout()).apply {
            border = BorderFactory.createTitledBorder("Tasks")
            add(JScrollPane(taskTable), BorderLayout.CENTER)
        }

        // Event log panel
        val eventLogPanel = JPanel(BorderLayout()).apply {
            border = BorderFactory.createTitledBorder("Event Log")
            add(JScrollPane(eventLogList), BorderLayout.CENTER)
        }

        // Split pane for table and log
        val splitPane = JSplitPane(JSplitPane.VERTICAL_SPLIT, taskTablePanel, eventLogPanel).apply {
            resizeWeight = 0.7
            dividerLocation = 400
        }

        // Main layout
        panel.apply {
            add(topPanel, BorderLayout.NORTH)
            add(splitPane, BorderLayout.CENTER)
        }
    }

    private fun observeState() {
        // Observe connection state
        scope.launch {
            service.connectionState.collect { state ->
                SwingUtilities.invokeLater {
                    updateConnectionStatus(state)
                }
            }
        }

        // Observe tasks
        scope.launch {
            service.tasks.collect { tasks ->
                SwingUtilities.invokeLater {
                    updateTaskTable(tasks)
                }
            }
        }

        // Observe stats
        scope.launch {
            service.stats.collect { stats ->
                SwingUtilities.invokeLater {
                    updateMetrics(stats)
                }
            }
        }
    }

    private fun updateConnectionStatus(state: ConnectionState) {
        connectionStatusLabel.text = when (state) {
            is ConnectionState.Disconnected -> "Disconnected"
            is ConnectionState.Connecting -> "Connecting..."
            is ConnectionState.Connected -> "Connected to ${state.host}:${state.port}"
            is ConnectionState.Error -> "Error: ${state.message}"
        }

        connectionStatusLabel.foreground = when (state) {
            is ConnectionState.Connected -> java.awt.Color.GREEN.darker()
            is ConnectionState.Error -> java.awt.Color.RED
            else -> java.awt.Color.GRAY
        }
    }

    private fun updateTaskTable(tasks: Map<Long, TaskInfo>) {
        // Clear existing rows
        taskTableModel.rowCount = 0

        // Add task rows
        tasks.values.sortedByDescending { it.id }.forEach { task ->
            taskTableModel.addRow(
                arrayOf(
                    task.id,
                    task.name,
                    task.state,
                    task.parent?.toString() ?: "-",
                    task.pollCount,
                    task.duration?.let { String.format("%.2f", it) } ?: "-"
                )
            )
        }
    }

    private fun updateMetrics(stats: InspectorStats) {
        totalTasksLabel.text = "Total: ${stats.totalTasks}"
        runningTasksLabel.text = "Running: ${stats.runningTasks}"
        completedTasksLabel.text = "Completed: ${stats.completedTasks}"
        failedTasksLabel.text = "Failed: ${stats.failedTasks}"
        blockedTasksLabel.text = "Blocked: ${stats.blockedTasks}"
    }

    fun addEventLog(message: String) {
        SwingUtilities.invokeLater {
            eventLogModel.add(0, message)
            // Keep only last 1000 events
            while (eventLogModel.size > 1000) {
                eventLogModel.remove(eventLogModel.size - 1)
            }
        }
    }
}
