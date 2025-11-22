import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';

export interface TaskInfo {
    id: number;
    name: string;
    state: 'Pending' | 'Running' | 'Blocked' | 'Completed' | 'Failed';
    created_at: number;
    duration_ms: number;
    poll_count: number;
    location?: string;
    parent_id?: number;
}

export interface DeadlockInfo {
    tasks: number[];
    description: string;
}

export interface Stats {
    total_tasks: number;
    running_tasks: number;
    blocked_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    total_events: number;
    avg_task_duration: number;
}

export class AsyncInspectManager {
    private process?: cp.ChildProcess;
    private tasks: Map<number, TaskInfo> = new Map();
    private deadlocks: DeadlockInfo[] = [];
    private stats: Stats = {
        total_tasks: 0,
        running_tasks: 0,
        blocked_tasks: 0,
        completed_tasks: 0,
        failed_tasks: 0,
        total_events: 0,
        avg_task_duration: 0
    };
    private outputChannel: vscode.OutputChannel;

    constructor(private context: vscode.ExtensionContext) {
        this.outputChannel = vscode.window.createOutputChannel('Async-Inspect');
        context.subscriptions.push(this.outputChannel);
    }

    async start(): Promise<void> {
        if (this.process) {
            vscode.window.showWarningMessage('Async-Inspect is already running');
            return;
        }

        const config = vscode.workspace.getConfiguration('async-inspect');
        const cliPath = config.get<string>('cliPath', 'async-inspect');
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
            this.process.stdout?.on('data', (data: Buffer) => {
                const lines = data.toString().split('\n');
                for (const line of lines) {
                    if (line.trim()) {
                        try {
                            const update = JSON.parse(line);
                            this.handleUpdate(update);
                        } catch (e) {
                            this.outputChannel.appendLine(`Parse error: ${e}`);
                        }
                    }
                }
            });

            // Handle stderr
            this.process.stderr?.on('data', (data: Buffer) => {
                this.outputChannel.appendLine(`Error: ${data.toString()}`);
            });

            // Handle process exit
            this.process.on('close', (code) => {
                this.outputChannel.appendLine(`async-inspect exited with code ${code}`);
                this.process = undefined;
            });

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to start async-inspect: ${error}`);
            this.outputChannel.appendLine(`Error: ${error}`);
        }
    }

    async stop(): Promise<void> {
        if (this.process) {
            this.process.kill();
            this.process = undefined;
            this.outputChannel.appendLine('Stopped async-inspect monitoring');
        }
    }

    async export(filepath: string): Promise<void> {
        const config = vscode.workspace.getConfiguration('async-inspect');
        const cliPath = config.get<string>('cliPath', 'async-inspect');
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0].uri.fsPath;

        const ext = path.extname(filepath);
        const format = ext === '.csv' ? 'csv' : 'json';

        return new Promise((resolve, reject) => {
            const exportProcess = cp.spawn(
                cliPath,
                ['export', '--format', format, '--output', filepath],
                { cwd: workspaceRoot, shell: true }
            );

            exportProcess.on('close', (code) => {
                if (code === 0) {
                    resolve();
                } else {
                    reject(new Error(`Export failed with code ${code}`));
                }
            });
        });
    }

    async clear(): Promise<void> {
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

    async detectDeadlocks(): Promise<DeadlockInfo[]> {
        // For now, return cached deadlocks
        // In future, could trigger real-time analysis
        return this.deadlocks;
    }

    getTasks(): TaskInfo[] {
        return Array.from(this.tasks.values());
    }

    getStats(): Stats {
        return { ...this.stats };
    }

    getDeadlocks(): DeadlockInfo[] {
        return [...this.deadlocks];
    }

    isRunning(): boolean {
        return this.process !== undefined;
    }

    private handleUpdate(update: any): void {
        // Handle different types of updates
        if (update.type === 'task') {
            this.handleTaskUpdate(update.data);
        } else if (update.type === 'deadlock') {
            this.handleDeadlockUpdate(update.data);
        } else if (update.type === 'stats') {
            this.handleStatsUpdate(update.data);
        }
    }

    private handleTaskUpdate(task: any): void {
        const taskInfo: TaskInfo = {
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
        const threshold = config.get<number>('performanceThreshold', 1000);

        if (taskInfo.duration_ms > threshold && taskInfo.state === 'Running') {
            vscode.window.showWarningMessage(
                `Slow task detected: ${taskInfo.name} (${taskInfo.duration_ms}ms)`,
                'Show Task'
            ).then(selection => {
                if (selection === 'Show Task') {
                    vscode.commands.executeCommand('async-inspect.jumpToTask', taskInfo);
                }
            });
        }
    }

    private handleDeadlockUpdate(deadlock: any): void {
        const deadlockInfo: DeadlockInfo = {
            tasks: deadlock.tasks,
            description: deadlock.description
        };

        this.deadlocks.push(deadlockInfo);

        // Show notification for deadlocks
        const config = vscode.workspace.getConfiguration('async-inspect');
        if (config.get('deadlockAlerts')) {
            vscode.window.showErrorMessage(
                `Deadlock detected: ${deadlockInfo.description}`,
                'View Details'
            ).then(selection => {
                if (selection === 'View Details') {
                    vscode.commands.executeCommand('async-inspect.analyzeDeadlocks');
                }
            });
        }
    }

    private handleStatsUpdate(stats: any): void {
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
