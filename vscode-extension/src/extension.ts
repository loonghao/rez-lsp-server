import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    // Create output channel for logging
    outputChannel = vscode.window.createOutputChannel('Rez LSP');

    // Get configuration
    const config = vscode.workspace.getConfiguration('rezLsp');
    const serverPath = config.get<string>('serverPath', 'rez-lsp-server');
    const traceLevel = config.get<string>('trace.server', 'off');

    outputChannel.appendLine(`Starting Rez LSP Server: ${serverPath}`);

    // Server options
    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        // Register the server for Rez package files
        documentSelector: [
            { scheme: 'file', language: 'python', pattern: '**/package.py' },
            { scheme: 'file', language: 'rez-package' }
        ],
        synchronize: {
            // Notify the server about file changes to package.py files
            fileEvents: vscode.workspace.createFileSystemWatcher('**/package.py')
        },
        outputChannel: outputChannel,
        traceOutputChannel: traceLevel !== 'off' ? outputChannel : undefined
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

    // Start the client
    client.start().then(() => {
        outputChannel.appendLine('Rez LSP Server started successfully');
    }).catch((error) => {
        outputChannel.appendLine(`Failed to start Rez LSP Server: ${error}`);
        vscode.window.showErrorMessage(`Failed to start Rez LSP Server: ${error}`);
    });
}

export function deactivate(): Promise<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
