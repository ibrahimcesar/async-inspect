package com.asyncinspect.intellij.actions

import com.asyncinspect.intellij.services.ConnectionState
import com.asyncinspect.intellij.services.InspectorService
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager

/**
 * Action to start monitoring async tasks
 */
class StartMonitoringAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val service = ApplicationManager.getApplication().getService(InspectorService::class.java)
        service.connect()
    }

    override fun update(e: AnActionEvent) {
        val service = ApplicationManager.getApplication().getService(InspectorService::class.java)
        e.presentation.isEnabled = service.connectionState.value !is ConnectionState.Connected
    }
}
