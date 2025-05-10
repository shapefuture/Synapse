# Contributing to Synapse

Thank you for your interest in contributing to the Synapse programming language! This document provides guidelines and information for contributors.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please treat everyone with respect and help us maintain a positive community.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork to your local machine
3. Add the original repository as an upstream remote
4. Create a new branch for your changes

## Development Workflow

### Setting Up Your Environment

Ensure you have the following installed:
- Rust toolchain (stable)
- Protobuf compiler (protoc)
- Git

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test
```

### Code Style

We follow the Rust standard code style:
- Use `cargo fmt` to format your code
- Run `cargo clippy` to check for common issues
- Write comments for public API using rustdoc format
- Use meaningful variable and function names

## Pull Request Process

1. Update your branch with the latest changes from upstream
2. Ensure your code builds and passes all tests
3. Write tests for your changes
4. Update documentation if necessary
5. Submit a pull request with a clear description of your changes

Your pull request should:
- Have a clear title and description
- Reference any related issues
- Explain what problem it solves and how it does so
- Include any necessary tests

## Commit Message Guidelines

We use a simplified version of Conventional Commits:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `test`: Adding or modifying tests
- `refactor`: Code changes that neither fix bugs nor add features
- `chore`: Changes to the build process or auxiliary tools

Example:
```
feat(parser): Add support for effect syntax
```

## Documentation

Please keep documentation up to date with your changes:
- Update README.md if necessary
- Add entries to DESIGN_LOG.md for significant design decisions
- Update or add formal specifications in docs/ when changing language semantics

## Issues and Feature Requests

Feel free to submit issues and enhancement requests. Please provide as much detail as possible:
- For bugs: Steps to reproduce, expected behavior, actual behavior
- For features: Use cases, expected behavior, alternatives considered

## Licensing

By contributing, you agree that your contributions will be licensed under the project's MIT License.

## Questions?

If you have any questions, please create an issue or reach out to the maintainers.

Thank you for contributing to Synapse!