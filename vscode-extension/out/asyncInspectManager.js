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
exports.AsyncInspectManager = void 0;
const vscode = __importStar(require("vscode"));
const cp = __importStar(require("child_process"));
const path = __importStar(require("path"));
class AsyncInspectManager {
    constructor(context) {
        this.context = context;
        this.tasks = new Map();
        this.deadlocks = [];
        this.stats = {
            total_tasks: 0,
            running_tasks: 0,
            blocked_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            total_events: 0,
            avg_task_duration: 0
        };
        this.outputChannel = vscode.window.createOutputChannel('Async-Inspect');
        context.subscriptions.push(this.outputChannel);
    }
    async start() {
        if (this.process) {
            vscode.window.showWarningMessage('Async-Inspect is already running');
            return;
        }
        const config = vscode.workspace.getConfiguration('async-inspect');
        const cliPath = config.get('cliPath', 'async-inspect');
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0].uri.fsPath;
        if (!workspaceRoot) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }
        try {
            // Start async-inspect monitoring
            this.process = cp.spawn(cliPath, ['monitor', '--json'], {
                cwd: workspaceRoot,
                shell: true
            });
            this.outputChannel.appendLine('Started async-inspect monitoring');
            this.outputChannel.show(true);
            // Handle stdout - parse JSON updates
            this.process.stdout?.on('data', (data) => {
                const lines = data.toString().split('\n');
                for (const line of lines) {
                    if (line.trim()) {
                        try {
                            const update = JSON.parse(line);
                            this.handleUpdate(update);
                        }
                        catch (e) {
                            this.outputChannel.appendLine(`Parse error: ${e}`);
                        }
                    }
                }
            });
            // Handle stderr
            this.process.stderr?.on('data', (data) => {
                this.outputChannel.appendLine(`Error: ${data.toString()}`);
            });
            // Handle process exit
            this.process.on('close', (code) => {
                this.outputChannel.appendLine(`async-inspect exited with code ${code}`);
                this.process = undefined;
            });
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to start async-inspect: ${error}`);
            this.outputChannel.appendLine(`Error: ${error}`);
        }
    }
    async stop() {
        if (this.process) {
            this.process.kill();
            this.process = undefined;
            this.outputChannel.appendLine('Stopped async-inspect monitoring');
        }
    }
    async export(filepath) {
        const config = vscode.workspace.getConfiguration('async-inspect');
        const cliPath = config.get('cliPath', 'async-inspect');
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0].uri.fsPath;
        const ext = path.extname(filepath);
        const format = ext === '.csv' ? 'csv' : 'json';
        return new Promise((resolve, reject) => {
            const exportProcess = cp.spawn(cliPath, ['export', '--format', format, '--output', filepath], { cwd: workspaceRoot, shell: true });
            exportProcess.on('close', (code) => {
                if (code === 0) {
                    resolve();
                }
                else {
                    reject(new Error(`Export failed with code ${code}`));
                }
            });
        });
    }
    async clear() {
        this.tasks.clear();
        this.deadlocks = [];
        this.stats = {
            total_tasks: 0,
            running_tasks: 0,
            blocked_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            total_events: 0,
            avg_task_duration: 0
        };
    }
    async detectDeadlocks() {
        // For now, return cached deadlocks
        // In future, could trigger real-time analysis
        return this.deadlocks;
    }
    getTasks() {
        return Array.from(this.tasks.values());
    }
    getStats() {
        return { ...this.stats };
    }
    getDeadlocks() {
        return [...this.deadlocks];
    }
    isRunning() {
        return this.process !== undefined;
    }
    handleUpdate(update) {
        // Handle different types of updates
        if (update.type === 'task') {
            this.handleTaskUpdate(update.data);
        }
        else if (update.type === 'deadlock') {
            this.handleDeadlockUpdate(update.data);
        }
        else if (update.type === 'stats') {
            this.handleStatsUpdate(update.data);
        }
    }
    handleTaskUpdate(task) {
        const taskInfo = {
            id: task.id,
            name: task.name,
            state: task.state,
            created_at: task.created_at_ms,
            duration_ms: task.duration_ms,
            poll_count: task.poll_count,
            location: task.location,
            parent_id: task.parent_id
        };
        this.tasks.set(taskInfo.id, taskInfo);
        // Show notification for slow tasks
        const config = vscode.workspace.getConfiguration('async-inspect');
        const threshold = config.get('performanceThreshold', 1000);
        if (taskInfo.duration_ms > threshold && taskInfo.state === 'Running') {
            vscode.window.showWarningMessage(`Slow task detected: ${taskInfo.name} (${taskInfo.duration_ms}ms)`, 'Show Task').then(selection => {
                if (selection === 'Show Task') {
                    vscode.commands.executeCommand('async-inspect.jumpToTask', taskInfo);
                }
            });
        }
    }
    handleDeadlockUpdate(deadlock) {
        const deadlockInfo = {
            tasks: deadlock.tasks,
            description: deadlock.description
        };
        this.deadlocks.push(deadlockInfo);
        // Show notification for deadlocks
        const config = vscode.workspace.getConfiguration('async-inspect');
        if (config.get('deadlockAlerts')) {
            vscode.window.showErrorMessage(`Deadlock detected: ${deadlockInfo.description}`, 'View Details').then(selection => {
                if (selection === 'View Details') {
                    vscode.commands.executeCommand('async-inspect.analyzeDeadlocks');
                }
            });
        }
    }
    handleStatsUpdate(stats) {
        this.stats = {
            total_tasks: stats.total_tasks || 0,
            running_tasks: stats.running_tasks || 0,
            blocked_tasks: stats.blocked_tasks || 0,
            completed_tasks: stats.completed_tasks || 0,
            failed_tasks: stats.failed_tasks || 0,
            total_events: stats.total_events || 0,
            avg_task_duration: stats.avg_task_duration || 0
        };
    }
}
exports.AsyncInspectManager = AsyncInspectManager;
//# sourceMappingURL=asyncInspectManager.js.map