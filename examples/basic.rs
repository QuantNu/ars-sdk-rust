use ars_client::{ARSClient, AgentDetailsBuilder, ARSError};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), ARSError> {
    // Create a new client
    let mut client = ARSClient::new("https://ars.example.com");
    
    // For demonstration, we'll use a simple Base64-encoded placeholder for a public key
    // In a real implementation, you would generate a proper key pair
    let public_key = base64::encode("DEMO_PUBLIC_KEY_NOT_FOR_PRODUCTION");
    
    // Build agent details using the builder pattern
    let agent = AgentDetailsBuilder::new()
        .name("Example Rust Agent")
        .version("1.0.0")
        .endpoint("https://agent.example.com/api")
        .add_capability("query")
        .add_capability("response")
        .add_capability("calculation")
        .public_key(public_key)
        .add_metadata("description", "Example agent for demonstration")
        .add_metadata("creator", "ARS SDK")
        .add_metadata("language", "Rust")
        .build()
        .expect("Failed to build agent details");
    
    // Register the agent
    let response = client.register_agent(agent).await?;
    
    println!("Registered agent with ID: {}", response.agent.id.unwrap_or_default());
    println!("Authentication token: {}", response.token);
    
    // Discover agents with the "query" capability
    let discovery = client
        .discover_agents(
            Some(vec!["query".to_string()]),
            None,
            Some(5),
            None,
        )
        .await?;
    
    println!("Found {} agents with 'query' capability:", discovery.total);
    for (i, agent) in discovery.agents.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, agent.name, agent.id.as_ref().unwrap_or(&"unknown".to_string()));
    }
    
    // Update the agent
    let mut update_metadata = HashMap::new();
    update_metadata.insert("status".to_string(), "active".to_string());
    
    let update_details = AgentDetailsBuilder::new()
        .version("1.0.1")
        .metadata(update_metadata)
        .build()
        .expect("Failed to build update details");
    
    let updated_agent = client.update_agent(update_details).await?;
    println!("Updated agent to version {}", updated_agent.version);
    
    // Deregister the agent
    client.deregister_agent().await?;
    println!("Agent deregistered successfully");
    
    Ok(())
}
