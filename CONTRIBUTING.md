# Contributing to Fechatter

First off, thank you for considering contributing to Fechatter! It's people like you that make Fechatter such a great tool. 🎉

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Your First Code Contribution](#your-first-code-contribution)
  - [Pull Requests](#pull-requests)
- [Development Setup](#development-setup)
- [Style Guidelines](#style-guidelines)
  - [Git Commit Messages](#git-commit-messages)
  - [Rust Style Guide](#rust-style-guide)
  - [JavaScript/Vue Style Guide](#javascriptvue-style-guide)
  - [Documentation Style Guide](#documentation-style-guide)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [conduct@fechatter.io](mailto:conduct@fechatter.io).

## Getting Started

Fechatter is built with:

- **Backend**: Rust (using Axum, Tokio, etc.)
- **Frontend**: Vue.js 3 with TypeScript
- **Infrastructure**: PostgreSQL, Redis, NATS, Meilisearch
- **Deployment**: Docker, Kubernetes

Before you begin:

1. Read our [README](README.md) to understand the project
2. Check out our [Development Guide](./fechatter_server/docs/DEVELOPMENT_GUIDE.md)
3. Set up your development environment

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible.

**Bug Report Template:**

```markdown
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Environment:**
 - OS: [e.g. macOS, Ubuntu]
 - Browser [e.g. chrome, safari]
 - Version [e.g. 22]

**Additional context**
Add any other context about the problem here.
```

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use a clear and descriptive title**
- **Provide a step-by-step description** of the suggested enhancement
- **Provide specific examples** to demonstrate the steps
- **Describe the current behavior** and **explain which behavior you expected to see instead**
- **Explain why this enhancement would be useful**

### Your First Code Contribution

Unsure where to begin contributing? You can start by looking through these issues:

- [Good First Issues](https://github.com/yourusername/fechatter/labels/good%20first%20issue) - issues which should only require a few lines of code
- [Help Wanted](https://github.com/yourusername/fechatter/labels/help%20wanted) - issues which should be a bit more involved than beginner issues

### Pull Requests

1. Fork the repo and create your branch from `main`
2. If you've added code that should be tested, add tests
3. If you've changed APIs, update the documentation
4. Ensure the test suite passes
5. Make sure your code lints
6. Issue that pull request!

## Development Setup

### Prerequisites

```bash
# Required tools
- Rust 1.70+
- Node.js 18+
- Docker & Docker Compose
- Git
```

### Local Development

```bash
# Clone your fork
git clone https://github.com/yourusername/fechatter.git
cd fechatter

# Add upstream remote
git remote add upstream https://github.com/originalowner/fechatter.git

# Install dependencies
make setup-dev

# Start development environment
make dev

# Run tests
make test
```

### Testing

```bash
# Run all tests
make test

# Run specific test suites
make test-unit
make test-integration
make test-e2e

# Run linting
make lint

# Format code
make fmt
```

## Style Guidelines

### Git Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

**Commit Message Format:**

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools

**Examples:**

```
feat(chat): add emoji reactions to messages

- Added emoji picker component
- Implemented reaction storage in database
- Added real-time reaction updates via SSE

Closes #123
```

### Rust Style Guide

We follow the standard Rust style guidelines:

```rust
// Use rustfmt for formatting
cargo fmt

// Use clippy for linting
cargo clippy -- -D warnings

// Example code style
pub async fn send_message(
    user_id: UserId,
    chat_id: ChatId,
    content: String,
) -> Result<Message, Error> {
    // Validate input
    if content.is_empty() {
        return Err(Error::EmptyMessage);
    }
  
    // Process message
    let message = Message::new(user_id, chat_id, content);
  
    // Save to database
    message_repository.save(&message).await?;
  
    Ok(message)
}
```

### JavaScript/Vue Style Guide

We use ESLint and Prettier for JavaScript/TypeScript:

```javascript
// Use composition API for Vue components
<script setup lang="ts">
import { ref, computed } from 'vue'

interface Props {
  userId: string
  chatId: string
}

const props = defineProps<Props>()
const message = ref('')

const sendMessage = async () => {
  if (!message.value.trim()) return
  
  await api.sendMessage({
    userId: props.userId,
    chatId: props.chatId,
    content: message.value
  })
  
  message.value = ''
}
</script>
```

### Documentation Style Guide

- Use Markdown for all documentation
- Include code examples where relevant
- Keep language clear and concise
- Update documentation with code changes
- Include diagrams for complex concepts

## Project Structure

```
fechatter/
├── fechatter_server/     # Rust backend services
│   ├── src/
│   │   ├── services/    # Microservices
│   │   ├── models/      # Data models
│   │   └── handlers/    # API handlers
│   └── tests/
├── fechatter_frontend/   # Vue.js frontend
│   ├── src/
│   │   ├── components/  # Vue components
│   │   ├── views/       # Page views
│   │   └── stores/      # Pinia stores
│   └── tests/
└── docs/                # Documentation
```

## Recognition

Contributors will be recognized in our:

- [Contributors Page](https://github.com/yourusername/fechatter/graphs/contributors)
- Release notes
- Project documentation

## Questions?

Feel free to contact the project maintainers if you have any questions. We're here to help!

Thank you for contributing to Fechatter! 🚀
