package com.asyncinspect.intellij.toolwindow

import com.asyncinspect.intellij.services.DashboardEvent
import com.asyncinspect.intellij.services.EventListener
import com.asyncinspect.intellij.services.TaskInfo
import java.awt.*
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.JPanel
import javax.swing.JToolTip
import kotlin.math.max
import kotlin.math.min

/**
 * Timeline visualization panel showing task execution over time
 */
class TimelinePanel : JPanel(), EventListener {
    private val tasks = mutableMapOf<Long, TimelineTask>()
    private var startTime: Long = System.currentTimeMillis()
    private var endTime: Long = startTime + 60000 // 60 seconds window
    private var hoveredTask: TimelineTask? = null

    // Colors
    private val colorRunning = Color(59, 130, 246) // Blue
    private val colorCompleted = Color(34, 197, 94) // Green
    private val colorFailed = Color(239, 68, 68) // Red
    private val colorBlocked = Color(234, 179, 8) // Yellow
    private val colorBackground = Color(249, 250, 251)
    private val colorGrid = Color(229, 231, 235)
    private val colorText = Color(55, 65, 81)

    // Layout constants
    private val taskHeight = 24
    private val taskSpacing = 4
    private val leftMargin = 150
    private val rightMargin = 20
    private val topMargin = 40
    private val bottomMargin = 20

    init {
        background = colorBackground
        preferredSize = Dimension(800, 400)

        addMouseMotionListener(object : MouseAdapter() {
            override fun mouseMoved(e: MouseEvent) {
                val task = findTaskAtPoint(e.point)
                if (task != hoveredTask) {
                    hoveredTask = task
                    toolTipText = task?.let { createTooltip(it) }
                    repaint()
                }
            }
        })

        addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                hoveredTask?.let { task ->
                    // TODO: Show task details dialog
                    println("Clicked task: ${task.name} (${task.id})")
                }
            }
        })
    }

    override fun paintComponent(g: Graphics) {
        super.paintComponent(g)
        val g2d = g as Graphics2D
        g2d.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON)

        drawTimeGrid(g2d)
        drawTasks(g2d)
    }

    private fun drawTimeGrid(g2d: Graphics2D) {
        val chartWidth = width - leftMargin - rightMargin
        val chartHeight = height - topMargin - bottomMargin

        // Draw background
        g2d.color = Color.WHITE
        g2d.fillRect(leftMargin, topMargin, chartWidth, chartHeight)

        // Draw grid lines
        g2d.color = colorGrid
        val numGridLines = 6
        for (i in 0..numGridLines) {
            val x = leftMargin + (chartWidth * i / numGridLines)
            g2d.drawLine(x, topMargin, x, topMargin + chartHeight)

            // Time label
            val time = startTime + ((endTime - startTime) * i / numGridLines)
            val relativeTime = (time - startTime) / 1000.0
            val label = String.format("%.1fs", relativeTime)
            g2d.color = colorText
            g2d.font = Font("Arial", Font.PLAIN, 10)
            val fm = g2d.fontMetrics
            g2d.drawString(label, x - fm.stringWidth(label) / 2, topMargin + chartHeight + 15)
        }

        // Draw title
        g2d.color = colorText
        g2d.font = Font("Arial", Font.BOLD, 14)
        g2d.drawString("Task Timeline", leftMargin, topMargin - 15)

        // Draw current time indicator
        val now = System.currentTimeMillis()
        if (now in startTime..endTime) {
            val x = timeToX(now)
            g2d.color = Color.RED
            g2d.drawLine(x, topMargin, x, topMargin + chartHeight)
        }
    }

    private fun drawTasks(g2d: Graphics2D) {
        val chartHeight = height - topMargin - bottomMargin
        var y = topMargin + taskSpacing

        val sortedTasks = tasks.values.sortedBy { it.startTime }

        for (task in sortedTasks) {
            if (y + taskHeight > topMargin + chartHeight) break

            drawTask(g2d, task, y)
            y += taskHeight + taskSpacing
        }
    }

    private fun drawTask(g2d: Graphics2D, task: TimelineTask, y: Int) {
        // Task name
        g2d.color = colorText
        g2d.font = Font("Arial", Font.PLAIN, 11)
        val fm = g2d.fontMetrics
        val name = if (task.name.length > 20) task.name.substring(0, 17) + "..." else task.name
        g2d.drawString(name, 10, y + taskHeight - 6)

        // Task bar
        val startX = max(leftMargin, timeToX(task.startTime))
        val endX = if (task.endTime != null) {
            min(width - rightMargin, timeToX(task.endTime!!))
        } else {
            min(width - rightMargin, timeToX(System.currentTimeMillis()))
        }

        val barWidth = max(2, endX - startX)

        // Choose color based on state
        g2d.color = when (task.state) {
            "Completed" -> colorCompleted
            "Failed" -> colorFailed
            "Blocked" -> colorBlocked
            else -> colorRunning
        }

        // Draw bar
        g2d.fillRoundRect(startX, y, barWidth, taskHeight - 2, 4, 4)

        // Highlight if hovered
        if (task == hoveredTask) {
            g2d.color = Color(0, 0, 0, 50)
            g2d.fillRoundRect(startX, y, barWidth, taskHeight - 2, 4, 4)
            g2d.color = Color.BLACK
            g2d.drawRoundRect(startX, y, barWidth, taskHeight - 2, 4, 4)
        }

        // Draw duration if completed
        if (task.endTime != null && task.duration != null) {
            g2d.color = colorText
            g2d.font = Font("Arial", Font.PLAIN, 9)
            val durationText = String.format("%.1fms", task.duration)
            val textWidth = g2d.fontMetrics.stringWidth(durationText)
            if (barWidth > textWidth + 4) {
                g2d.drawString(durationText, startX + (barWidth - textWidth) / 2, y + taskHeight - 8)
            }
        }
    }

    private fun timeToX(time: Long): Int {
        val chartWidth = width - leftMargin - rightMargin
        val timeRange = endTime - startTime
        val relativeTime = time - startTime
        return leftMargin + (chartWidth * relativeTime / timeRange).toInt()
    }

    private fun findTaskAtPoint(point: Point): TimelineTask? {
        val chartHeight = height - topMargin - bottomMargin
        var y = topMargin + taskSpacing

        val sortedTasks = tasks.values.sortedBy { it.startTime }

        for (task in sortedTasks) {
            if (y + taskHeight > topMargin + chartHeight) break

            val startX = max(leftMargin, timeToX(task.startTime))
            val endX = if (task.endTime != null) {
                min(width - rightMargin, timeToX(task.endTime!!))
            } else {
                min(width - rightMargin, timeToX(System.currentTimeMillis()))
            }

            if (point.x in startX..endX && point.y in y..(y + taskHeight)) {
                return task
            }

            y += taskHeight + taskSpacing
        }

        return null
    }

    private fun createTooltip(task: TimelineTask): String {
        val duration = if (task.duration != null) {
            String.format("%.2f ms", task.duration)
        } else {
            val elapsed = System.currentTimeMillis() - task.startTime
            String.format("%.2f ms (running)", elapsed.toDouble())
        }

        return buildString {
            append("<html>")
            append("<b>${task.name}</b><br>")
            append("ID: ${task.id}<br>")
            append("State: ${task.state}<br>")
            append("Duration: $duration<br>")
            if (task.parent != null) {
                append("Parent: ${task.parent}<br>")
            }
            append("</html>")
        }
    }

    // Event listener implementations
    override fun onTaskSpawned(event: DashboardEvent.TaskSpawned) {
        val now = System.currentTimeMillis()
        tasks[event.taskId] = TimelineTask(
            id = event.taskId,
            name = event.name,
            state = "Running",
            parent = event.parent,
            startTime = now,
            endTime = null,
            duration = null
        )

        // Adjust time window if needed
        if (now > endTime) {
            val windowSize = endTime - startTime
            startTime = now - windowSize / 2
            endTime = now + windowSize / 2
        }

        repaint()
    }

    override fun onTaskCompleted(event: DashboardEvent.TaskCompleted) {
        tasks[event.taskId]?.let { task ->
            tasks[event.taskId] = task.copy(
                state = "Completed",
                endTime = System.currentTimeMillis(),
                duration = event.durationMs
            )
            repaint()
        }
    }

    override fun onTaskFailed(event: DashboardEvent.TaskFailed) {
        tasks[event.taskId]?.let { task ->
            tasks[event.taskId] = task.copy(
                state = "Failed",
                endTime = System.currentTimeMillis()
            )
            repaint()
        }
    }

    override fun onStateChanged(event: DashboardEvent.StateChanged) {
        tasks[event.taskId]?.let { task ->
            tasks[event.taskId] = task.copy(state = event.newState)
            repaint()
        }
    }

    fun clear() {
        tasks.clear()
        startTime = System.currentTimeMillis()
        endTime = startTime + 60000
        hoveredTask = null
        repaint()
    }

    fun setTimeWindow(durationSeconds: Int) {
        val now = System.currentTimeMillis()
        endTime = now
        startTime = now - (durationSeconds * 1000L)
        repaint()
    }
}

/**
 * Timeline task representation
 */
data class TimelineTask(
    val id: Long,
    val name: String,
    val state: String,
    val parent: Long?,
    val startTime: Long,
    val endTime: Long?,
    val duration: Double?
)
