"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.GraphPanel = void 0;
const vscode = __importStar(require("vscode"));
class GraphPanel {
    constructor(panel, extensionUri, manager) {
        this.manager = manager;
        this._disposables = [];
        this._panel = panel;
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
        this._panel.webview.html = this._getHtmlForWebview(this._panel.webview);
        this._setWebviewMessageListener(this._panel.webview);
    }
    static createOrShow(extensionUri, manager) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;
        if (GraphPanel.currentPanel) {
            GraphPanel.currentPanel._panel.reveal(column);
            return;
        }
        const panel = vscode.window.createWebviewPanel('asyncInspectGraph', 'Async-Inspect Task Graph', column || vscode.ViewColumn.One, {
            enableScripts: true,
            retainContextWhenHidden: true
        });
        GraphPanel.currentPanel = new GraphPanel(panel, extensionUri, manager);
    }
    dispose() {
        GraphPanel.currentPanel = undefined;
        this._panel.dispose();
        while (this._disposables.length) {
            const disposable = this._disposables.pop();
            if (disposable) {
                disposable.dispose();
            }
        }
    }
    _getHtmlForWebview(webview) {
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
    _setWebviewMessageListener(webview) {
        webview.onDidReceiveMessage((message) => {
            if (message.type === 'getTasks') {
                const tasks = this.manager.getTasks();
                webview.postMessage({ type: 'tasks', tasks });
            }
        }, undefined, this._disposables);
    }
}
exports.GraphPanel = GraphPanel;
//# sourceMappingURL=graphPanel.js.map