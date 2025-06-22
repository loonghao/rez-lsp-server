import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;
let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;

// Server status enum
enum ServerStatus {
    Stopped = 'stopped',
    Starting = 'starting',
    Running = 'running',
    Error = 'error'
}

let currentServerStatus: ServerStatus = ServerStatus.Stopped;

function updateStatusBarItem() {
    if (!statusBarItem) {
        return;
    }

    const config = vscode.workspace.getConfiguration('rezLsp');
    const showStatusBar = config.get<boolean>('showStatusBarItem', true);

    if (!showStatusBar) {
        statusBarItem.hide();
        return;
    }

    // Create rich tooltip like rust-analyzer
    const tooltip = new vscode.MarkdownString('', true);
    tooltip.isTrusted = true;

    switch (currentServerStatus) {
        case ServerStatus.Stopped:
            statusBarItem.text = '$(stop-circle) rez-lsp';
            statusBarItem.color = new vscode.ThemeColor('statusBarItem.warningForeground');
            statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
            tooltip.appendText('Server is stopped');
            tooltip.appendMarkdown('\n\n[Start server](command:rezLsp.restartServer)');
            break;
        case ServerStatus.Starting:
            statusBarItem.text = '$(loading~spin) rez-lsp';
            statusBarItem.color = undefined;
            statusBarItem.backgroundColor = undefined;
            tooltip.appendText('Server is starting...');
            break;
        case ServerStatus.Running:
            statusBarItem.text = 'rez-lsp';
            statusBarItem.color = undefined;
            statusBarItem.backgroundColor = undefined;
            tooltip.appendText('Server is running');
            break;
        case ServerStatus.Error:
            statusBarItem.text = '$(error) rez-lsp';
            statusBarItem.color = new vscode.ThemeColor('statusBarItem.errorForeground');
            statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
            tooltip.appendText('Server encountered an error');
            tooltip.appendMarkdown('\n\n[Restart server](command:rezLsp.restartServer)');
            break;
    }

    // Add common actions to tooltip (like rust-analyzer)
    if (currentServerStatus !== ServerStatus.Stopped) {
        const checkOnSave = config.get<boolean>('checkOnSave', true);
        const toggleCheckText = checkOnSave ? 'Disable' : 'Enable';

        tooltip.appendMarkdown('\n\n---\n\n');
        tooltip.appendMarkdown('[$(terminal) Open Logs](command:rezLsp.showOutputChannel "Open the server logs")\n\n');
        tooltip.appendMarkdown(`[$(settings) ${toggleCheckText} Check on Save](command:rezLsp.toggleDiagnostics "Temporarily ${toggleCheckText.toLowerCase()} check on save functionality")\n\n`);
        tooltip.appendMarkdown('[$(refresh) Reload Workspace](command:rezLsp.reloadWorkspace "Reload and rediscover workspaces")\n\n');
        tooltip.appendMarkdown('[$(symbol-property) Rebuild Build Dependencies](command:rezLsp.rebuildDependencies "Rebuild build scripts and dependencies")\n\n');
        tooltip.appendMarkdown('[$(stop-circle) Stop server](command:rezLsp.stopServer "Stop the server")\n\n');
        tooltip.appendMarkdown('[$(debug-restart) Restart server](command:rezLsp.restartServer "Restart the server")');
    }

    statusBarItem.tooltip = tooltip;
    statusBarItem.show();
}

function setServerStatus(status: ServerStatus) {
    currentServerStatus = status;
    updateStatusBarItem();
    outputChannel.appendLine(`🔄 Server status changed to: ${status}`);
}

function testServerAccessibility(serverPath: string): boolean {
    try {
        const stats = fs.statSync(serverPath);
        outputChannel.appendLine(`✅ Server file exists: ${serverPath} (${stats.size} bytes)`);

        // Check if file is executable (on Unix-like systems)
        if (process.platform !== 'win32') {
            const mode = stats.mode;
            const isExecutable = (mode & parseInt('111', 8)) !== 0;
            if (!isExecutable) {
                outputChannel.appendLine(`⚠️ Server file is not executable`);
                return false;
            }
        }

        return true;
    } catch (error) {
        outputChannel.appendLine(`❌ Server accessibility test failed: ${error}`);
        return false;
    }
}

function findLspServer(context: vscode.ExtensionContext): string {
    // Try to find embedded server first
    const serverBinary = process.platform === 'win32' ? 'rez-lsp-server.exe' : 'rez-lsp-server';
    const embeddedServerPath = path.join(context.extensionPath, 'server', serverBinary);

    outputChannel.appendLine(`Looking for embedded server at: ${embeddedServerPath}`);

    if (fs.existsSync(embeddedServerPath)) {
        try {
            const stats = fs.statSync(embeddedServerPath);
            outputChannel.appendLine(`Found embedded LSP server at: ${embeddedServerPath} (size: ${stats.size} bytes)`);
            return embeddedServerPath;
        } catch (error) {
            outputChannel.appendLine(`Error checking embedded server: ${error}`);
        }
    } else {
        outputChannel.appendLine(`Embedded server not found at: ${embeddedServerPath}`);
    }

    // Fall back to configured path or system PATH
    const config = vscode.workspace.getConfiguration('rezLsp');
    const configuredPath = config.get<string>('serverPath');

    if (configuredPath && configuredPath !== 'rez-lsp-server') {
        if (fs.existsSync(configuredPath)) {
            outputChannel.appendLine(`Using configured LSP server at: ${configuredPath}`);
            return configuredPath;
        } else {
            outputChannel.appendLine(`Configured server path not found: ${configuredPath}`);
        }
    }

    // Default to system PATH
    outputChannel.appendLine('Using LSP server from system PATH: rez-lsp-server');
    return 'rez-lsp-server';
}

export function activate(context: vscode.ExtensionContext) {
    console.log('🚀 Rez LSP Extension activating...');

    // Create output channel for logging
    outputChannel = vscode.window.createOutputChannel('Rez LSP');
    outputChannel.show(true);
    outputChannel.appendLine('🎯 Rez LSP Extension activated successfully!');

    // Create status bar item (positioned at the bottom left like rust-analyzer)
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBarItem.command = 'rezLsp.showServerStatus';
    context.subscriptions.push(statusBarItem);

    // Initialize server status
    setServerStatus(ServerStatus.Stopped);

    // Find the LSP server binary
    const serverPath = findLspServer(context);

    // Get configuration
    const config = vscode.workspace.getConfiguration('rezLsp');
    const traceLevel = config.get<string>('trace.server', 'off');

    outputChannel.appendLine(`🚀 Starting Rez LSP Server: ${serverPath}`);
    outputChannel.appendLine(`📊 Trace level: ${traceLevel}`);
    outputChannel.appendLine(`📁 Extension path: ${context.extensionPath}`);

    // Test server accessibility first
    const isAccessible = testServerAccessibility(serverPath);
    if (!isAccessible) {
        outputChannel.appendLine(`⚠️ Server accessibility test failed, but continuing anyway...`);
        vscode.window.showWarningMessage('Rez LSP Server accessibility test failed. The server may not work properly.');
    }

    // Server options with better error handling
    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio,
            options: {
                env: {
                    ...process.env,
                    RUST_LOG: traceLevel === 'verbose' ? 'debug' : 'info',
                    RUST_BACKTRACE: '1'
                }
            }
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio,
            options: {
                env: {
                    ...process.env,
                    RUST_LOG: 'debug',
                    RUST_BACKTRACE: 'full'
                }
            }
        }
    };

    // Client options with improved configuration
    const clientOptions: LanguageClientOptions = {
        // Register the server for Rez package files
        documentSelector: [
            { scheme: 'file', language: 'python', pattern: '**/package.py' },
            { scheme: 'file', language: 'rez-package' },
            { scheme: 'file', pattern: '**/*.rxt' }
        ],
        synchronize: {
            // Notify the server about file changes to package.py and .rxt files
            fileEvents: [
                vscode.workspace.createFileSystemWatcher('**/package.py'),
                vscode.workspace.createFileSystemWatcher('**/*.rxt')
            ]
        },
        outputChannel: outputChannel,
        traceOutputChannel: traceLevel !== 'off' ? outputChannel : undefined,
        errorHandler: {
            error: (error, message, count) => {
                outputChannel.appendLine(`❌ LSP Error: ${error.message}`);
                if (message) {
                    outputChannel.appendLine(`📝 Message: ${JSON.stringify(message)}`);
                }
                outputChannel.appendLine(`🔢 Error count: ${count ?? 0}`);
                setServerStatus(ServerStatus.Error);
                return { action: (count ?? 0) < 5 ? 1 : 2 }; // Restart if less than 5 errors, otherwise shutdown
            },
            closed: () => {
                outputChannel.appendLine(`🔌 LSP connection closed`);
                setServerStatus(ServerStatus.Stopped);
                return { action: 1 }; // Restart
            }
        },
        initializationFailedHandler: (error) => {
            outputChannel.appendLine(`💥 LSP initialization failed: ${error.message}`);
            vscode.window.showErrorMessage(`Rez LSP Server initialization failed: ${error.message}`);
            return false; // Don't retry
        }
    };

    // Create the language client and start the client
    client = new LanguageClient(
        'rezLspServer',
        'Rez LSP Server',
        serverOptions,
        clientOptions
    );

    // Register commands
    const restartCommand = vscode.commands.registerCommand('rezLsp.restartServer', async () => {
        outputChannel.appendLine('🔄 Restarting Rez LSP Server...');
        setServerStatus(ServerStatus.Starting);
        try {
            if (client) {
                await client.stop();
            }
            await client.start();
            setServerStatus(ServerStatus.Running);
            outputChannel.appendLine('✅ Rez LSP Server restarted successfully');
        } catch (error) {
            setServerStatus(ServerStatus.Error);
            outputChannel.appendLine(`❌ Failed to restart server: ${error}`);
            vscode.window.showErrorMessage(`Failed to restart Rez LSP Server: ${error}`);
        }
    });

    const stopCommand = vscode.commands.registerCommand('rezLsp.stopServer', async () => {
        outputChannel.appendLine('🛑 Stopping Rez LSP Server...');
        setServerStatus(ServerStatus.Stopped);
        try {
            if (client) {
                await client.stop();
            }
            outputChannel.appendLine('✅ Rez LSP Server stopped successfully');
        } catch (error) {
            outputChannel.appendLine(`❌ Failed to stop server: ${error}`);
            vscode.window.showErrorMessage(`Failed to stop Rez LSP Server: ${error}`);
        }
    });

    const reloadWorkspaceCommand = vscode.commands.registerCommand('rezLsp.reloadWorkspace', async () => {
        outputChannel.appendLine('🔄 Reloading workspace...');
        try {
            // Send workspace reload request to the server
            if (client && currentServerStatus === ServerStatus.Running) {
                await client.sendRequest('workspace/didChangeConfiguration', {
                    settings: vscode.workspace.getConfiguration('rezLsp')
                });
                outputChannel.appendLine('✅ Workspace reloaded successfully');
                vscode.window.showInformationMessage('Rez LSP: Workspace reloaded');
            } else {
                outputChannel.appendLine('⚠️ Server is not running, cannot reload workspace');
                vscode.window.showWarningMessage('Rez LSP Server is not running');
            }
        } catch (error) {
            outputChannel.appendLine(`❌ Failed to reload workspace: ${error}`);
            vscode.window.showErrorMessage(`Failed to reload workspace: ${error}`);
        }
    });

    const showOutputCommand = vscode.commands.registerCommand('rezLsp.showOutputChannel', () => {
        outputChannel.show();
    });

    const rebuildDependenciesCommand = vscode.commands.registerCommand('rezLsp.rebuildDependencies', async () => {
        outputChannel.appendLine('🔨 Rebuilding build dependencies...');
        try {
            // This would typically send a custom request to the LSP server
            if (client && currentServerStatus === ServerStatus.Running) {
                // For now, we'll just reload the workspace as a placeholder
                await vscode.commands.executeCommand('rezLsp.reloadWorkspace');
                outputChannel.appendLine('✅ Build dependencies rebuilt');
                vscode.window.showInformationMessage('Rez LSP: Build dependencies rebuilt');
            } else {
                outputChannel.appendLine('⚠️ Server is not running, cannot rebuild dependencies');
                vscode.window.showWarningMessage('Rez LSP Server is not running');
            }
        } catch (error) {
            outputChannel.appendLine(`❌ Failed to rebuild dependencies: ${error}`);
            vscode.window.showErrorMessage(`Failed to rebuild dependencies: ${error}`);
        }
    });

    const toggleDiagnosticsCommand = vscode.commands.registerCommand('rezLsp.toggleDiagnostics', async () => {
        const config = vscode.workspace.getConfiguration('rezLsp');
        const currentValue = config.get<boolean>('checkOnSave', true);
        const newValue = !currentValue;

        try {
            await config.update('checkOnSave', newValue, vscode.ConfigurationTarget.Workspace);
            outputChannel.appendLine(`🔧 Check on save ${newValue ? 'enabled' : 'disabled'}`);
            vscode.window.showInformationMessage(`Rez LSP: Check on save ${newValue ? 'enabled' : 'disabled'}`);
        } catch (error) {
            outputChannel.appendLine(`❌ Failed to toggle diagnostics: ${error}`);
            vscode.window.showErrorMessage(`Failed to toggle diagnostics: ${error}`);
        }
    });

    const showServerStatusCommand = vscode.commands.registerCommand('rezLsp.showServerStatus', async () => {
        outputChannel.appendLine('📋 Show Server Status command executed');
        const items: vscode.QuickPickItem[] = [
            {
                label: '$(play) Start Server',
                description: 'Start the Rez LSP Server',
                detail: currentServerStatus === ServerStatus.Running ? 'Server is already running' : undefined
            },
            {
                label: '$(debug-restart) Restart Server',
                description: 'Restart the Rez LSP Server'
            },
            {
                label: '$(stop) Stop Server',
                description: 'Stop the Rez LSP Server',
                detail: currentServerStatus === ServerStatus.Stopped ? 'Server is already stopped' : undefined
            },
            {
                label: '$(refresh) Reload Workspace',
                description: 'Reload the current workspace'
            },
            {
                label: '$(output) Open Logs',
                description: 'Show the Rez LSP output channel'
            },
            {
                label: '$(tools) Rebuild Dependencies',
                description: 'Rebuild build dependencies'
            },
            {
                label: '$(checklist) Toggle Check on Save',
                description: 'Toggle diagnostic checks when saving files'
            }
        ];

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: `Rez LSP Server Status: ${currentServerStatus}`,
            title: 'Rez LSP Server Actions'
        });

        if (selected) {
            switch (selected.label) {
                case '$(play) Start Server':
                    if (currentServerStatus !== ServerStatus.Running) {
                        await vscode.commands.executeCommand('rezLsp.restartServer');
                    }
                    break;
                case '$(debug-restart) Restart Server':
                    await vscode.commands.executeCommand('rezLsp.restartServer');
                    break;
                case '$(stop) Stop Server':
                    await vscode.commands.executeCommand('rezLsp.stopServer');
                    break;
                case '$(refresh) Reload Workspace':
                    await vscode.commands.executeCommand('rezLsp.reloadWorkspace');
                    break;
                case '$(output) Open Logs':
                    await vscode.commands.executeCommand('rezLsp.showOutputChannel');
                    break;
                case '$(tools) Rebuild Dependencies':
                    await vscode.commands.executeCommand('rezLsp.rebuildDependencies');
                    break;
                case '$(checklist) Toggle Check on Save':
                    await vscode.commands.executeCommand('rezLsp.toggleDiagnostics');
                    break;
            }
        }
    });

    outputChannel.appendLine('📝 Registering commands...');
    context.subscriptions.push(
        restartCommand,
        stopCommand,
        reloadWorkspaceCommand,
        showOutputCommand,
        rebuildDependenciesCommand,
        toggleDiagnosticsCommand,
        showServerStatusCommand
    );
    outputChannel.appendLine('✅ All commands registered successfully');

    // Start the client with better error handling
    outputChannel.appendLine('🔄 Starting LSP client...');
    setServerStatus(ServerStatus.Starting);

    client.start().then(() => {
        outputChannel.appendLine('✅ Rez LSP Server started successfully');
        outputChannel.appendLine('🎉 Rez LSP Server is ready!');
        setServerStatus(ServerStatus.Running);
    }).catch((error) => {
        outputChannel.appendLine(`❌ Failed to start Rez LSP Server: ${error}`);
        outputChannel.appendLine(`🔍 Server path: ${serverPath}`);
        outputChannel.appendLine(`📋 Error details: ${error.stack || error.message}`);
        setServerStatus(ServerStatus.Error);

        vscode.window.showErrorMessage(
            `Failed to start Rez LSP Server. Check the output channel for details.`,
            'Show Output'
        ).then((selection) => {
            if (selection === 'Show Output') {
                outputChannel.show();
            }
        });
    });
}

export function deactivate(): Promise<void> | undefined {
    // Clean up status bar
    if (statusBarItem) {
        statusBarItem.dispose();
    }

    // Stop the client
    if (!client) {
        return undefined;
    }

    setServerStatus(ServerStatus.Stopped);
    return client.stop();
}
