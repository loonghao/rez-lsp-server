const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

class ManualRelease {
    constructor() {
        this.rootDir = path.join(__dirname, '..');
        this.version = null;
        this.isPrerelease = false;
    }

    async release() {
        console.log('🚀 Manual Release Process');
        console.log('=========================');
        
        this.checkPrerequisites();
        this.getVersionInput();
        this.validateVersion();
        this.runTests();
        this.buildServer();
        this.buildExtension();
        this.createTag();
        this.showNextSteps();
    }

    checkPrerequisites() {
        console.log('\n🔍 Checking prerequisites...');
        
        // Check if we're on main branch
        try {
            const branch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
            if (branch !== 'main') {
                console.log(`⚠️ Warning: You're on branch '${branch}', not 'main'`);
            } else {
                console.log('✅ On main branch');
            }
        } catch (error) {
            console.log('⚠️ Could not determine current branch');
        }

        // Check for uncommitted changes
        try {
            const status = execSync('git status --porcelain', { encoding: 'utf8' });
            if (status.trim()) {
                console.log('❌ Uncommitted changes detected:');
                console.log(status);
                process.exit(1);
            } else {
                console.log('✅ Working directory clean');
            }
        } catch (error) {
            console.log('⚠️ Could not check git status');
        }

        // Check required tools
        const tools = ['cargo', 'npm', 'git'];
        tools.forEach(tool => {
            try {
                execSync(`${tool} --version`, { stdio: 'ignore' });
                console.log(`✅ ${tool} available`);
            } catch (error) {
                console.log(`❌ ${tool} not found`);
                process.exit(1);
            }
        });
    }

    getVersionInput() {
        console.log('\n📝 Version Information');
        console.log('Current version in Cargo.toml:');
        
        const cargoToml = fs.readFileSync(path.join(this.rootDir, 'Cargo.toml'), 'utf8');
        const versionMatch = cargoToml.match(/version\s*=\s*"([^"]+)"/);
        if (versionMatch) {
            console.log(`   ${versionMatch[1]}`);
        }

        // For manual release, we'll use the version from Cargo.toml
        this.version = versionMatch ? versionMatch[1] : '0.1.0';
        
        // Check if it's a prerelease
        this.isPrerelease = this.version.includes('alpha') || 
                           this.version.includes('beta') || 
                           this.version.includes('rc');
        
        console.log(`📦 Release version: ${this.version}`);
        console.log(`🏷️ Prerelease: ${this.isPrerelease}`);
    }

    validateVersion() {
        console.log('\n✅ Validating version format...');
        
        const versionRegex = /^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+(\.[0-9]+)?)?$/;
        if (!versionRegex.test(this.version)) {
            console.log(`❌ Invalid version format: ${this.version}`);
            process.exit(1);
        }
        
        console.log('✅ Version format valid');
    }

    runTests() {
        console.log('\n🧪 Running tests...');
        
        try {
            execSync('cargo test --lib', { stdio: 'inherit', cwd: this.rootDir });
            console.log('✅ Tests passed');
        } catch (error) {
            console.log('❌ Tests failed');
            process.exit(1);
        }
    }

    buildServer() {
        console.log('\n🔨 Building LSP server...');
        
        try {
            execSync('cargo build --release', { stdio: 'inherit', cwd: this.rootDir });
            console.log('✅ LSP server built successfully');
        } catch (error) {
            console.log('❌ LSP server build failed');
            process.exit(1);
        }
    }

    buildExtension() {
        console.log('\n📦 Building VSCode extension...');
        
        const extensionDir = path.join(this.rootDir, 'vscode-extension');
        
        try {
            // Update version in package.json
            execSync(`npm version ${this.version} --no-git-tag-version`, { 
                stdio: 'inherit', 
                cwd: extensionDir 
            });
            
            // Install dependencies
            execSync('npm ci', { stdio: 'inherit', cwd: extensionDir });
            
            // Build extension
            execSync('npm run build', { stdio: 'inherit', cwd: extensionDir });
            
            // Package extension
            const packageCmd = this.isPrerelease 
                ? 'npx vsce package --pre-release'
                : 'npx vsce package';
            
            execSync(packageCmd, { stdio: 'inherit', cwd: extensionDir });
            
            console.log('✅ VSCode extension built successfully');
        } catch (error) {
            console.log('❌ VSCode extension build failed');
            process.exit(1);
        }
    }

    createTag() {
        console.log('\n🏷️ Creating git tag...');
        
        const tagName = `v${this.version}`;
        
        try {
            // Check if tag already exists
            try {
                execSync(`git rev-parse ${tagName}`, { stdio: 'ignore' });
                console.log(`⚠️ Tag ${tagName} already exists`);
                return;
            } catch (error) {
                // Tag doesn't exist, which is good
            }
            
            // Create tag
            execSync(`git tag -a ${tagName} -m "Release ${tagName}"`, { 
                stdio: 'inherit', 
                cwd: this.rootDir 
            });
            
            console.log(`✅ Created tag: ${tagName}`);
        } catch (error) {
            console.log('❌ Failed to create tag');
            process.exit(1);
        }
    }

    showNextSteps() {
        console.log('\n🎉 Release prepared successfully!');
        console.log('================================');
        console.log('');
        console.log('📋 Next steps:');
        console.log(`1. Push the tag: git push origin v${this.version}`);
        console.log('2. This will trigger the GitHub Actions release workflow');
        console.log('3. The workflow will:');
        console.log('   - Build binaries for all platforms');
        console.log('   - Build VSCode extensions for all platforms');
        console.log('   - Create a GitHub release');
        console.log('   - Publish to VS Code Marketplace');
        console.log('   - Publish to Open VSX Registry');
        console.log('');
        console.log('🔍 Monitor the release at:');
        console.log('   https://github.com/loonghao/rez-lsp-server/actions');
        console.log('');
        console.log('📦 Files ready for release:');
        console.log(`   - LSP server binary: target/release/rez-lsp-server${process.platform === 'win32' ? '.exe' : ''}`);
        console.log(`   - VSCode extension: vscode-extension/rez-lsp-extension-${this.version}.vsix`);
        console.log('');
        console.log('⚠️ If you need to cancel:');
        console.log(`   git tag -d v${this.version}`);
    }
}

if (require.main === module) {
    const release = new ManualRelease();
    release.release().catch(console.error);
}
