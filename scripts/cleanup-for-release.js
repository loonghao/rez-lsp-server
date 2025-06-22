const fs = require('fs');
const path = require('path');

class ReleaseCleanup {
    constructor() {
        this.rootDir = path.join(__dirname, '..');
        this.filesToDelete = [];
        this.dirsToDelete = [];
    }

    cleanup() {
        console.log('🧹 Cleaning up project for release...');
        
        this.identifyFilesToDelete();
        this.deleteFiles();
        this.updateGitignore();
        this.generateCleanupReport();
        
        console.log('✅ Cleanup completed!');
    }

    identifyFilesToDelete() {
        // Root level test files
        this.filesToDelete.push(
            'test_lsp_client.js',
            'test_package.py'
        );

        // VSCode extension debug/test files
        const vscodeScriptsToDelete = [
            'debug-current-issue.js',
            'debug-lsp-protocol.js',
            'diagnose-extension.js',
            'diagnostic-report.txt',
            'final-test.js',
            'install-extension.js',
            'test-lsp-communication.js',
            'test-lsp-server.js',
            'test-minimal-lsp.js',
            'test-new-server.js',
            'test-stdio-mode.js'
        ];

        vscodeScriptsToDelete.forEach(file => {
            this.filesToDelete.push(`vscode-extension/scripts/${file}`);
        });

        // VSCode extension test directories
        this.dirsToDelete.push(
            'vscode-extension/test-files',
            'vscode-extension/test-workspace'
        );

        // VSCode extension documentation that's not needed for release
        this.filesToDelete.push(
            'vscode-extension/ICON_SETUP.md',
            'vscode-extension/SUCCESS_VERIFICATION.md',
            'vscode-extension/TROUBLESHOOTING.md'
        );

        // Old VSIX files
        this.filesToDelete.push(
            'vscode-extension/rez-lsp-extension.vsix'
        );

        // Test packages (keep only one simple example)
        this.dirsToDelete.push(
            'test_packages'
        );

        // Build artifacts that shouldn't be in repo
        this.dirsToDelete.push(
            'target/debug',
            'target/tmp'
        );
    }

    deleteFiles() {
        console.log('\n📁 Deleting unnecessary files and directories...');

        // Delete files
        this.filesToDelete.forEach(file => {
            const fullPath = path.join(this.rootDir, file);
            if (fs.existsSync(fullPath)) {
                try {
                    fs.unlinkSync(fullPath);
                    console.log(`✅ Deleted file: ${file}`);
                } catch (error) {
                    console.log(`⚠️ Could not delete file ${file}: ${error.message}`);
                }
            }
        });

        // Delete directories
        this.dirsToDelete.forEach(dir => {
            const fullPath = path.join(this.rootDir, dir);
            if (fs.existsSync(fullPath)) {
                try {
                    fs.rmSync(fullPath, { recursive: true, force: true });
                    console.log(`✅ Deleted directory: ${dir}`);
                } catch (error) {
                    console.log(`⚠️ Could not delete directory ${dir}: ${error.message}`);
                }
            }
        });
    }

    updateGitignore() {
        console.log('\n📝 Updating .gitignore...');
        
        const gitignorePath = path.join(this.rootDir, '.gitignore');
        const additionalIgnores = [
            '',
            '# Development and testing files',
            'test_lsp_client.js',
            'test_package.py',
            'vscode-extension/test-files/',
            'vscode-extension/test-workspace/',
            'vscode-extension/scripts/debug-*.js',
            'vscode-extension/scripts/test-*.js',
            'vscode-extension/scripts/diagnose-*.js',
            'vscode-extension/scripts/install-*.js',
            'vscode-extension/scripts/final-*.js',
            'vscode-extension/scripts/diagnostic-*.txt',
            '',
            '# VSIX files (except release versions)',
            'vscode-extension/*.vsix',
            '!vscode-extension/rez-lsp-extension-*.vsix',
            '',
            '# Documentation files for development',
            'vscode-extension/ICON_SETUP.md',
            'vscode-extension/SUCCESS_VERIFICATION.md',
            'vscode-extension/TROUBLESHOOTING.md'
        ];

        try {
            let gitignoreContent = '';
            if (fs.existsSync(gitignorePath)) {
                gitignoreContent = fs.readFileSync(gitignorePath, 'utf8');
            }

            // Check if our additions are already there
            if (!gitignoreContent.includes('# Development and testing files')) {
                gitignoreContent += '\n' + additionalIgnores.join('\n') + '\n';
                fs.writeFileSync(gitignorePath, gitignoreContent);
                console.log('✅ Updated .gitignore');
            } else {
                console.log('ℹ️ .gitignore already contains development ignores');
            }
        } catch (error) {
            console.log(`⚠️ Could not update .gitignore: ${error.message}`);
        }
    }

    generateCleanupReport() {
        console.log('\n📊 Cleanup Report');
        console.log('==================');
        console.log(`Files deleted: ${this.filesToDelete.length}`);
        console.log(`Directories deleted: ${this.dirsToDelete.length}`);
        
        console.log('\n📦 Project structure after cleanup:');
        console.log('├── src/                    # Rust LSP server source');
        console.log('├── vscode-extension/       # VSCode extension');
        console.log('│   ├── src/               # Extension TypeScript source');
        console.log('│   ├── scripts/           # Build and utility scripts');
        console.log('│   ├── images/            # Icons and assets');
        console.log('│   ├── server/            # LSP server binary');
        console.log('│   └── package.json       # Extension manifest');
        console.log('├── test-package/          # Example Rez package');
        console.log('├── tests/                 # Rust tests');
        console.log('├── docs/                  # Documentation');
        console.log('├── scripts/               # Build scripts');
        console.log('└── README.md              # Project documentation');
        
        console.log('\n🚀 Ready for release!');
        console.log('Next steps:');
        console.log('1. Review changes: git status');
        console.log('2. Test the extension: npm run package');
        console.log('3. Commit changes: git add . && git commit -m "Clean up for release"');
        console.log('4. Create PR for release');
    }
}

if (require.main === module) {
    const cleanup = new ReleaseCleanup();
    cleanup.cleanup();
}
