# Contributing to Synapse

Thank you for your interest in contributing to the Synapse programming language project! This document provides guidelines for contributing to the project.

## Code Style

- Follow the Rust style guide and use `rustfmt` for formatting
- Use meaningful variable and function names
- Write comprehensive documentation comments (///) for public APIs
- Include unit tests for all new functionality

## Development Workflow

1. **Task Selection**: Choose a task from the implementation plan (`plan.md`)
2. **Branch Creation**: Create a branch with a descriptive name (e.g., `feat/p0t3-asg-core-implementation`)
3. **Implementation**: Implement the feature or fix the bug
4. **Testing**: Write tests to verify your implementation
5. **Documentation**: Update documentation as needed
6. **Pull Request**: Submit a pull request with a clear description of the changes

## Commit Messages

Use the following format for commit messages:

```
feat(P0T3): Implement ASG serialization

- Add binary serialization using Protocol Buffers
- Add JSON serialization for debugging
- Add tests for round-trip serialization
```

Where the prefix indicates the type of change:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or modifying tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

And the parenthesized tag indicates the task ID from the implementation plan.

## Pull Request Process

1. Ensure all tests pass
2. Update documentation as needed
3. Request review from at least one maintainer
4. Address review comments
5. Once approved, a maintainer will merge the PR

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Assume good intentions
- Help others learn and grow

## Getting Help

If you need help with your contribution, feel free to:
- Open an issue with questions
- Ask for clarification in an existing issue
- Reach out to the maintainers
