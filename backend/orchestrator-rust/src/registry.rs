use serde::Serialize;
use serde_json::Value;

pub type ToolResult = Result<Value, String>;
pub type ToolHandler = fn(&Value, &Registry) -> ToolResult;

#[derive(Clone, Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub roles: Vec<String>,
    #[serde(rename = "input_schema")]
    pub input_schema: Value,
    #[serde(skip_serializing)]
    pub handler: ToolHandler,
}

#[derive(Clone, Debug, Default)]
pub struct Registry {
    tools: Vec<Tool>,
}

impl Registry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    pub fn list_registered_tools(&self) -> Vec<String> {
        self.tools.iter().map(|tool| tool.name.clone()).collect()
    }

    pub fn tools_for_role(&self, role: &str) -> Vec<Tool> {
        self.tools
            .iter()
            .filter(|tool| {
                tool.roles.iter().any(|r| r == "*")
                    || tool.roles.iter().any(|r| r == role)
                    || role == "admin"
            })
            .cloned()
            .collect()
    }

    pub fn find_for_role(&self, name: &str, role: &str) -> Option<Tool> {
        self.tools_for_role(role)
            .into_iter()
            .find(|tool| tool.name == name)
    }
}

pub fn object_schema(properties: Value, required: Vec<&str>) -> Value {
    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}
