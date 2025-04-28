# Agent Registration Server Rust Client

A Rust client library for interacting with the Agent Registration Server (ARS).

## Installation

Add the library to your Cargo.toml:

```toml
[dependencies]
ars-client = { git = "https://github.com/yourusername/ars", path = "sdk/rust" }
```

## Usage

```rust
use ars_client::{ARSClient, AgentDetailsBuilder, ARSError};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), ARSError> {
    // Create a new client
    let mut client = ARSClient::new("https://ars.example.com");
    
    // Generate a key pair (simplified for example)
    let public_key = base64::encode("YOUR_PUBLIC_KEY_HERE");
    
    // Build agent details
    let agent = AgentDetailsBuilder::new()
        .name("My Agent")
        .version("1.0.0")
        .endpoint("https://myagent.example.com/api")
        .add_capability("query")
        .add_capability("response")
        .public_key(public_key)
        .add_metadata("description", "My awesome agent")
        .build()?;
    
    // Register the agent
    let response = client.register_agent(agent).await?;
    println!("Registered agent with ID: {}", response.agent.id.unwrap_or_default());
    
    // Save credentials for future sessions
    let agent_id = response.agent.id.unwrap();
    let token = response.token;
    
    // Or create a client with existing credentials
    let client = ARSClient::with_credentials(
        "https://ars.example.com",
        agent_id,
        token,
    );
    
    // Discover other agents
    let discovery = client
        .discover_agents(
            Some(vec!["query".to_string()]),
            None,
            Some(5),
            None,
        )
        .await?;
    
    println!("Found {} agents with 'query' capability", discovery.total);
    
    // Deregister when done
    client.deregister_agent().await?;
    
    Ok(())
}
```

## API Reference

### ARSClient

```rust
let client = ARSClient::new(server_url);
// or
let client = ARSClient::with_credentials(server_url, agent_id, auth_token);
```

Create a new client with the server URL, optionally with existing credentials.

### Methods

All methods return `Result<T, ARSError>` and are async.

- `register_agent(&mut self, agent: AgentDetails) -> Result<RegisterResponse, ARSError>`: Register a new agent
- `discover_agents(&self, capabilities: Option<Vec<String>>, metadata_filter: Option<HashMap<String, String>>, limit: Option<usize>, offset: Option<usize>) -> Result<DiscoverResponse, ARSError>`: Find agents matching criteria
- `get_agent(&self, agent_id: &str) -> Result<AgentDetails, ARSError>`: Get details for a specific agent
- `update_agent(&self, details: AgentDetails) -> Result<AgentDetails, ARSError>`: Update your agent
- `deregister_agent(&mut self) -> Result<(), ARSError>`: Remove your agent from the registry
- `verify_agent(&self, agent_id: &str, challenge: &[u8], signature: &[u8]) -> Result<bool, ARSError>`: Verify another agent's identity

### AgentDetailsBuilder

A builder pattern for constructing agent details:

```rust
let agent = AgentDetailsBuilder::new()
    .name("My Agent")
    .version("1.0.0")
    .endpoint("https://example.com/api")
    .add_capability("query")
    .public_key(public_key)
    .add_metadata("key", "value")
    .build()?;
```

## Error Handling

The library provides an `ARSError` enum that wraps various error conditions:

```rust
#[derive(Error, Debug)]
pub enum ARSError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("API error: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
    },
}
```

Handle these errors appropriately in your code.

## Examples

See the [examples](./examples) directory for more usage examples.

## License

MIT
