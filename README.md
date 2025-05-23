# GameCode - Agentic AI Development Platform

An experimental, modular platform for building agentic AI assistants with AWS
Bedrock Claude 3.7 and Haiku 3.5 models. GameCode provides a clean, trait-based
architecture for LLM backends with comprehensive tool support and session
management.

## 🏗️ Architecture Overview

GameCode follows a modular, clean architecture with clear separation of concerns:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   gamecode-cli  │───▶│ gamecode-backend │◀───│gamecode-bedrock │
│ (User Interface)│    │   (Core Traits)  │    │ (AWS Provider)  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ gamecode-tools  │    │gamecode-context  │    │gamecode-prompt   │
│ (JSONRPC Tools) │    │ (Context Mgmt)   │    │(Prompt Templates)│
└─────────────────┘    └──────────────────┘    └──────────────────┘
```

### Core Design Principles

- **Backend Agnostic**: Clean trait abstraction allows multiple LLM providers
- **Tool-First**: Native JSONRPC tool integration with schema generation
- **Session Management**: Persistent conversation state with UUID identification
- **Retry Resilience**: Production-ready exponential backoff and error handling
- **Status Transparency**: Real-time feedback for long-running operations
- **Modular Components**: Each repository serves a single, well-defined purpose

## 📦 Component Repositories

### Core Infrastructure

#### [gamecode-backend](https://github.com/navicore/gamecode-backend)
**Core LLM Backend Traits and Types**

Defines the foundational `LLMBackend` trait and common types for
provider-agnostic LLM interactions. Includes comprehensive error handling, retry
configuration, and status callback system.

- **Key Features**: Backend trait definition, retry policies, status callbacks
- **Dependencies**: None (pure Rust trait library)
- **Used By**: gamecode-cli, gamecode-bedrock

#### [gamecode-bedrock](https://github.com/navicore/gamecode-bedrock)
**AWS Bedrock Provider Implementation**

AWS Bedrock implementation of the `LLMBackend` trait with robust retry logic,
rate limiting, and comprehensive error handling.

- **Key Features**: Claude 3.7/Haiku 3.5 support, exponential backoff, message conversion
- **Dependencies**: aws-sdk-bedrock, gamecode-backend
- **Used By**: gamecode-cli

### User Interface

#### [gamecode-cli](https://github.com/navicore/gamecode-cli)
**Command Line Interface**

Interactive CLI with shell completion, session management, and tool
orchestration. Provides a clean user experience with real-time status feedback
and comprehensive session commands.

- **Key Features**: Tab completion, session management, tool dispatching, status display
- **Dependencies**: clap, gamecode-backend, gamecode-bedrock, gamecode-tools
- **Commands**: Chat interface, session list/show/delete, shell completion generation

### Tool Ecosystem

#### [gamecode-tools](https://github.com/navicore/gamecode-tools)
**JSONRPC Tool Implementations**

Collection of JSONRPC-compatible tools for file operations, system commands, and
development workflows. Provides schema generation and type-safe tool
dispatching.

- **Key Features**: File I/O tools, system commands, schema generation
- **Dependencies**: serde, tokio
- **Used By**: gamecode-cli

#### [gamecode-context](https://github.com/navicore/gamecode-context)
**Context Management**

Advanced context window management with intelligent truncation, summarization,
and conversation state preservation.

- **Key Features**: Context window optimization, conversation summarization
- **Dependencies**: gamecode-backend
- **Status**: Planned/Future

#### [gamecode-prompt](https://github.com/navicore/gamecode-prompt)
**Prompt Templates**

Reusable prompt templates and engineering patterns for common agentic AI
workflows and use cases.

- **Key Features**: Template library, prompt engineering patterns
- **Dependencies**: None
- **Status**: Planned/Future

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- AWS credentials configured
- Access to AWS Bedrock Claude models

### Installation

```bash
# Clone the CLI
git clone https://github.com/navicore/gamecode-cli.git
cd gamecode-cli

# Build and install
cargo install --path .

# Generate shell completions (optional)
gamecode-cli completion bash > /etc/bash_completion.d/gamecode-cli
```

### Basic Usage

```bash
# Start a conversation
gamecode-cli --prompt "Help me refactor this Rust code"

# Use a specific model
gamecode-cli --model "anthropic.claude-3-5-haiku-20241022-v1:0" --prompt "Explain async/await"

# Continue a previous session
gamecode-cli --session "550e8400-e29b-41d4-a716-446655440000" --prompt "Continue where we left off"

# Manage sessions
gamecode-cli sessions list
gamecode-cli sessions show <session-id>
gamecode-cli sessions delete <session-id>
```

## 🔧 Development

### Local Development Setup

```bash
# Clone all repositories
git clone https://github.com/navicore/gamecode.git
cd gamecode

# Clone component repositories
git clone https://github.com/navicore/gamecode-backend.git
git clone https://github.com/navicore/gamecode-bedrock.git
git clone https://github.com/navicore/gamecode-cli.git
git clone https://github.com/navicore/gamecode-tools.git
git clone https://github.com/navicore/gamecode-desktop.git
```

### Building Components

```bash
# Build backend trait library
cd gamecode-backend && cargo build && cargo test

# Build Bedrock implementation
cd ../gamecode-bedrock && cargo build && cargo test

# Build CLI with local dependencies
cd ../gamecode-cli && cargo build

# Test end-to-end
cargo run -- --prompt "Hello, GameCode!"
```

## 📊 Status

| Component | Status | Version | Tests | Docs |
|-----------|--------|---------|-------|------|
| gamecode-backend | ✅ Stable | 0.1.0 | ✅ | ✅ |
| gamecode-bedrock | ✅ Stable | 0.1.0 | ✅ | ✅ |
| gamecode-cli | ✅ Stable | 0.2.5 |   |   |
| gamecode-tools | ✅ Stable | 0.2.0 | ✅ | ✅ |
| gamecode-context | ✅ Stable | 0.1.0 | - | - |
| gamecode-prompt | ✅ Stable | 0.1.0 | - | - |
| gamecode-desktop |   In Dev | 0.1.0 | - | - |

## 🎯 Roadmap

- Full MCP HTTP for the tools lib
- Additional LLM provider backends (OpenAI, Anthropic Direct)

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Community

- **Issues**: Report bugs and request features in individual component repositories
- **Contributions**: PRs welcome

---

