const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class PreReleaseCheck {
    constructor() {
        this.rootDir = path.join(__dirname, '..');
        this.issues = [];
        this.warnings = [];
    }

    async check() {
        console.log('🔍 Pre-release Check');
        console.log('====================');
        
        this.checkProjectStructure();
        this.checkCargoToml();
        this.checkPackageJson();
        this.checkDocumentation();
        this.checkBuildArtifacts();
        this.checkGitStatus();
        
        this.generateReport();
    }

    checkProjectStructure() {
        console.log('\n📁 Checking project structure...');
        
        const requiredFiles = [
            'Cargo.toml',
            'README.md',
            'LICENSE',
            'src/main.rs',
            'src/lib.rs',
            'vscode-extension/package.json',
            'vscode-extension/README.md',
            'vscode-extension/src/extension.ts'
        ];

        const requiredDirs = [
            'src',
            'vscode-extension/src',
            'vscode-extension/images',
            'vscode-extension/server'
        ];

        requiredFiles.forEach(file => {
            const fullPath = path.join(this.rootDir, file);
            if (!fs.existsSync(fullPath)) {
                this.issues.push(`Missing required file: ${file}`);
            } else {
                console.log(`✅ ${file}`);
            }
        });

        requiredDirs.forEach(dir => {
            const fullPath = path.join(this.rootDir, dir);
            if (!fs.existsSync(fullPath)) {
                this.issues.push(`Missing required directory: ${dir}`);
            } else {
                console.log(`✅ ${dir}/`);
            }
        });

        // Check for unwanted files
        const unwantedFiles = [
            'test_lsp_client.js',
            'test_package.py',
            'vscode-extension/TROUBLESHOOTING.md',
            'vscode-extension/SUCCESS_VERIFICATION.md'
        ];

        unwantedFiles.forEach(file => {
            const fullPath = path.join(this.rootDir, file);
            if (fs.existsSync(fullPath)) {
                this.warnings.push(`Unwanted file still exists: ${file}`);
            }
        });
    }

    checkCargoToml() {
        console.log('\n📦 Checking Cargo.toml...');
        
        const cargoPath = path.join(this.rootDir, 'Cargo.toml');
        if (!fs.existsSync(cargoPath)) {
            this.issues.push('Cargo.toml not found');
            return;
        }

        const cargoContent = fs.readFileSync(cargoPath, 'utf8');
        
        // Check version
        const versionMatch = cargoContent.match(/version\s*=\s*"([^"]+)"/);
        if (versionMatch) {
            console.log(`✅ Version: ${versionMatch[1]}`);
        } else {
            this.issues.push('No version found in Cargo.toml');
        }

        // Check required fields
        const requiredFields = ['name', 'description', 'license', 'authors'];
        requiredFields.forEach(field => {
            if (cargoContent.includes(`${field} =`)) {
                console.log(`✅ ${field} field present`);
            } else {
                this.issues.push(`Missing ${field} field in Cargo.toml`);
            }
        });
    }

    checkPackageJson() {
        console.log('\n📦 Checking VSCode extension package.json...');
        
        const packagePath = path.join(this.rootDir, 'vscode-extension', 'package.json');
        if (!fs.existsSync(packagePath)) {
            this.issues.push('VSCode extension package.json not found');
            return;
        }

        try {
            const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
            
            // Check required fields
            const requiredFields = ['name', 'version', 'description', 'publisher', 'engines'];
            requiredFields.forEach(field => {
                if (packageJson[field]) {
                    console.log(`✅ ${field}: ${typeof packageJson[field] === 'object' ? JSON.stringify(packageJson[field]) : packageJson[field]}`);
                } else {
                    this.issues.push(`Missing ${field} field in package.json`);
                }
            });

            // Check icon
            if (packageJson.icon) {
                const iconPath = path.join(this.rootDir, 'vscode-extension', packageJson.icon);
                if (fs.existsSync(iconPath)) {
                    console.log(`✅ Icon file exists: ${packageJson.icon}`);
                } else {
                    this.issues.push(`Icon file not found: ${packageJson.icon}`);
                }
            } else {
                this.warnings.push('No icon specified in package.json');
            }

        } catch (error) {
            this.issues.push(`Invalid JSON in package.json: ${error.message}`);
        }
    }

    checkDocumentation() {
        console.log('\n📚 Checking documentation...');
        
        // Check README files
        const readmeFiles = [
            'README.md',
            'vscode-extension/README.md'
        ];

        readmeFiles.forEach(file => {
            const fullPath = path.join(this.rootDir, file);
            if (fs.existsSync(fullPath)) {
                const content = fs.readFileSync(fullPath, 'utf8');
                if (content.length > 100) {
                    console.log(`✅ ${file} (${content.length} chars)`);
                } else {
                    this.warnings.push(`${file} seems too short (${content.length} chars)`);
                }
            } else {
                this.issues.push(`Missing ${file}`);
            }
        });

        // Check LICENSE
        const licensePath = path.join(this.rootDir, 'LICENSE');
        if (fs.existsSync(licensePath)) {
            console.log('✅ LICENSE file exists');
        } else {
            this.issues.push('Missing LICENSE file');
        }
    }

    checkBuildArtifacts() {
        console.log('\n🔨 Checking build artifacts...');
        
        // Check LSP server binary
        const serverPath = path.join(this.rootDir, 'vscode-extension', 'server', 'rez-lsp-server.exe');
        if (fs.existsSync(serverPath)) {
            const stats = fs.statSync(serverPath);
            console.log(`✅ LSP server binary (${(stats.size / 1024 / 1024).toFixed(2)} MB)`);
        } else {
            this.issues.push('LSP server binary not found in vscode-extension/server/');
        }

        // Check compiled TypeScript
        const outPath = path.join(this.rootDir, 'vscode-extension', 'out');
        if (fs.existsSync(outPath)) {
            console.log('✅ TypeScript compiled output exists');
        } else {
            this.warnings.push('TypeScript not compiled (run npm run compile)');
        }
    }

    checkGitStatus() {
        console.log('\n📋 Checking git status...');
        
        try {
            const status = execSync('git status --porcelain', { 
                cwd: this.rootDir, 
                encoding: 'utf8' 
            });
            
            if (status.trim()) {
                this.warnings.push('Uncommitted changes detected');
                console.log('⚠️ Uncommitted changes:');
                console.log(status);
            } else {
                console.log('✅ Working directory clean');
            }
        } catch (error) {
            this.warnings.push('Could not check git status');
        }
    }

    generateReport() {
        console.log('\n📊 Pre-release Report');
        console.log('=====================');
        
        if (this.issues.length === 0 && this.warnings.length === 0) {
            console.log('🎉 All checks passed! Ready for release.');
        } else {
            if (this.issues.length > 0) {
                console.log('\n❌ Issues that must be fixed:');
                this.issues.forEach(issue => console.log(`   • ${issue}`));
            }
            
            if (this.warnings.length > 0) {
                console.log('\n⚠️ Warnings (consider addressing):');
                this.warnings.forEach(warning => console.log(`   • ${warning}`));
            }
        }

        console.log('\n🚀 Next steps for release:');
        console.log('1. Fix any issues listed above');
        console.log('2. Test the extension: cd vscode-extension && npm run package');
        console.log('3. Commit all changes: git add . && git commit -m "Prepare for release"');
        console.log('4. Create release PR');
        console.log('5. Tag release: git tag v0.1.0 && git push origin v0.1.0');
    }
}

if (require.main === module) {
    const checker = new PreReleaseCheck();
    checker.check().catch(console.error);
}
