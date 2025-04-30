# ARS Rust SDK

Rust SDK for the Agent Registration Server (ARS).

## Overview

The ARS Rust SDK provides a client library for interacting with the Agent Registration Server, enabling Rust applications to register agents, discover capabilities, manage sessions, and execute operations across different agent protocols.

## Installation

### For Users

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ars-client = "0.1.0"
```

### For Developers

Clone the repository and build:

```bash
git clone https://github.com/quantnu/ars.git
cd ars/sdk/rust
cargo build
```

## Building Independently

The Rust SDK can be built independently using the included Makefile:

```bash
# Clean, build, and test
make

# Individual steps
make build     # Build the project in debug mode
make release   # Build the project in release mode
make test      # Run tests
make clean     # Clean build artifacts
make doc       # Generate documentation
make clippy    # Run clippy for linting
make fmt       # Format code
```

If you don't have `make` available, you can use Cargo commands directly:

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Generate documentation
cargo doc --no-deps
```

## Usage

```rust
use ars_client::{ARSClient, AgentRegistration, Protocol, TrustLevel, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create client
    let client = ARSClient::new("https://ars.example.com");

    // Register an agent
    let (agent, token) = client.register_agent(AgentRegistration {
        name: "Example Agent".to_string(),
        description: "An example agent demonstrating basic functionality".to_string(),
        capabilities: vec!["translate".to_string(), "summarize".to_string()],
        endpoint: "https://example.com/agent".to_string(),
        protocol: Protocol::MCP,
        protocol_version: "1.0".to_string(),
        public_key: "example-public-key".to_string(),
        metadata: None,
    }).await?;

    println!("Registered agent with ID: {}", agent.id);
    println!("Agent token: {}", token);

    // Discover agents with specific capabilities
    let agents = client.discover_agents(
        vec!["translate".to_string()],
        vec![TrustLevel::Verified, TrustLevel::Partner],
    ).await?;

    println!("Found {} agents", agents.len());

    // Execute a task
    let params = serde_json::json!({
        "text": "Hello world",
        "sourceLanguage": "en",
        "targetLanguage": "fr",
    });

    let result = client.execute_task("translate", params).await?;
    println!("Translation result: {}", result);

    Ok(())
}
```

## Session Management

```rust
use ars_client::{ARSSessionClient, SessionData, Error};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create session-aware client
    let client = ARSSessionClient::new("https://ars.example.com");

    // Create session data
    let mut preferences = HashMap::new();
    preferences.insert("language".to_string(), "en".into());

    let mut context = HashMap::new();
    context.insert("user".to_string(), "user-123".into());
    context.insert("preferences".to_string(), preferences.into());

    let session_data = SessionData {
        context: context,
        metadata: HashMap::new(),
    };

    // Create a session
    let (session, session_token) = client.create_session(session_data).await?;
    println!("Created session with ID: {}", session.id);

    // Execute task using session context
    let params = serde_json::json!({
        "text": "Hello world",
        "targetLanguage": "fr",
    });

    let result = client.execute_task_with_session("translate", params, &session.id).await?;
    println!("Translation result: {}", result);

    // Update session with new information
    let mut history_entry = HashMap::new();
    history_entry.insert("task".to_string(), "translate".into());
    history_entry.insert("input".to_string(), serde_json::json!({"text": "Hello world", "targetLanguage": "fr"}).into());
    history_entry.insert("output".to_string(), "Bonjour le monde".into());

    let mut history = Vec::new();
    history.push(history_entry);

    let mut new_context = HashMap::new();
    new_context.insert("history".to_string(), history.into());

    let update_data = SessionData {
        context: new_context,
        metadata: HashMap::new(),
    };

    client.update_session(&session.id, update_data).await?;

    Ok(())
}
```

## Features

- **Agent Registration**: Register agents with the ARS
- **Agent Discovery**: Find agents based on capabilities and trust levels
- **Trust Verification**: Verify agent identity and trust levels
- **Session Management**: Maintain stateful interactions between agents
- **Cross-Protocol Operation**: Work with agents across different protocols
- **Async/Await**: Full async support with Tokio runtime
- **Error Handling**: Comprehensive error handling with custom error types
- **Serialization**: Seamless serde integration for request/response handling

## Contributing

Contributions are welcome! Please see the main repository's CONTRIBUTING.md for guidelines.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
