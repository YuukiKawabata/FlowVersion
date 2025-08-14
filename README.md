# FlowVersion

Next-generation version control system with intent-based commits and AI assistance.

## Overview

FlowVersion revolutionizes version control by focusing on the "why" behind code changes through intent-based commits, AI-powered analysis, and innovative features like quantum branching.

## Features (MVP)

- **Intent-based commits**: Record not just what changed, but why it changed
- **Repository management**: Initialize and manage FlowVersion repositories
- **Basic version control**: File tracking, staging, and commit history
- **AI integration**: Optional AI-powered analysis and suggestions

## Installation

```bash
# Install from source (requires Rust)
cargo install --path .

# Or build locally
cargo build --release
```

## Quick Start

```bash
# Initialize a new repository
flow init --name my-project

# Add files to staging
flow add src/main.rs --intention "Add main application entry point"

# Create an intent-based commit
flow commit --intention "Implement user authentication" \
           --context "Required for security compliance" \
           --impact "Affects login flow and user management" \
           --confidence 0.9

# View commit history
flow log --intentions
```

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── error.rs             # Error handling
├── cli/                 # Command line interface
│   ├── commands.rs      # Command routing
│   └── handlers.rs      # Command handlers
├── core/                # Core functionality
│   ├── intention.rs     # Intention data model
│   ├── commit.rs        # Commit data model
│   ├── objects.rs       # Object storage primitives
│   ├── repository.rs    # Repository operations
│   └── stream.rs        # Stream (branch) management
├── storage/             # Storage layer
│   ├── object_store.rs  # Object storage implementation
│   ├── index.rs         # Staging area management
│   └── config.rs        # Configuration management
└── utils/               # Utilities
    ├── hash.rs          # Hashing functions
    ├── diff.rs          # Diff algorithms
    └── fs.rs            # File system operations
```

## Development Status

This is the MVP implementation focusing on core functionality:

- ✅ Project structure and dependencies
- ✅ Core data models (Intention, Commit, Objects)
- ✅ Repository initialization and management
- ✅ Basic CLI interface
- ✅ File staging and commit operations
- ✅ Object storage system
- ⏳ Testing and validation
- ⏳ Documentation
- ❌ AI integration
- ❌ Advanced features (quantum branching, real-time collaboration)

## Architecture

FlowVersion follows a layered architecture:

1. **CLI Layer**: User interface and command handling
2. **Application Layer**: Business logic and command processing
3. **Core Layer**: Domain models and repository operations
4. **Storage Layer**: Data persistence and object management
5. **Infrastructure Layer**: File system, network, and external integrations

## Contributing

This is currently a personal project in early development. Future contributions will be welcomed once the core functionality is stable.

## License

MIT License - see LICENSE file for details.

## Roadmap

### Phase 1 (Current - MVP)
- Basic repository operations
- Intent-based commits
- CLI interface
- Local storage

### Phase 2 (Future)
- AI integration (OpenAI/Claude)
- Git migration tools
- Advanced merge strategies
- IDE integrations

### Phase 3 (Future)
- Quantum branching
- Real-time collaboration
- Web interface
- Enterprise features