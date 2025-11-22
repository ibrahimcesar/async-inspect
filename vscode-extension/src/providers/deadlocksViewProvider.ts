import * as vscode from 'vscode';
import { AsyncInspectManager, DeadlockInfo } from '../asyncInspectManager';

export class DeadlocksViewProvider implements vscode.TreeDataProvider<DeadlockTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<DeadlockTreeItem | undefined | null | void> = new vscode.EventEmitter<DeadlockTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<DeadlockTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor(private manager: AsyncInspectManager) {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: DeadlockTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(): Thenable<DeadlockTreeItem[]> {
        const deadlocks = this.manager.getDeadlocks();

        if (deadlocks.length === 0) {
            return Promise.resolve([
                new DeadlockTreeItem('No deadlocks detected', '✓', 'check')
            ]);
        }

        return Promise.resolve(deadlocks.map((d, idx) => new DeadlockTreeItem(
            `Deadlock ${idx + 1}`,
            d.description,
            'warning',
            d
        )));
    }
}

class DeadlockTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        description: string,
        iconName: string,
        public readonly deadlock?: DeadlockInfo
    ) {
        super(label, vscode.TreeItemCollapsibleState.None);
        this.description = description;
        this.iconPath = new vscode.ThemeIcon(iconName);
        this.tooltip = deadlock
            ? `Tasks involved: ${deadlock.tasks.join(', ')}\n${deadlock.description}`
            : 'No deadlocks';
    }
}
