const fs = require('fs');
const path = require('path');

console.log('📦 Copying LSP server binary...');

// Determine the server binary name based on platform
const serverBinary = process.platform === 'win32' ? 'rez-lsp-server.exe' : 'rez-lsp-server';

// Paths
const sourceDir = path.join(__dirname, '..', '..', 'target', 'release');
const sourcePath = path.join(sourceDir, serverBinary);
const destDir = path.join(__dirname, '..', 'server');
const destPath = path.join(destDir, serverBinary);

// Create destination directory
if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
}

// Check if source exists
if (!fs.existsSync(sourcePath)) {
    console.log(`⚠️  LSP server binary not found at: ${sourcePath}`);
    console.log('   Building Rust project first...');
    
    // Try to build the Rust project
    const { execSync } = require('child_process');
    try {
        process.chdir(path.join(__dirname, '..', '..'));
        console.log('🦀 Building Rust LSP server...');
        execSync('cargo build --release', { stdio: 'inherit' });
        
        // Check again
        if (!fs.existsSync(sourcePath)) {
            console.error(`❌ Failed to build LSP server binary`);
            process.exit(1);
        }
    } catch (error) {
        console.error(`❌ Failed to build Rust project: ${error.message}`);
        console.log('');
        console.log('💡 Manual build steps:');
        console.log('   1. cd ..');
        console.log('   2. cargo build --release');
        console.log('   3. npm run copy-server');
        process.exit(1);
    }
}

// Copy the binary
try {
    fs.copyFileSync(sourcePath, destPath);
    
    // Make executable on Unix systems
    if (process.platform !== 'win32') {
        fs.chmodSync(destPath, 0o755);
    }
    
    const stats = fs.statSync(destPath);
    const sizeMB = (stats.size / (1024 * 1024)).toFixed(2);
    
    console.log(`✅ Copied ${serverBinary} to extension (${sizeMB} MB)`);
    console.log(`   Source: ${sourcePath}`);
    console.log(`   Destination: ${destPath}`);
} catch (error) {
    console.error(`❌ Failed to copy server binary: ${error.message}`);
    process.exit(1);
}
