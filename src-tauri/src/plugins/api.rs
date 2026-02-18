// Plugin API - API exposed to plugins

use serde::{Deserialize, Serialize};

/// Plugin API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// Plugin API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Available API methods
pub const API_METHODS: &[&str] = &[
    "fs.readFile",
    "fs.writeFile",
    "fs.listFiles",
    "fs.deleteFile",
    "http.get",
    "http.post",
    "db.query",
    "db.execute",
    "system.notify",
    "system.clipboard",
    "log.info",
    "log.error",
];

/// API method categories
pub fn get_api_categories() -> Vec<ApiCategory> {
    vec![
        ApiCategory {
            name: "fs".to_string(),
            methods: vec![
                ApiMethod {
                    name: "readFile".to_string(),
                    description: "Read file contents".to_string(),
                    params: vec!["path: string".to_string()],
                    returns: "string".to_string(),
                },
                ApiMethod {
                    name: "writeFile".to_string(),
                    description: "Write file contents".to_string(),
                    params: vec!["path: string".to_string(), "content: string".to_string()],
                    returns: "void".to_string(),
                },
                ApiMethod {
                    name: "listFiles".to_string(),
                    description: "List files in directory".to_string(),
                    params: vec!["path: string".to_string()],
                    returns: "string[]".to_string(),
                },
                ApiMethod {
                    name: "deleteFile".to_string(),
                    description: "Delete a file".to_string(),
                    params: vec!["path: string".to_string()],
                    returns: "void".to_string(),
                },
            ],
        },
        ApiCategory {
            name: "http".to_string(),
            methods: vec![
                ApiMethod {
                    name: "get".to_string(),
                    description: "HTTP GET request".to_string(),
                    params: vec!["url: string".to_string()],
                    returns: "Response".to_string(),
                },
                ApiMethod {
                    name: "post".to_string(),
                    description: "HTTP POST request".to_string(),
                    params: vec!["url: string".to_string(), "body: any".to_string()],
                    returns: "Response".to_string(),
                },
            ],
        },
        ApiCategory {
            name: "db".to_string(),
            methods: vec![
                ApiMethod {
                    name: "query".to_string(),
                    description: "Execute SQL query".to_string(),
                    params: vec!["sql: string".to_string()],
                    returns: "Row[]".to_string(),
                },
                ApiMethod {
                    name: "execute".to_string(),
                    description: "Execute SQL statement".to_string(),
                    params: vec!["sql: string".to_string()],
                    returns: "void".to_string(),
                },
            ],
        },
        ApiCategory {
            name: "system".to_string(),
            methods: vec![
                ApiMethod {
                    name: "notify".to_string(),
                    description: "Show system notification".to_string(),
                    params: vec!["title: string".to_string(), "message: string".to_string()],
                    returns: "void".to_string(),
                },
                ApiMethod {
                    name: "clipboard".to_string(),
                    description: "Access clipboard".to_string(),
                    params: vec!["action: 'read' | 'write'".to_string()],
                    returns: "string | void".to_string(),
                },
            ],
        },
        ApiCategory {
            name: "log".to_string(),
            methods: vec![
                ApiMethod {
                    name: "info".to_string(),
                    description: "Log info message".to_string(),
                    params: vec!["message: string".to_string()],
                    returns: "void".to_string(),
                },
                ApiMethod {
                    name: "error".to_string(),
                    description: "Log error message".to_string(),
                    params: vec!["message: string".to_string()],
                    returns: "void".to_string(),
                },
            ],
        },
    ]
}

/// API category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCategory {
    pub name: String,
    pub methods: Vec<ApiMethod>,
}

/// API method definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMethod {
    pub name: String,
    pub description: String,
    pub params: Vec<String>,
    pub returns: String,
}

/// Handle plugin API request
pub fn handle_request(request: PluginRequest) -> PluginResponse {
    // Validate method exists
    if !API_METHODS.contains(&request.method.as_str()) {
        return PluginResponse {
            id: request.id,
            result: None,
            error: Some(format!("Unknown method: {}", request.method)),
        };
    }

    // In production, this would route to actual implementations
    PluginResponse {
        id: request.id,
        result: Some(serde_json::json!({"status": "ok"})),
        error: None,
    }
}
