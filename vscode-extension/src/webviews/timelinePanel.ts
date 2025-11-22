import * as vscode from 'vscode';
import { AsyncInspectManager } from '../asyncInspectManager';

export class TimelinePanel {
    public static currentPanel: TimelinePanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];

    private constructor(
        panel: vscode.WebviewPanel,
        extensionUri: vscode.Uri,
        private manager: AsyncInspectManager
    ) {
        this._panel = panel;
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
        this._panel.webview.html = this._getHtmlForWebview(this._panel.webview);
        this._setWebviewMessageListener(this._panel.webview);
    }

    public static createOrShow(extensionUri: vscode.Uri, manager: AsyncInspectManager) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        if (TimelinePanel.currentPanel) {
            TimelinePanel.currentPanel._panel.reveal(column);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'asyncInspectTimeline',
            'Async-Inspect Timeline',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        TimelinePanel.currentPanel = new TimelinePanel(panel, extensionUri, manager);
    }

    public dispose() {
        TimelinePanel.currentPanel = undefined;
        this._panel.dispose();

        while (this._disposables.length) {
            const disposable = this._disposables.pop();
            if (disposable) {
                disposable.dispose();
            }
        }
    }

    private _getHtmlForWebview(webview: vscode.Webview): string {
        return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Async-Inspect Timeline</title>
            <style>
                body {
                    font-family: var(--vscode-font-family);
                    color: var(--vscode-foreground);
                    background-color: var(--vscode-editor-background);
                    padding: 20px;
                }
                .timeline {
                    margin-top: 20px;
                }
                .task-bar {
                    height: 30px;
                    margin: 5px 0;
                    display: flex;
                    align-items: center;
                }
                .task-label {
                    width: 200px;
                    padding-right: 10px;
                }
                .task-duration {
                    background: var(--vscode-button-background);
                    height: 20px;
                    border-radius: 3px;
                    padding: 0 10px;
                    display: flex;
                    align-items: center;
                    color: var(--vscode-button-foreground);
                }
            </style>
        </head>
        <body>
            <h1>📈 Task Timeline</h1>
            <div class="timeline" id="timeline">
                <p>Loading tasks...</p>
            </div>
            <script>
                const vscode = acquireVsCodeApi();

                // Request task data
                vscode.postMessage({ type: 'getTasks' });

                // Handle messages from extension
                window.addEventListener('message', event => {
                    const message = event.data;
                    if (message.type === 'tasks') {
                        renderTimeline(message.tasks);
                    }
                });

                function renderTimeline(tasks) {
                    const timeline = document.getElementById('timeline');
                    timeline.innerHTML = '';

                    tasks.forEach(task => {
                        const taskBar = document.createElement('div');
                        taskBar.className = 'task-bar';

                        const label = document.createElement('div');
                        label.className = 'task-label';
                        label.textContent = task.name;

                        const duration = document.createElement('div');
                        duration.className = 'task-duration';
                        duration.style.width = (task.duration_ms / 10) + 'px';
                        duration.textContent = task.duration_ms + 'ms';

                        taskBar.appendChild(label);
                        taskBar.appendChild(duration);
                        timeline.appendChild(taskBar);
                    });
                }
            </script>
        </body>
        </html>`;
    }

    private _setWebviewMessageListener(webview: vscode.Webview) {
        webview.onDidReceiveMessage(
            (message: any) => {
                if (message.type === 'getTasks') {
                    const tasks = this.manager.getTasks();
                    webview.postMessage({ type: 'tasks', tasks });
                }
            },
            undefined,
            this._disposables
        );
    }
}
