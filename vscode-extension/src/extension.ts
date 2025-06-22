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
    // Create output channel for logging
    outputChannel = vscode.window.createOutputChannel('Rez LSP');
    outputChannel.show(true);

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
                return { action: (count ?? 0) < 5 ? 1 : 2 }; // Restart if less than 5 errors, otherwise shutdown
            },
            closed: () => {
                outputChannel.appendLine(`🔌 LSP connection closed`);
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
        outputChannel.appendLine('Restarting Rez LSP Server...');
        if (client) {
            await client.stop();
        }
        client.start();
    });

    const showOutputCommand = vscode.commands.registerCommand('rezLsp.showOutputChannel', () => {
        outputChannel.show();
    });

    context.subscriptions.push(restartCommand, showOutputCommand);

    // Start the client with better error handling
    outputChannel.appendLine('🔄 Starting LSP client...');

    client.start().then(() => {
        outputChannel.appendLine('✅ Rez LSP Server started successfully');
        outputChannel.appendLine('🎉 Rez LSP Server is ready!');
    }).catch((error) => {
        outputChannel.appendLine(`❌ Failed to start Rez LSP Server: ${error}`);
        outputChannel.appendLine(`🔍 Server path: ${serverPath}`);
        outputChannel.appendLine(`📋 Error details: ${error.stack || error.message}`);

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
    if (!client) {
        return undefined;
    }
    return client.stop();
}
