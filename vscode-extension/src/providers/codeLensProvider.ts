import * as vscode from 'vscode';
import { AsyncInspectManager } from '../asyncInspectManager';

export class AsyncInspectCodeLensProvider implements vscode.CodeLensProvider {
    private _onDidChangeCodeLenses: vscode.EventEmitter<void> = new vscode.EventEmitter<void>();
    public readonly onDidChangeCodeLenses: vscode.Event<void> = this._onDidChangeCodeLenses.event;

    constructor(private manager: AsyncInspectManager) {}

    public refresh(): void {
        this._onDidChangeCodeLenses.fire();
    }

    public provideCodeLenses(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.CodeLens[]> {
        const config = vscode.workspace.getConfiguration('async-inspect');
        if (!config.get('showInlineStats') || !this.manager.isRunning()) {
            return [];
        }

        const codeLenses: vscode.CodeLens[] = [];
        const text = document.getText();
        const tasks = this.manager.getTasks();

        // Find async functions and test functions
        const asyncFnRegex = /(?:async\s+fn|#\[tokio::test\][\s\S]*?async\s+fn)\s+(\w+)/g;
        let match;

        while ((match = asyncFnRegex.exec(text)) !== null) {
            const functionName = match[1];
            const position = document.positionAt(match.index);

            // Find matching tasks for this function
            const matchingTasks = tasks.filter(t => t.name.includes(functionName));

            if (matchingTasks.length > 0) {
                const avgDuration = matchingTasks.reduce((sum, t) => sum + t.duration_ms, 0) / matchingTasks.length;
                const maxDuration = Math.max(...matchingTasks.map(t => t.duration_ms));
                const totalCalls = matchingTasks.length;

                const statsText = `🔍 ${totalCalls} calls | avg: ${avgDuration.toFixed(1)}ms | max: ${maxDuration.toFixed(1)}ms`;

                const codeLens = new vscode.CodeLens(new vscode.Range(position, position), {
                    title: statsText,
                    command: 'async-inspect.showTimeline',
                    tooltip: `Click to view timeline for ${functionName}`
                });

                codeLenses.push(codeLens);

                // Add warning for slow functions
                const threshold = config.get<number>('performanceThreshold', 1000);
                if (avgDuration > threshold) {
                    const warningLens = new vscode.CodeLens(
                        new vscode.Range(position.translate(1, 0), position.translate(1, 0)),
                        {
                            title: `⚠️  Slow function: avg ${avgDuration.toFixed(1)}ms`,
                            command: '',
                            tooltip: 'This function is slower than the performance threshold'
                        }
                    );
                    codeLenses.push(warningLens);
                }
            }
        }

        return codeLenses;
    }
}
