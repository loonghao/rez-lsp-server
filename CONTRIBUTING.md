# Contributing to Rez LSP Server

Thank you for your interest in contributing to the Rez LSP Server! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Git
- A text editor or IDE with Rust support

### Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/rez-lsp-server.git
   cd rez-lsp-server
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Before Making Changes

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make sure all tests pass:
   ```bash
   cargo test
   ```

### Making Changes

1. Write your code following Rust best practices
2. Add tests for new functionality
3. Update documentation if needed
4. Ensure your code is properly formatted:
   ```bash
   cargo fmt --all
   ```

5. Run clippy to catch common issues:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

### Submitting Changes

1. Commit your changes with a descriptive message:
   ```bash
   git commit -m "feat: add package version completion"
   ```

2. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

3. Create a Pull Request on GitHub

## Code Style

- Follow standard Rust formatting (use `cargo fmt`)
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Keep functions focused and reasonably sized

## Commit Message Format

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for adding or modifying tests
- `refactor:` for code refactoring
- `chore:` for maintenance tasks

## Testing

- Write unit tests for new functionality
- Ensure all existing tests continue to pass
- Test your changes with real Rez package files

## Documentation

- Update README.md if you add new features
- Add inline documentation for public APIs
- Update examples if behavior changes

## Getting Help

If you need help or have questions:

- Open an issue on GitHub
- Check existing issues and discussions
- Review the Rez documentation for context

## License

By contributing to this project, you agree that your contributions will be licensed under the Apache License 2.0.

Signed-off-by: Hal <hal.long@outlook.com>
