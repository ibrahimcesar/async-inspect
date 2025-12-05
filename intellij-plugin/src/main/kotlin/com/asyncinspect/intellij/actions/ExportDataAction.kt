package com.asyncinspect.intellij.actions

import com.asyncinspect.intellij.services.InspectorService
import com.google.gson.GsonBuilder
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.fileChooser.FileChooserFactory
import com.intellij.openapi.fileChooser.FileSaverDescriptor
import com.intellij.openapi.vfs.VirtualFileWrapper
import java.io.File

/**
 * Action to export task data to JSON
 */
class ExportDataAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val service = ApplicationManager.getApplication().getService(InspectorService::class.java)
        val project = e.project ?: return

        // Create file chooser
        val descriptor = FileSaverDescriptor(
            "Export Async Inspect Data",
            "Choose location to save task data",
            "json"
        )

        val chooser = FileChooserFactory.getInstance().createSaveFileDialog(descriptor, project)
        val fileWrapper: VirtualFileWrapper = chooser.save(null, "async-inspect-export.json") ?: return

        val file = fileWrapper.file

        try {
            // Collect data
            val data = mapOf(
                "tasks" to service.tasks.value,
                "stats" to service.stats.value,
                "exportedAt" to System.currentTimeMillis()
            )

            // Write to file
            val gson = GsonBuilder().setPrettyPrinting().create()
            val json = gson.toJson(data)
            file.writeText(json)

            // Show success notification
            com.intellij.notification.NotificationGroupManager.getInstance()
                .getNotificationGroup("Async Inspect Notifications")
                .createNotification(
                    "Data exported successfully to ${file.absolutePath}",
                    com.intellij.notification.NotificationType.INFORMATION
                )
                .notify(project)

        } catch (ex: Exception) {
            com.intellij.notification.NotificationGroupManager.getInstance()
                .getNotificationGroup("Async Inspect Notifications")
                .createNotification(
                    "Failed to export data: ${ex.message}",
                    com.intellij.notification.NotificationType.ERROR
                )
                .notify(project)
        }
    }

    override fun update(e: AnActionEvent) {
        val service = ApplicationManager.getApplication().getService(InspectorService::class.java)
        e.presentation.isEnabled = service.tasks.value.isNotEmpty()
    }
}
