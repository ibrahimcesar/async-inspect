import * as vscode from 'vscode';
import { AsyncInspectManager, TaskInfo } from '../asyncInspectManager';

export class TaskTreeProvider implements vscode.TreeDataProvider<TaskTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<TaskTreeItem | undefined | null | void> = new vscode.EventEmitter<TaskTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TaskTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    constructor(private manager: AsyncInspectManager) {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TaskTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: TaskTreeItem): Thenable<TaskTreeItem[]> {
        if (!this.manager.isRunning()) {
            return Promise.resolve([]);
        }

        if (element) {
            // Return children of a task
            const tasks = this.manager.getTasks();
            const children = tasks.filter(t => t.parent_id === element.task.id);
            return Promise.resolve(children.map(t => new TaskTreeItem(t)));
        } else {
            // Return root tasks
            const tasks = this.manager.getTasks();
            const rootTasks = tasks.filter(t => !t.parent_id);
            return Promise.resolve(rootTasks.map(t => new TaskTreeItem(t)));
        }
    }
}

class TaskTreeItem extends vscode.TreeItem {
    constructor(public readonly task: TaskInfo) {
        super(
            task.name,
            vscode.TreeItemCollapsibleState.Collapsed
        );

        this.tooltip = this.getTooltip();
        this.description = this.getDescription();
        this.iconPath = this.getIcon();
        this.contextValue = 'task';

        // Command to jump to task location
        if (task.location) {
            this.command = {
                command: 'async-inspect.jumpToTask',
                title: 'Jump to Task',
                arguments: [task]
            };
        }
    }

    private getDescription(): string {
        const { state, duration_ms, poll_count } = this.task;

        if (state === 'Running' || state === 'Blocked') {
            return `${state} - ${duration_ms}ms (${poll_count} polls)`;
        } else if (state === 'Completed') {
            return `✓ ${duration_ms}ms`;
        } else if (state === 'Failed') {
            return `✗ Failed`;
        }

        return state;
    }

    private getTooltip(): string {
        const { id, name, state, duration_ms, poll_count, location } = this.task;

        let tooltip = `Task #${id}: ${name}\n`;
        tooltip += `State: ${state}\n`;
        tooltip += `Duration: ${duration_ms}ms\n`;
        tooltip += `Polls: ${poll_count}\n`;

        if (location) {
            tooltip += `Location: ${location}\n`;
        }

        return tooltip;
    }

    private getIcon(): vscode.ThemeIcon {
        switch (this.task.state) {
            case 'Running':
                return new vscode.ThemeIcon('play', new vscode.ThemeColor('async-inspect.taskRunning'));
            case 'Blocked':
                return new vscode.ThemeIcon('clock', new vscode.ThemeColor('async-inspect.taskBlocked'));
            case 'Completed':
                return new vscode.ThemeIcon('check', new vscode.ThemeColor('async-inspect.taskCompleted'));
            case 'Failed':
                return new vscode.ThemeIcon('error', new vscode.ThemeColor('async-inspect.taskFailed'));
            default:
                return new vscode.ThemeIcon('circle-outline');
        }
    }
}
