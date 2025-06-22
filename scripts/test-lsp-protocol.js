#!/usr/bin/env node

/**
 * Test script for LSP protocol communication
 * This script tests basic LSP functionality by sending initialize request
 */

const { spawn } = require('child_process');
const path = require('path');

class LSPTester {
    constructor(serverPath) {
        this.serverPath = serverPath;
        this.server = null;
        this.messageId = 1;
    }

    async testLSPProtocol() {
        console.log('🧪 Testing LSP Protocol Communication');
        console.log('=====================================');

        try {
            await this.startServer();
            await this.testInitialize();
            await this.testShutdown();
            console.log('✅ All LSP protocol tests passed!');
            return true;
        } catch (error) {
            console.error('❌ LSP protocol test failed:', error.message);
            return false;
        } finally {
            this.cleanup();
        }
    }

    startServer() {
        return new Promise((resolve, reject) => {
            console.log('🚀 Starting LSP server...');
            
            this.server = spawn(this.serverPath, ['--stdio'], {
                stdio: ['pipe', 'pipe', 'pipe']
            });

            this.server.stderr.on('data', (data) => {
                // LSP server logs go to stderr, which is expected
                console.log('📝 Server log:', data.toString().trim());
            });

            this.server.on('error', (error) => {
                reject(new Error(`Failed to start server: ${error.message}`));
            });

            this.server.on('exit', (code, signal) => {
                if (code !== 0 && code !== null) {
                    reject(new Error(`Server exited with code ${code}`));
                }
            });

            // Give server time to start
            setTimeout(() => {
                if (this.server && !this.server.killed) {
                    console.log('✅ LSP server started successfully');
                    resolve();
                } else {
                    reject(new Error('Server failed to start'));
                }
            }, 1000);
        });
    }

    testInitialize() {
        return new Promise((resolve, reject) => {
            console.log('📡 Sending initialize request...');

            const initializeRequest = {
                jsonrpc: '2.0',
                id: this.messageId++,
                method: 'initialize',
                params: {
                    processId: process.pid,
                    clientInfo: {
                        name: 'test-client',
                        version: '1.0.0'
                    },
                    capabilities: {
                        textDocument: {
                            completion: {
                                completionItem: {
                                    snippetSupport: true
                                }
                            },
                            hover: {
                                contentFormat: ['markdown', 'plaintext']
                            }
                        }
                    },
                    workspaceFolders: null
                }
            };

            const message = JSON.stringify(initializeRequest);
            const header = `Content-Length: ${Buffer.byteLength(message)}\r\n\r\n`;
            
            let responseBuffer = '';
            let headerReceived = false;
            let contentLength = 0;

            const onData = (data) => {
                responseBuffer += data.toString();
                
                if (!headerReceived) {
                    const headerEnd = responseBuffer.indexOf('\r\n\r\n');
                    if (headerEnd !== -1) {
                        const headerPart = responseBuffer.substring(0, headerEnd);
                        const contentLengthMatch = headerPart.match(/Content-Length: (\d+)/);
                        
                        if (contentLengthMatch) {
                            contentLength = parseInt(contentLengthMatch[1]);
                            headerReceived = true;
                            responseBuffer = responseBuffer.substring(headerEnd + 4);
                        }
                    }
                }

                if (headerReceived && responseBuffer.length >= contentLength) {
                    this.server.stdout.removeListener('data', onData);
                    
                    try {
                        const response = JSON.parse(responseBuffer.substring(0, contentLength));
                        console.log('📨 Received initialize response');
                        
                        if (response.result && response.result.capabilities) {
                            console.log('✅ Server capabilities received');
                            resolve();
                        } else {
                            reject(new Error('Invalid initialize response'));
                        }
                    } catch (error) {
                        reject(new Error(`Failed to parse response: ${error.message}`));
                    }
                }
            };

            this.server.stdout.on('data', onData);

            // Set timeout for response
            const timeout = setTimeout(() => {
                this.server.stdout.removeListener('data', onData);
                reject(new Error('Initialize request timed out'));
            }, 5000);

            // Send the request
            this.server.stdin.write(header + message);

            // Clear timeout when resolved
            resolve = ((originalResolve) => {
                return (...args) => {
                    clearTimeout(timeout);
                    originalResolve(...args);
                };
            })(resolve);

            reject = ((originalReject) => {
                return (...args) => {
                    clearTimeout(timeout);
                    originalReject(...args);
                };
            })(reject);
        });
    }

    testShutdown() {
        return new Promise((resolve) => {
            console.log('🔄 Sending shutdown request...');

            const shutdownRequest = {
                jsonrpc: '2.0',
                id: this.messageId++,
                method: 'shutdown',
                params: null
            };

            const message = JSON.stringify(shutdownRequest);
            const header = `Content-Length: ${Buffer.byteLength(message)}\r\n\r\n`;

            this.server.stdin.write(header + message);

            // Send exit notification
            setTimeout(() => {
                const exitNotification = {
                    jsonrpc: '2.0',
                    method: 'exit',
                    params: null
                };

                const exitMessage = JSON.stringify(exitNotification);
                const exitHeader = `Content-Length: ${Buffer.byteLength(exitMessage)}\r\n\r\n`;

                this.server.stdin.write(exitHeader + exitMessage);
                console.log('✅ Shutdown sequence completed');
                resolve();
            }, 1000);
        });
    }

    cleanup() {
        if (this.server && !this.server.killed) {
            console.log('🧹 Cleaning up server process...');
            this.server.kill('SIGTERM');
            
            setTimeout(() => {
                if (!this.server.killed) {
                    this.server.kill('SIGKILL');
                }
            }, 2000);
        }
    }
}

async function main() {
    const args = process.argv.slice(2);
    
    if (args.length === 0) {
        console.error('Usage: node test-lsp-protocol.js <path-to-lsp-server>');
        process.exit(1);
    }

    const serverPath = path.resolve(args[0]);
    console.log(`Testing LSP server: ${serverPath}`);

    const tester = new LSPTester(serverPath);
    const success = await tester.testLSPProtocol();
    
    process.exit(success ? 0 : 1);
}

if (require.main === module) {
    main().catch(console.error);
}

module.exports = { LSPTester };
