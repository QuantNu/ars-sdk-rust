use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error type for ARS client operations
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

/// Agent details structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub version: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<u64>,
}

/// Response from agent registration
#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub agent: AgentDetails,
    pub token: String,
}

/// Response from agent discovery
#[derive(Debug, Deserialize)]
pub struct DiscoverResponse {
    pub agents: Vec<AgentDetails>,
    pub total: usize,
}

/// Response from agent verification
#[derive(Debug, Deserialize)]
pub struct VerifyResponse {
    pub verified: bool,
}

/// Builder for agent details
#[derive(Default)]
pub struct AgentDetailsBuilder {
    name: Option<String>,
    version: Option<String>,
    endpoint: Option<String>,
    capabilities: Vec<String>,
    public_key: Option<String>,
    metadata: HashMap<String, String>,
}

impl AgentDetailsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
    
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }
    
    pub fn add_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
    
    pub fn capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }
    
    pub fn public_key(mut self, public_key: impl Into<String>) -> Self {
        self.public_key = Some(public_key.into());
        self
    }
    
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn build(self) -> Result<AgentDetails, String> {
        let name = self.name.ok_or_else(|| "name is required".to_string())?;
        let version = self.version.ok_or_else(|| "version is required".to_string())?;
        let endpoint = self.endpoint.ok_or_else(|| "endpoint is required".to_string())?;
        let public_key = self.public_key.ok_or_else(|| "public_key is required".to_string())?;
        
        Ok(AgentDetails {
            id: None,
            name,
            version,
            endpoint,
            capabilities: self.capabilities,
            public_key,
            metadata: if self.metadata.is_empty() {
                None
            } else {
                Some(self.metadata)
            },
            registered_at: None,
            updated_at: None,
        })
    }
}

/// Client for interacting with the Agent Registration Server
#[derive(Debug, Clone)]
pub struct ARSClient {
    server_url: String,
    agent_id: Option<String>,
    auth_token: Option<String>,
    client: reqwest::Client,
}

impl ARSClient {
    /// Create a new ARS client
    pub fn new(server_url: impl Into<String>) -> Self {
        let mut server_url = server_url.into();
        // Remove trailing slash if present
        if server_url.ends_with('/') {
            server_url.pop();
        }
        
        Self {
            server_url,
            agent_id: None,
            auth_token: None,
            client: reqwest::Client::new(),
        }
    }
    
    /// Create a new client with existing credentials
    pub fn with_credentials(
        server_url: impl Into<String>,
        agent_id: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Self {
        let mut client = Self::new(server_url);
        client.agent_id = Some(agent_id.into());
        client.auth_token = Some(auth_token.into());
        client
    }
    
    /// Register a new agent with the ARS
    pub async fn register_agent(&mut self, agent: AgentDetails) -> Result<RegisterResponse, ARSError> {
        let url = format!("{}/api/agents", self.server_url);
        
        let response = self.client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&agent)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(ARSError::ApiError {
                status_code,
                message,
            });
        }
        
        let register_response = response.json::<RegisterResponse>().await?;
        
        // Save credentials for future calls
        self.agent_id = register_response.agent.id.clone();
        self.auth_token = Some(register_response.token.clone());
        
        Ok(register_response)
    }
    
    /// Discover agents based on capabilities and metadata
    pub async fn discover_agents(
        &self,
        capabilities: Option<Vec<String>>,
        metadata_filter: Option<HashMap<String, String>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<DiscoverResponse, ARSError> {
        let mut url = format!("{}/api/agents/discover", self.server_url);
        
        // Build query parameters
        let mut query_params = Vec::new();
        
        if let Some(limit_val) = limit {
            query_params.push(format!("limit={}", limit_val));
        }
        
        if let Some(offset_val) = offset {
            query_params.push(format!("offset={}", offset_val));
        }
        
        if let Some(capabilities_val) = capabilities {
            if !capabilities_val.is_empty() {
                query_params.push(format!("capability={}", capabilities_val.join(",")));
            }
        }
        
        if let Some(metadata) = metadata_filter {
            if !metadata.is_empty() {
                let metadata_json = serde_json::to_string(&metadata)?;
                query_params.push(format!("metadata={}", metadata_json));
            }
        }
        
        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(ARSError::ApiError {
                status_code,
                message,
            });
        }
        
        let discover_response = response.json::<DiscoverResponse>().await?;
        Ok(discover_response)
    }
    
    /// Get information about a specific agent
    pub async fn get_agent(&self, agent_id: &str) -> Result<AgentDetails, ARSError> {
        let url = format!("{}/api/agents/{}", self.server_url, agent_id);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(ARSError::ApiError {
                status_code,
                message,
            });
        }
        
        let agent = response.json::<AgentDetails>().await?;
        Ok(agent)
    }
    
    /// Update agent details
    pub async fn update_agent(&self, details: AgentDetails) -> Result<AgentDetails, ARSError> {
        let agent_id = match &self.agent_id {
            Some(id) => id,
            None => return Err(ARSError::AuthError("Agent not registered".to_string())),
        };
        
        let auth_token = match &self.auth_token {
            Some(token) => token,
            None => return Err(ARSError::AuthError("Not authenticated".to_string())),
        };
        
        let url = format!("{}/api/protected/agents/{}", self.server_url, agent_id);
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_token))
                .map_err(|_| ARSError::AuthError("Invalid auth token".to_string()))?,
        );
        
        let response = self.client
            .patch(&url)
            .headers(headers)
            .json(&details)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(ARSError::ApiError {
                status_code,
                message,
            });
        }
        
        let updated_agent = response.json::<AgentDetails>().await?;
        Ok(updated_agent)
    }
    
    /// Deregister an agent
    pub async fn deregister_agent(&mut self) -> Result<(), ARSError> {
        let agent_id = match &self.agent_id {
            Some(id) => id,
            None => return Err(ARSError::AuthError("Agent not registered".to_string())),
        };
        
        let auth_token = match &self.auth_token {
            Some(token) => token,
            None => return Err(ARSError::AuthError("Not authenticated".to_string())),
        };
        
        let url = format!("{}/api/protected/agents/{}", self.server_url, agent_id);
        
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_token))
                .map_err(|_| ARSError::AuthError("Invalid auth token".to_string()))?,
        );
        
        let response = self.client
            .delete(&url)
            .headers(headers)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(ARSError::ApiError {
                status_code,
                message,
            });
        }
        
        // Clear client state
        self.agent_id = None;
        self.auth_token = None;
        
        Ok(())
    }
    
    /// Verify another agent's identity using challenge-response
    pub async fn verify_agent(
        &self,
        agent_id: &str,
        challenge: &[u8],
        signature: &[u8],
    ) -> Result<bool, ARSError> {
        let url = format!("{}/api/agents/{}/verify", self.server_url, agent_id);
        
        let payload = serde_json::json!({
            "challenge": base64::encode(challenge),
            "signature": base64::encode(signature),
        });
        
        let response = self.client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(ARSError::ApiError {
                status_code,
                message,
            });
        }
        
        let verify_response = response.json::<VerifyResponse>().await?;
        Ok(verify_response.verified)
    }
}
