#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

// Simple LSP client to test our server
class SimpleLSPClient {
    constructor() {
        this.messageId = 1;
        this.server = null;
    }

    start() {
        const serverPath = path.join(__dirname, 'target', 'debug', 'rez-lsp-server.exe');
        console.log(`Starting LSP server: ${serverPath}`);
        
        this.server = spawn(serverPath, [], {
            stdio: ['pipe', 'pipe', 'inherit']
        });

        this.server.stdout.on('data', (data) => {
            console.log('Server response:', data.toString());
        });

        this.server.on('error', (err) => {
            console.error('Server error:', err);
        });

        this.server.on('close', (code) => {
            console.log(`Server exited with code ${code}`);
        });

        // Send initialize request
        setTimeout(() => {
            this.sendInitialize();
        }, 100);
    }

    sendMessage(method, params = {}) {
        const message = {
            jsonrpc: "2.0",
            id: this.messageId++,
            method: method,
            params: params
        };

        const content = JSON.stringify(message);
        const header = `Content-Length: ${content.length}\r\n\r\n`;
        const fullMessage = header + content;

        console.log('Sending:', fullMessage);
        this.server.stdin.write(fullMessage);
    }

    sendInitialize() {
        this.sendMessage('initialize', {
            processId: process.pid,
            clientInfo: {
                name: "test-client",
                version: "1.0.0"
            },
            capabilities: {
                textDocument: {
                    completion: {
                        completionItem: {
                            snippetSupport: true
                        }
                    }
                }
            },
            workspaceFolders: [{
                uri: `file://${__dirname}`,
                name: "test-workspace"
            }]
        });

        // Send initialized notification after a delay
        setTimeout(() => {
            this.sendNotification('initialized', {});
            
            // Test completion after initialization
            setTimeout(() => {
                this.testCompletion();
            }, 500);
        }, 500);
    }

    sendNotification(method, params = {}) {
        const message = {
            jsonrpc: "2.0",
            method: method,
            params: params
        };

        const content = JSON.stringify(message);
        const header = `Content-Length: ${content.length}\r\n\r\n`;
        const fullMessage = header + content;

        console.log('Sending notification:', fullMessage);
        this.server.stdin.write(fullMessage);
    }

    testCompletion() {
        console.log('Testing completion...');
        this.sendMessage('textDocument/completion', {
            textDocument: {
                uri: "file:///test_package.py"
            },
            position: {
                line: 10,
                character: 5
            }
        });

        // Shutdown after test
        setTimeout(() => {
            this.shutdown();
        }, 2000);
    }

    shutdown() {
        console.log('Shutting down...');
        this.sendMessage('shutdown', {});
        
        setTimeout(() => {
            this.sendNotification('exit', {});
            this.server.kill();
        }, 500);
    }
}

// Run the test
const client = new SimpleLSPClient();
client.start();
