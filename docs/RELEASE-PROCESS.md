# Release Process Guide

This document outlines the complete release process for the Rez LSP Server project, from preparation to publication.

## 🎯 Release Types

### Stable Release
- **Version Format**: `x.y.z` (e.g., `1.0.0`, `1.2.3`)
- **Trigger**: Git tag starting with `v` (e.g., `v1.0.0`)
- **Publishing**: Both VSCode Marketplace and Open VSX Registry
- **Audience**: General users, production environments

### Pre-release
- **Version Format**: `x.y.z-prerelease` (e.g., `1.0.0-beta.1`, `1.0.0-rc.1`)
- **Trigger**: Git tag with pre-release identifier or manual workflow dispatch
- **Publishing**: Marked as pre-release on both marketplaces
- **Audience**: Early adopters, testing environments

### Development Build
- **Version Format**: `x.y.z-dev-{commit}` (e.g., `1.0.0-dev-abc1234`)
- **Trigger**: Push to `main` branch
- **Publishing**: Pre-release to marketplaces (optional)
- **Audience**: Developers, continuous integration

## 🚀 Automated Release Process

### Prerequisites

1. **Repository Setup**:
   - All tests passing on main branch
   - CHANGELOG.md updated with release notes
   - Version numbers updated in relevant files

2. **GitHub Secrets Configured**:
   - `VSCODE_MARKETPLACE_TOKEN`: Visual Studio Marketplace personal access token
   - `OPEN_VSX_TOKEN`: Open VSX Registry access token
   - `SNYK_TOKEN`: Snyk security scanning token (optional)

3. **Tools Installed** (for manual releases):
   - GitHub CLI (`gh`)
   - Git
   - PowerShell (Windows) or Bash (Linux/macOS)

### Method 1: Automatic Release (Recommended)

This is the simplest method using Git tags:

```bash
# 1. Prepare the release
git checkout main
git pull origin main

# 2. Update version and changelog
# Edit Cargo.toml, vscode-extension/package.json, and CHANGELOG.md

# 3. Commit changes
git add .
git commit -m "chore: prepare release v1.0.0"

# 4. Create and push tag
git tag v1.0.0
git push origin main
git push origin v1.0.0
```

**What happens automatically**:
1. ✅ GitHub Actions detects the tag
2. ✅ Runs comprehensive tests on all platforms
3. ✅ Builds binaries for 6 platforms (Windows, Linux, macOS × x64/ARM64)
4. ✅ Packages VSCode extension
5. ✅ Publishes to VSCode Marketplace
6. ✅ Publishes to Open VSX Registry
7. ✅ Creates GitHub release with all assets

### Method 2: Manual Workflow Trigger

Use the trigger script for more control:

```powershell
# Windows
.\scripts\trigger-release.ps1 -Version "1.0.0"
.\scripts\trigger-release.ps1 -Version "1.0.0-beta.1" -PreRelease

# Linux/macOS
./scripts/trigger-release.sh --version "1.0.0"
./scripts/trigger-release.sh --version "1.0.0-beta.1" --prerelease
```

### Method 3: GitHub Web Interface

1. Go to repository → Actions → Release workflow
2. Click "Run workflow"
3. Enter version and select options
4. Click "Run workflow"

## 📋 Release Checklist

### Pre-Release (1-2 days before)

- [ ] **Code Freeze**: No new features, only bug fixes
- [ ] **Version Bump**: Update version in all relevant files:
  - [ ] `Cargo.toml`
  - [ ] `vscode-extension/package.json`
  - [ ] `README.md` (if version is mentioned)
- [ ] **Changelog Update**: Add release notes to `CHANGELOG.md`
- [ ] **Documentation Review**: Ensure docs are up to date
- [ ] **Security Audit**: Run security scans and address issues
- [ ] **Performance Testing**: Run performance benchmarks
- [ ] **Cross-platform Testing**: Test on Windows, Linux, macOS

### Release Day

- [ ] **Final Testing**: Run full test suite locally
- [ ] **Create Release**: Use one of the automated methods above
- [ ] **Monitor CI/CD**: Watch GitHub Actions progress
- [ ] **Verify Publication**: Check both marketplaces
- [ ] **Test Installation**: Install and test the published extension
- [ ] **Update Documentation**: Update any version-specific docs
- [ ] **Announce Release**: Social media, Discord, etc.

### Post-Release (within 24 hours)

- [ ] **Monitor Issues**: Watch for bug reports
- [ ] **Update Milestones**: Close completed milestone, create next one
- [ ] **Backup**: Ensure release artifacts are backed up
- [ ] **Metrics Review**: Check download/install metrics
- [ ] **Feedback Collection**: Gather user feedback

## 🛠️ Manual Release Process

If automated release fails, you can perform manual steps:

### 1. Build Binaries

```bash
# Build for all platforms
.\scripts\build-multiplatform.ps1 -All -Release

# Or build specific platforms
.\scripts\build-multiplatform.ps1 -Targets "x86_64-pc-windows-msvc","x86_64-unknown-linux-gnu" -Release
```

### 2. Package VSCode Extension

```bash
cd vscode-extension
npm ci
npm run compile
vsce package --out rez-lsp-extension-1.0.0.vsix
```

### 3. Publish to Marketplaces

```bash
# VSCode Marketplace
vsce publish --packagePath rez-lsp-extension-1.0.0.vsix

# Open VSX Registry
ovsx publish rez-lsp-extension-1.0.0.vsix
```

### 4. Create GitHub Release

```bash
gh release create v1.0.0 \
  --title "Release v1.0.0" \
  --notes-file CHANGELOG.md \
  target/*/release/rez-lsp-server* \
  vscode-extension/rez-lsp-extension-1.0.0.vsix
```

## 🔍 Monitoring and Verification

### GitHub Actions

Monitor the release workflow:
- **URL**: `https://github.com/loonghao/rez-lsp-server/actions`
- **Expected Duration**: 20-30 minutes
- **Key Steps**: Test → Build → Package → Publish → Release

### Marketplace Verification

**VSCode Marketplace**:
- **URL**: `https://marketplace.visualstudio.com/publishers/loonghao`
- **Check**: Version number, download count, ratings
- **Test**: Install via VSCode extensions panel

**Open VSX Registry**:
- **URL**: `https://open-vsx.org/user/loonghao`
- **Check**: Version number, metadata
- **Test**: Install via compatible editors

### GitHub Release

**URL**: `https://github.com/loonghao/rez-lsp-server/releases`

**Verify**:
- [ ] Release notes are correct
- [ ] All binary assets are present (6 platforms)
- [ ] VSCode extension VSIX is attached
- [ ] Download links work
- [ ] Checksums are provided (if applicable)

## 🚨 Rollback Procedures

### If Release Fails During CI/CD

1. **Cancel Workflow**: Stop the GitHub Actions workflow
2. **Delete Tag**: `git tag -d v1.0.0 && git push origin :refs/tags/v1.0.0`
3. **Fix Issues**: Address the problems
4. **Retry**: Create tag again with same version

### If Published Release Has Critical Issues

1. **Immediate Actions**:
   - Mark release as pre-release on GitHub
   - Add warning to release notes
   - Communicate issue to users

2. **VSCode Marketplace**:
   - Cannot unpublish, but can publish hotfix
   - Use patch version (e.g., 1.0.1)

3. **Open VSX Registry**:
   - Contact registry administrators if needed
   - Publish hotfix version

4. **Hotfix Process**:
   - Create hotfix branch from release tag
   - Fix critical issues
   - Follow expedited release process
   - Communicate fix to users

## 📊 Release Metrics

Track these metrics for each release:

### Technical Metrics
- **Build Time**: Total CI/CD duration
- **Binary Sizes**: Size of each platform binary
- **Test Coverage**: Percentage of code covered by tests
- **Security Issues**: Number of vulnerabilities found/fixed

### User Metrics
- **Download Count**: VSCode Marketplace downloads
- **Install Count**: Active installations
- **Ratings**: User ratings and reviews
- **Issue Reports**: Bug reports after release

### Performance Metrics
- **Startup Time**: LSP server initialization time
- **Memory Usage**: Peak memory consumption
- **Response Time**: Average LSP response times
- **Package Discovery**: Time to scan package repositories

## 🔧 Troubleshooting

### Common Issues

**Build Failures**:
- Check Rust version compatibility
- Verify cross-compilation tools
- Review dependency conflicts

**Publishing Failures**:
- Verify marketplace tokens
- Check version conflicts
- Ensure proper licensing

**Extension Issues**:
- Test in clean VSCode environment
- Verify LSP server path
- Check extension activation

### Getting Help

- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Ask questions and share ideas
- **Discord/Slack**: Real-time community support
- **Documentation**: Check docs/ directory for guides

## 📋 Quick Reference

### One-Command Release
```bash
# Stable release
git tag v1.0.0 && git push origin v1.0.0

# Pre-release
.\scripts\trigger-release.ps1 -Version "1.0.0-beta.1" -PreRelease
```

### Emergency Hotfix
```bash
# 1. Create hotfix branch
git checkout -b hotfix/v1.0.1 v1.0.0

# 2. Fix issues and commit
git commit -m "fix: critical issue"

# 3. Release hotfix
git tag v1.0.1 && git push origin v1.0.1
```

### Verification URLs
- **Actions**: https://github.com/loonghao/rez-lsp-server/actions
- **Releases**: https://github.com/loonghao/rez-lsp-server/releases
- **VSCode**: https://marketplace.visualstudio.com/publishers/loonghao
- **Open VSX**: https://open-vsx.org/user/loonghao

This release process ensures consistent, high-quality releases while minimizing manual work and potential errors.
