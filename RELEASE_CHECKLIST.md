# 🚀 Release Checklist

## Pre-Release Preparation

### 📋 Code Quality
- [ ] All tests pass: `cargo test`
- [ ] Code compiles without warnings: `cargo build --release`
- [ ] Extension builds successfully: `cd vscode-extension && npm run package`
- [ ] No linting errors: `cargo clippy`
- [ ] Code is formatted: `cargo fmt --check`

### 📚 Documentation
- [ ] README.md is up to date
- [ ] CHANGELOG.md includes new version
- [ ] VSCode extension README.md is current
- [ ] All features are documented
- [ ] Installation instructions are correct

### 🧪 Testing
- [ ] Manual testing completed
- [ ] Extension works in VSCode
- [ ] LSP server responds correctly
- [ ] All advertised features work
- [ ] Cross-platform compatibility verified

### 🔧 Configuration
- [ ] Version numbers are consistent:
  - [ ] Cargo.toml version
  - [ ] vscode-extension/package.json version
  - [ ] CHANGELOG.md version
- [ ] GitHub repository settings are correct
- [ ] Secrets are configured:
  - [ ] `VSCODE_MARKETPLACE_TOKEN`
  - [ ] `OPEN_VSX_TOKEN`

## Release Process

### 🏷️ Version Management
- [ ] Determine version number (semantic versioning)
- [ ] Update version in all files
- [ ] Update CHANGELOG.md with release notes
- [ ] Commit version changes

### 🚀 Release Execution

#### Option 1: Automated Release (Recommended)
1. [ ] Create and push git tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
2. [ ] Monitor GitHub Actions workflow
3. [ ] Verify release artifacts
4. [ ] Test published extension

#### Option 2: Manual Release
1. [ ] Run release script: `node scripts/release-manual.js`
2. [ ] Follow script instructions
3. [ ] Push tag to trigger CI/CD

### 📦 Release Artifacts

The release should include:
- [ ] **Server Binaries**:
  - [ ] `rez-lsp-server-linux-x64.tar.gz`
  - [ ] `rez-lsp-server-linux-arm64.tar.gz`
  - [ ] `rez-lsp-server-windows-x64.exe.zip`
  - [ ] `rez-lsp-server-windows-arm64.exe.zip`
  - [ ] `rez-lsp-server-macos-x64.tar.gz`
  - [ ] `rez-lsp-server-macos-arm64.tar.gz`

- [ ] **VSCode Extensions**:
  - [ ] `rez-lsp-extension-win32-x64-{version}.vsix`
  - [ ] `rez-lsp-extension-linux-x64-{version}.vsix`
  - [ ] `rez-lsp-extension-darwin-x64-{version}.vsix`
  - [ ] `rez-lsp-extension-darwin-arm64-{version}.vsix`

### 🌐 Publication Verification
- [ ] **GitHub Release**:
  - [ ] Release created with correct tag
  - [ ] All artifacts attached
  - [ ] Release notes are accurate
  - [ ] Links work correctly

- [ ] **VS Code Marketplace**:
  - [ ] Extension published successfully
  - [ ] All platform variants available
  - [ ] Extension page looks correct
  - [ ] Installation works from marketplace

- [ ] **Open VSX Registry**:
  - [ ] Extension published successfully
  - [ ] Available for alternative editors

## Post-Release

### 📢 Communication
- [ ] Update project README with new version
- [ ] Announce release (if applicable):
  - [ ] GitHub Discussions
  - [ ] Social media
  - [ ] Rez community channels

### 🔍 Monitoring
- [ ] Monitor for installation issues
- [ ] Check error reports
- [ ] Respond to user feedback
- [ ] Monitor download statistics

### 🛠️ Maintenance
- [ ] Create milestone for next version
- [ ] Update project roadmap
- [ ] Plan next release cycle

## Rollback Plan

If issues are discovered after release:

1. [ ] **Immediate Actions**:
   - [ ] Document the issue
   - [ ] Assess impact and severity
   - [ ] Communicate with users if necessary

2. [ ] **Quick Fix** (for minor issues):
   - [ ] Create hotfix branch
   - [ ] Fix the issue
   - [ ] Release patch version

3. [ ] **Rollback** (for major issues):
   - [ ] Unpublish from marketplaces (if possible)
   - [ ] Revert to previous stable version
   - [ ] Communicate rollback to users

## Release Notes Template

```markdown
## [0.1.0] - 2025-01-XX

### Added
- Initial release of Rez LSP Server
- VSCode extension with full LSP support
- Code completion for Rez package files
- Hover information for Rez keywords
- Syntax validation and diagnostics
- Go to definition for package dependencies
- File icons for .py and .rxt files

### Features
- Cross-platform support (Windows, macOS, Linux)
- Multi-architecture binaries (x64, ARM64)
- Automatic server binary bundling
- Comprehensive error handling

### Technical
- Built with Rust and tower-lsp
- TypeScript VSCode extension
- Automated CI/CD with GitHub Actions
- Multi-platform extension packaging
```

## Emergency Contacts

- **Repository Owner**: @loonghao
- **CI/CD Issues**: Check GitHub Actions logs
- **Marketplace Issues**: VS Code Marketplace support
- **Community Support**: GitHub Issues

---

**Remember**: Take your time with releases. It's better to delay a release than to ship broken software. 🎯
