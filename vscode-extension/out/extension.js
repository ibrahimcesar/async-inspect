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
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const taskTreeProvider_1 = require("./providers/taskTreeProvider");
const statsViewProvider_1 = require("./providers/statsViewProvider");
const deadlocksViewProvider_1 = require("./providers/deadlocksViewProvider");
const codeLensProvider_1 = require("./providers/codeLensProvider");
const timelinePanel_1 = require("./webviews/timelinePanel");
const graphPanel_1 = require("./webviews/graphPanel");
const asyncInspectManager_1 = require("./asyncInspectManager");
let asyncInspectManager;
function activate(context) {
    console.log('async-inspect extension is now active!');
    // Initialize manager
    asyncInspectManager = new asyncInspectManager_1.AsyncInspectManager(context);
    // Register tree view providers
    const taskTreeProvider = new taskTreeProvider_1.TaskTreeProvider(asyncInspectManager);
    const statsViewProvider = new statsViewProvider_1.StatsViewProvider(asyncInspectManager);
    const deadlocksViewProvider = new deadlocksViewProvider_1.DeadlocksViewProvider(asyncInspectManager);
    context.subscriptions.push(vscode.window.registerTreeDataProvider('async-inspect.tasksView', taskTreeProvider), vscode.window.registerTreeDataProvider('async-inspect.statsView', statsViewProvider), vscode.window.registerTreeDataProvider('async-inspect.deadlocksView', deadlocksViewProvider));
    // Register CodeLens provider for Rust files
    const codeLensProvider = new codeLensProvider_1.AsyncInspectCodeLensProvider(asyncInspectManager);
    context.subscriptions.push(vscode.languages.registerCodeLensProvider({ language: 'rust', scheme: 'file' }, codeLensProvider));
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('async-inspect.start', async () => {
        await asyncInspectManager?.start();
        vscode.window.showInformationMessage('Async-Inspect: Monitoring started');
    }), vscode.commands.registerCommand('async-inspect.stop', async () => {
        await asyncInspectManager?.stop();
        vscode.window.showInformationMessage('Async-Inspect: Monitoring stopped');
    }), vscode.commands.registerCommand('async-inspect.export', async () => {
        const uri = await vscode.window.showSaveDialog({
            filters: { 'JSON': ['json'], 'CSV': ['csv'] },
            defaultUri: vscode.Uri.file('async-inspect-export.json')
        });
        if (uri) {
            await asyncInspectManager?.export(uri.fsPath);
            vscode.window.showInformationMessage(`Exported to ${uri.fsPath}`);
        }
    }), vscode.commands.registerCommand('async-inspect.clear', async () => {
        await asyncInspectManager?.clear();
        taskTreeProvider.refresh();
        statsViewProvider.refresh();
        deadlocksViewProvider.refresh();
        vscode.window.showInformationMessage('Async-Inspect: History cleared');
    }), vscode.commands.registerCommand('async-inspect.showGraph', () => {
        graphPanel_1.GraphPanel.createOrShow(context.extensionUri, asyncInspectManager);
    }), vscode.commands.registerCommand('async-inspect.showTimeline', () => {
        timelinePanel_1.TimelinePanel.createOrShow(context.extensionUri, asyncInspectManager);
    }), vscode.commands.registerCommand('async-inspect.analyzeDeadlocks', async () => {
        const deadlocks = await asyncInspectManager?.detectDeadlocks();
        if (deadlocks && deadlocks.length > 0) {
            vscode.window.showWarningMessage(`Found ${deadlocks.length} potential deadlock(s)`, 'Show Details').then(selection => {
                if (selection === 'Show Details') {
                    deadlocksViewProvider.refresh();
                }
            });
        }
        else {
            vscode.window.showInformationMessage('No deadlocks detected');
        }
    }), vscode.commands.registerCommand('async-inspect.refreshTasks', () => {
        taskTreeProvider.refresh();
        statsViewProvider.refresh();
        deadlocksViewProvider.refresh();
    }), vscode.commands.registerCommand('async-inspect.jumpToTask', (task) => {
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
    }));
    // Auto-start if configured
    const config = vscode.workspace.getConfiguration('async-inspect');
    if (config.get('autoStart')) {
        asyncInspectManager.start();
    }
    // Set up refresh interval
    const refreshInterval = config.get('refreshInterval', 500);
    setInterval(() => {
        if (asyncInspectManager?.isRunning()) {
            taskTreeProvider.refresh();
            statsViewProvider.refresh();
            deadlocksViewProvider.refresh();
            codeLensProvider.refresh();
        }
    }, refreshInterval);
}
function deactivate() {
    asyncInspectManager?.stop();
}
//# sourceMappingURL=extension.js.map