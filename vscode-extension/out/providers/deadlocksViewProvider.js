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
exports.DeadlocksViewProvider = void 0;
const vscode = __importStar(require("vscode"));
class DeadlocksViewProvider {
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
    getChildren() {
        const deadlocks = this.manager.getDeadlocks();
        if (deadlocks.length === 0) {
            return Promise.resolve([
                new DeadlockTreeItem('No deadlocks detected', '✓', 'check')
            ]);
        }
        return Promise.resolve(deadlocks.map((d, idx) => new DeadlockTreeItem(`Deadlock ${idx + 1}`, d.description, 'warning', d)));
    }
}
exports.DeadlocksViewProvider = DeadlocksViewProvider;
class DeadlockTreeItem extends vscode.TreeItem {
    constructor(label, description, iconName, deadlock) {
        super(label, vscode.TreeItemCollapsibleState.None);
        this.label = label;
        this.deadlock = deadlock;
        this.description = description;
        this.iconPath = new vscode.ThemeIcon(iconName);
        this.tooltip = deadlock
            ? `Tasks involved: ${deadlock.tasks.join(', ')}\n${deadlock.description}`
            : 'No deadlocks';
    }
}
//# sourceMappingURL=deadlocksViewProvider.js.map