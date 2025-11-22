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
exports.AsyncInspectCodeLensProvider = void 0;
const vscode = __importStar(require("vscode"));
class AsyncInspectCodeLensProvider {
    constructor(manager) {
        this.manager = manager;
        this._onDidChangeCodeLenses = new vscode.EventEmitter();
        this.onDidChangeCodeLenses = this._onDidChangeCodeLenses.event;
    }
    refresh() {
        this._onDidChangeCodeLenses.fire();
    }
    provideCodeLenses(document, token) {
        const config = vscode.workspace.getConfiguration('async-inspect');
        if (!config.get('showInlineStats') || !this.manager.isRunning()) {
            return [];
        }
        const codeLenses = [];
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
                const threshold = config.get('performanceThreshold', 1000);
                if (avgDuration > threshold) {
                    const warningLens = new vscode.CodeLens(new vscode.Range(position.translate(1, 0), position.translate(1, 0)), {
                        title: `⚠️  Slow function: avg ${avgDuration.toFixed(1)}ms`,
                        command: '',
                        tooltip: 'This function is slower than the performance threshold'
                    });
                    codeLenses.push(warningLens);
                }
            }
        }
        return codeLenses;
    }
}
exports.AsyncInspectCodeLensProvider = AsyncInspectCodeLensProvider;
//# sourceMappingURL=codeLensProvider.js.map