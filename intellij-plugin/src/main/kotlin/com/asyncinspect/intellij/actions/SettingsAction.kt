package com.asyncinspect.intellij.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.ui.DialogWrapper
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBTextField
import java.awt.BorderLayout
import java.awt.GridLayout
import javax.swing.*

/**
 * Action to open settings dialog
 */
class SettingsAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        SettingsDialog(project).show()
    }
}

/**
 * Settings dialog
 */
class SettingsDialog(project: com.intellij.openapi.project.Project) : DialogWrapper(project) {
    private val hostField = JBTextField("localhost", 20)
    private val portField = JBTextField("8080", 10)
    private val autoConnectCheckbox = JCheckBox("Auto-connect on project open")

    init {
        title = "Async Inspect Settings"
        init()
    }

    override fun createCenterPanel(): JComponent {
        val panel = JPanel(BorderLayout())

        val formPanel = JPanel(GridLayout(3, 2, 10, 10)).apply {
            border = BorderFactory.createEmptyBorder(10, 10, 10, 10)

            add(JBLabel("Dashboard Host:"))
            add(hostField)

            add(JBLabel("Dashboard Port:"))
            add(portField)

            add(JBLabel(""))
            add(autoConnectCheckbox)
        }

        panel.add(formPanel, BorderLayout.CENTER)

        return panel
    }

    override fun doOKAction() {
        // TODO: Save settings
        val host = hostField.text
        val port = portField.text.toIntOrNull() ?: 8080

        println("Settings saved: $host:$port, auto-connect: ${autoConnectCheckbox.isSelected}")

        super.doOKAction()
    }
}
