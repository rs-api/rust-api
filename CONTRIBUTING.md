# Contributing to Rust API

Thank you for your interest in contributing to Rust API!

## Philosophy

Keep the core lightweight and extensible. The framework provides essential features while the ecosystem handles specialized functionality.

## Ways to Contribute

### Core Framework

- Bug fixes and performance improvements
- Documentation improvements
- Test coverage expansion
- API design feedback

### Ecosystem Packages

Building middleware and extensions is the primary way to expand Rust API's capabilities. Consider creating packages for:

- Authentication and authorization
- Rate limiting and throttling
- Compression and caching
- Logging and monitoring
- Session management
- WebSocket support
- Template engines

## Guidelines

### Code Quality

- Follow Rust idioms and best practices
- Write clear, concise inline documentation
- Ensure zero warnings under strict compilation
- Add tests for new functionality
- Keep dependencies minimal

### Pull Requests

- Create focused PRs that solve one problem
- Write clear commit messages
- Update documentation as needed
- Ensure all tests pass
- Follow existing code style

### Creating Ecosystem Packages

When building middleware or extensions:

- Use `rust-api-` prefix for package names
- Depend on specific rust-api versions
- Provide clear examples
- Document all public APIs
- Follow production-level code standards

## Development Setup

Clone the repository and run tests:

```bash
git clone https://github.com/rs-api/rust-api
cd rust-api
cargo test
```

## Questions and Support

Open an issue on GitHub for questions, bug reports, or feature requests.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
