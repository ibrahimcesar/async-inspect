import * as vscode from 'vscode';
import { AsyncInspectManager } from '../asyncInspectManager';

export class StatsViewProvider implements vscode.TreeDataProvider<StatsTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<StatsTreeItem | undefined | null | void> = new vscode.EventEmitter<StatsTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<StatsTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor(private manager: AsyncInspectManager) {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: StatsTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(): Thenable<StatsTreeItem[]> {
        if (!this.manager.isRunning()) {
            return Promise.resolve([]);
        }

        const stats = this.manager.getStats();
        return Promise.resolve([
            new StatsTreeItem('Total Tasks', stats.total_tasks.toString(), 'symbol-number'),
            new StatsTreeItem('Running', stats.running_tasks.toString(), 'play', 'async-inspect.taskRunning'),
            new StatsTreeItem('Blocked', stats.blocked_tasks.toString(), 'clock', 'async-inspect.taskBlocked'),
            new StatsTreeItem('Completed', stats.completed_tasks.toString(), 'check', 'async-inspect.taskCompleted'),
            new StatsTreeItem('Failed', stats.failed_tasks.toString(), 'error', 'async-inspect.taskFailed'),
            new StatsTreeItem('Total Events', stats.total_events.toString(), 'symbol-event'),
            new StatsTreeItem('Avg Duration', `${stats.avg_task_duration.toFixed(2)}ms`, 'watch')
        ]);
    }
}

class StatsTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly value: string,
        iconName: string,
        iconColor?: string
    ) {
        super(label, vscode.TreeItemCollapsibleState.None);
        this.description = value;
        this.iconPath = iconColor
            ? new vscode.ThemeIcon(iconName, new vscode.ThemeColor(iconColor))
            : new vscode.ThemeIcon(iconName);
    }
}
