import * as vscode from 'vscode';
import { TaskTreeProvider } from './providers/taskTreeProvider';
import { StatsViewProvider } from './providers/statsViewProvider';
import { DeadlocksViewProvider } from './providers/deadlocksViewProvider';
import { AsyncInspectCodeLensProvider } from './providers/codeLensProvider';
import { TimelinePanel } from './webviews/timelinePanel';
import { GraphPanel } from './webviews/graphPanel';
import { AsyncInspectManager } from './asyncInspectManager';

let asyncInspectManager: AsyncInspectManager | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('async-inspect extension is now active!');

    // Initialize manager
    asyncInspectManager = new AsyncInspectManager(context);

    // Register tree view providers
    const taskTreeProvider = new TaskTreeProvider(asyncInspectManager);
    const statsViewProvider = new StatsViewProvider(asyncInspectManager);
    const deadlocksViewProvider = new DeadlocksViewProvider(asyncInspectManager);

    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('async-inspect.tasksView', taskTreeProvider),
        vscode.window.registerTreeDataProvider('async-inspect.statsView', statsViewProvider),
        vscode.window.registerTreeDataProvider('async-inspect.deadlocksView', deadlocksViewProvider)
    );

    // Register CodeLens provider for Rust files
    const codeLensProvider = new AsyncInspectCodeLensProvider(asyncInspectManager);
    context.subscriptions.push(
        vscode.languages.registerCodeLensProvider(
            { language: 'rust', scheme: 'file' },
            codeLensProvider
        )
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('async-inspect.start', async () => {
            await asyncInspectManager?.start();
            vscode.window.showInformationMessage('Async-Inspect: Monitoring started');
        }),

        vscode.commands.registerCommand('async-inspect.stop', async () => {
            await asyncInspectManager?.stop();
            vscode.window.showInformationMessage('Async-Inspect: Monitoring stopped');
        }),

        vscode.commands.registerCommand('async-inspect.export', async () => {
            const uri = await vscode.window.showSaveDialog({
                filters: { 'JSON': ['json'], 'CSV': ['csv'] },
                defaultUri: vscode.Uri.file('async-inspect-export.json')
            });

            if (uri) {
                await asyncInspectManager?.export(uri.fsPath);
                vscode.window.showInformationMessage(`Exported to ${uri.fsPath}`);
            }
        }),

        vscode.commands.registerCommand('async-inspect.clear', async () => {
            await asyncInspectManager?.clear();
            taskTreeProvider.refresh();
            statsViewProvider.refresh();
            deadlocksViewProvider.refresh();
            vscode.window.showInformationMessage('Async-Inspect: History cleared');
        }),

        vscode.commands.registerCommand('async-inspect.showGraph', () => {
            GraphPanel.createOrShow(context.extensionUri, asyncInspectManager!);
        }),

        vscode.commands.registerCommand('async-inspect.showTimeline', () => {
            TimelinePanel.createOrShow(context.extensionUri, asyncInspectManager!);
        }),

        vscode.commands.registerCommand('async-inspect.analyzeDeadlocks', async () => {
            const deadlocks = await asyncInspectManager?.detectDeadlocks();
            if (deadlocks && deadlocks.length > 0) {
                vscode.window.showWarningMessage(
                    `Found ${deadlocks.length} potential deadlock(s)`,
                    'Show Details'
                ).then(selection => {
                    if (selection === 'Show Details') {
                        deadlocksViewProvider.refresh();
                    }
                });
            } else {
                vscode.window.showInformationMessage('No deadlocks detected');
            }
        }),

        vscode.commands.registerCommand('async-inspect.refreshTasks', () => {
            taskTreeProvider.refresh();
            statsViewProvider.refresh();
            deadlocksViewProvider.refresh();
        }),

        vscode.commands.registerCommand('async-inspect.jumpToTask', (task: any) => {
            if (task.location) {
                const [file, line] = task.location.split(':');
                const lineNum = parseInt(line, 10) - 1;
                vscode.workspace.openTextDocument(file).then(doc => {
                    vscode.window.showTextDocument(doc).then(editor => {
                        const position = new vscode.Position(lineNum, 0);
                        editor.selection = new vscode.Selection(position, position);
                        editor.revealRange(new vscode.Range(position, position));
                    });
                });
            }
        })
    );

    // Auto-start if configured
    const config = vscode.workspace.getConfiguration('async-inspect');
    if (config.get('autoStart')) {
        asyncInspectManager.start();
    }

    // Set up refresh interval
    const refreshInterval = config.get<number>('refreshInterval', 500);
    setInterval(() => {
        if (asyncInspectManager?.isRunning()) {
            taskTreeProvider.refresh();
            statsViewProvider.refresh();
            deadlocksViewProvider.refresh();
            codeLensProvider.refresh();
        }
    }, refreshInterval);
}

export function deactivate() {
    asyncInspectManager?.stop();
}
