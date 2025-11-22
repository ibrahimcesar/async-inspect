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
exports.TaskTreeProvider = void 0;
const vscode = __importStar(require("vscode"));
class TaskTreeProvider {
    constructor(manager) {
        this.manager = manager;
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!this.manager.isRunning()) {
            return Promise.resolve([]);
        }
        if (element) {
            // Return children of a task
            const tasks = this.manager.getTasks();
            const children = tasks.filter(t => t.parent_id === element.task.id);
            return Promise.resolve(children.map(t => new TaskTreeItem(t)));
        }
        else {
            // Return root tasks
            const tasks = this.manager.getTasks();
            const rootTasks = tasks.filter(t => !t.parent_id);
            return Promise.resolve(rootTasks.map(t => new TaskTreeItem(t)));
        }
    }
}
exports.TaskTreeProvider = TaskTreeProvider;
class TaskTreeItem extends vscode.TreeItem {
    constructor(task) {
        super(task.name, vscode.TreeItemCollapsibleState.Collapsed);
        this.task = task;
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
    getDescription() {
        const { state, duration_ms, poll_count } = this.task;
        if (state === 'Running' || state === 'Blocked') {
            return `${state} - ${duration_ms}ms (${poll_count} polls)`;
        }
        else if (state === 'Completed') {
            return `✓ ${duration_ms}ms`;
        }
        else if (state === 'Failed') {
            return `✗ Failed`;
        }
        return state;
    }
    getTooltip() {
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
    getIcon() {
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
//# sourceMappingURL=taskTreeProvider.js.map