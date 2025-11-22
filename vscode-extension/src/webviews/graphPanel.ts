import * as vscode from 'vscode';
import { AsyncInspectManager } from '../asyncInspectManager';

export class GraphPanel {
    public static currentPanel: GraphPanel | undefined;
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

        if (GraphPanel.currentPanel) {
            GraphPanel.currentPanel._panel.reveal(column);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'asyncInspectGraph',
            'Async-Inspect Task Graph',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        GraphPanel.currentPanel = new GraphPanel(panel, extensionUri, manager);
    }

    public dispose() {
        GraphPanel.currentPanel = undefined;
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
            <title>Async-Inspect Task Graph</title>
            <style>
                body {
                    font-family: var(--vscode-font-family);
                    color: var(--vscode-foreground);
                    background-color: var(--vscode-editor-background);
                    padding: 0;
                    margin: 0;
                }
                #graph {
                    width: 100vw;
                    height: 100vh;
                }
                .node {
                    fill: var(--vscode-button-background);
                    stroke: var(--vscode-button-border);
                    stroke-width: 2px;
                }
                .link {
                    stroke: var(--vscode-foreground);
                    stroke-opacity: 0.6;
                    stroke-width: 2px;
                }
                .node text {
                    fill: var(--vscode-button-foreground);
                    font-size: 12px;
                    text-anchor: middle;
                }
            </style>
        </head>
        <body>
            <div id="graph">
                <svg width="100%" height="100%">
                    <g id="links"></g>
                    <g id="nodes"></g>
                </svg>
            </div>
            <script>
                const vscode = acquireVsCodeApi();
                vscode.postMessage({ type: 'getTasks' });

                window.addEventListener('message', event => {
                    const message = event.data;
                    if (message.type === 'tasks') {
                        renderGraph(message.tasks);
                    }
                });

                function renderGraph(tasks) {
                    // Simple graph visualization
                    // In a real implementation, use D3.js or similar
                    const svg = document.querySelector('svg');
                    const nodesGroup = document.getElementById('nodes');
                    const linksGroup = document.getElementById('links');

                    nodesGroup.innerHTML = '';
                    linksGroup.innerHTML = '';

                    tasks.forEach((task, idx) => {
                        const x = 100 + (idx % 5) * 150;
                        const y = 100 + Math.floor(idx / 5) * 100;

                        // Create node
                        const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
                        circle.setAttribute('class', 'node');
                        circle.setAttribute('cx', x);
                        circle.setAttribute('cy', y);
                        circle.setAttribute('r', 30);

                        const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                        text.setAttribute('x', x);
                        text.setAttribute('y', y + 5);
                        text.textContent = task.name.substring(0, 10);

                        nodesGroup.appendChild(circle);
                        nodesGroup.appendChild(text);

                        // Create links to parent
                        if (task.parent_id) {
                            const parentIdx = tasks.findIndex(t => t.id === task.parent_id);
                            if (parentIdx >= 0) {
                                const px = 100 + (parentIdx % 5) * 150;
                                const py = 100 + Math.floor(parentIdx / 5) * 100;

                                const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                                line.setAttribute('class', 'link');
                                line.setAttribute('x1', px);
                                line.setAttribute('y1', py);
                                line.setAttribute('x2', x);
                                line.setAttribute('y2', y);

                                linksGroup.appendChild(line);
                            }
                        }
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
