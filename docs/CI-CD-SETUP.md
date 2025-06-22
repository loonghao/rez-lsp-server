# CI/CD Setup Guide

This document explains how to set up the complete CI/CD pipeline for the Rez LSP Server project, including automatic building, testing, and publishing to VSCode Marketplace.

## 🔧 Prerequisites

Before setting up the CI/CD pipeline, ensure you have:

1. **GitHub Repository**: The project should be hosted on GitHub
2. **VSCode Marketplace Publisher**: A publisher account on Visual Studio Marketplace
3. **Open VSX Account**: An account on Open VSX Registry (optional but recommended)
4. **Node.js Knowledge**: Basic understanding of npm and VSCode extension development

## 🔑 Required Secrets

The CI/CD pipeline requires several secrets to be configured in your GitHub repository. Go to your repository settings → Secrets and variables → Actions to add these:

### Essential Secrets

#### `VSCODE_MARKETPLACE_TOKEN`
- **Purpose**: Publish extensions to Visual Studio Marketplace
- **How to get**:
  1. Go to [Visual Studio Marketplace Publisher Management](https://marketplace.visualstudio.com/manage)
  2. Sign in with your Microsoft account
  3. Create a publisher if you don't have one
  4. Go to "Personal Access Tokens" tab
  5. Create a new token with "Marketplace (publish)" scope
  6. Copy the token and add it as a secret

#### `OPEN_VSX_TOKEN`
- **Purpose**: Publish extensions to Open VSX Registry
- **How to get**:
  1. Go to [Open VSX Registry](https://open-vsx.org/)
  2. Sign in with your GitHub account
  3. Go to your user settings
  4. Generate a new access token
  5. Copy the token and add it as a secret

### Optional Secrets

#### `SNYK_TOKEN`
- **Purpose**: Security scanning of dependencies
- **How to get**:
  1. Sign up at [Snyk.io](https://snyk.io/)
  2. Go to Account Settings → API Token
  3. Copy your token and add it as a secret

## 🏗️ Workflow Overview

The project includes three main workflows:

### 1. CI Workflow (`.github/workflows/ci.yml`)

**Triggers**: Push to main/develop/feature branches, Pull Requests

**Jobs**:
- **Test**: Runs on Ubuntu, Windows, macOS with Rust stable/beta
- **Security Audit**: Checks for security vulnerabilities
- **VSCode Extension**: Builds and tests the VSCode extension

**Features**:
- Cross-platform testing
- Rust formatting and linting
- Extension packaging and artifact upload
- Caching for faster builds

### 2. Release Workflow (`.github/workflows/release.yml`)

**Triggers**: Git tags starting with `v*`, Manual workflow dispatch

**Jobs**:
- **Validate**: Validates version format and extracts metadata
- **Test**: Runs comprehensive tests before release
- **Build Binaries**: Creates release binaries for all platforms
- **Build VSCode Extension**: Packages the extension for release
- **Publish VSCode Extension**: Publishes to both marketplaces
- **Create GitHub Release**: Creates GitHub release with assets

**Features**:
- Multi-platform binary builds
- Automatic changelog extraction
- Pre-release support
- Dual marketplace publishing

### 3. VSCode Extension Workflow (`.github/workflows/vscode-extension.yml`)

**Triggers**: Changes to `vscode-extension/` directory

**Jobs**:
- **Lint and Test**: Extension-specific testing
- **Integration Test**: Tests LSP server integration
- **Package Extension**: Creates development packages
- **Publish Pre-release**: Publishes development builds
- **Security Scan**: Scans for vulnerabilities
- **Performance Test**: Tests with large package sets

## 🚀 Release Process

### Automatic Release (Recommended)

1. **Prepare Release**:
   ```bash
   # Update version in Cargo.toml and package.json
   # Update CHANGELOG.md
   git add .
   git commit -m "chore: prepare release v0.1.0"
   ```

2. **Create Release Tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **Automatic Process**:
   - GitHub Actions will automatically:
     - Run all tests
     - Build binaries for all platforms
     - Package VSCode extension
     - Publish to VSCode Marketplace
     - Publish to Open VSX Registry
     - Create GitHub release with assets

### Manual Release

You can also trigger releases manually:

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the version number (e.g., `0.1.0`)
4. Choose if it's a pre-release
5. Click "Run workflow"

## 🔧 Environment Configuration

### GitHub Environments

The workflows use GitHub environments for additional security:

1. **vscode-marketplace**: For stable releases
2. **vscode-marketplace-prerelease**: For development builds

To set up environments:
1. Go to repository Settings → Environments
2. Create the environments listed above
3. Add protection rules if desired (e.g., require reviews)
4. Add environment-specific secrets if needed

### Branch Protection

Recommended branch protection rules for `main`:

1. Require pull request reviews
2. Require status checks to pass
3. Require branches to be up to date
4. Include administrators in restrictions

## 📦 Extension Publishing Details

### Visual Studio Marketplace

- **URL**: https://marketplace.visualstudio.com
- **Features**: 
  - Large user base
  - Integrated with VSCode
  - Supports pre-releases
  - Detailed analytics

### Open VSX Registry

- **URL**: https://open-vsx.org
- **Features**:
  - Open source alternative
  - Used by VSCodium, Gitpod, etc.
  - Eclipse Foundation backed
  - Free to use

### Publishing Strategy

The CI/CD pipeline publishes to both registries:

1. **Stable Releases**: Published to both marketplaces
2. **Pre-releases**: Published with pre-release flag
3. **Development Builds**: Only from main branch
4. **Skip Duplicates**: Won't fail if version already exists

## 🐛 Troubleshooting

### Common Issues

#### Extension Publishing Fails

1. **Check Token Validity**: Ensure marketplace tokens are valid
2. **Version Conflicts**: Check if version already exists
3. **Namespace Issues**: Ensure Open VSX namespace is created
4. **License Issues**: Ensure extension has proper license

#### Build Failures

1. **Rust Version**: Ensure using supported Rust version
2. **Dependencies**: Check for outdated or conflicting dependencies
3. **Platform Issues**: Some tests may fail on specific platforms
4. **Cache Issues**: Clear GitHub Actions cache if needed

#### Secret Access Issues

1. **Environment Protection**: Check if environment requires approval
2. **Secret Names**: Ensure secret names match exactly
3. **Permissions**: Ensure repository has necessary permissions

### Debug Steps

1. **Check Workflow Logs**: Review detailed logs in Actions tab
2. **Test Locally**: Run build commands locally first
3. **Validate Secrets**: Test tokens manually with vsce/ovsx CLI
4. **Check Dependencies**: Ensure all dependencies are properly installed

## 📚 Additional Resources

- [VSCode Extension Publishing Guide](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
- [Open VSX Publishing Guide](https://github.com/eclipse/openvsx/wiki/Publishing-Extensions)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [HaaLeo/publish-vscode-extension Action](https://github.com/HaaLeo/publish-vscode-extension)

## 🔄 Maintenance

### Regular Tasks

1. **Update Dependencies**: Automated weekly via dependencies workflow
2. **Security Audits**: Automated on every build
3. **Token Rotation**: Rotate marketplace tokens periodically
4. **Monitor Builds**: Check for failing builds and fix promptly

### Monitoring

- **GitHub Actions**: Monitor workflow success rates
- **Marketplace Analytics**: Track extension downloads and ratings
- **Security Alerts**: Respond to Dependabot alerts promptly
- **Performance**: Monitor build times and optimize as needed

This setup provides a robust, automated CI/CD pipeline that ensures quality releases while minimizing manual work.
