package com.asyncinspect.intellij.actions

import com.asyncinspect.intellij.services.InspectorService
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager

/**
 * Action to clear all tracked tasks
 */
class ClearTasksAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val service = ApplicationManager.getApplication().getService(InspectorService::class.java)
        service.clearTasks()
    }

    override fun update(e: AnActionEvent) {
        val service = ApplicationManager.getApplication().getService(InspectorService::class.java)
        e.presentation.isEnabled = service.tasks.value.isNotEmpty()
    }
}
