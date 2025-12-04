package com.asyncinspect.intellij.services

import com.intellij.openapi.Disposable
import com.intellij.openapi.components.Service
import com.intellij.openapi.project.Project
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob

/**
 * Project-level service for managing async-inspect in a specific project
 */
@Service(Service.Level.PROJECT)
class ProjectInspectorService(private val project: Project) : Disposable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    var isMonitoring: Boolean = false
        private set

    /**
     * Start monitoring for this project
     */
    fun startMonitoring() {
        if (isMonitoring) return

        val service = project.getService(InspectorService::class.java)
        if (service.connectionState.value !is ConnectionState.Connected) {
            service.connect()
        }

        isMonitoring = true
    }

    /**
     * Stop monitoring for this project
     */
    fun stopMonitoring() {
        isMonitoring = false
    }

    override fun dispose() {
        stopMonitoring()
    }
}
