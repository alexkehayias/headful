use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Represents the Chrome Accessibility Tree node structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxTree {
    pub nodes: Vec<AxNode>,
}

/// A single node in the accessibility tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxNode {
    #[serde(rename = "backendDOMNodeId")]
    pub backend_dom_node_id: Option<i64>,
    #[serde(rename = "childIds")]
    pub child_ids: Option<Vec<String>>,
    #[serde(rename = "chromeRole")]
    pub chrome_role: Option<ChromeRole>,
    #[serde(skip_deserializing)]
    pub ignored: bool,
    #[serde(rename = "ignoredReasons")]
    pub ignored_reasons: Option<Vec<IgnoredReason>>,
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub role: Role,
    pub name: Option<Name>,
    pub properties: Option<Vec<Property>>,
}

/// Chrome role (used in chromeRole field)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeRole {
    #[serde(rename = "type")]
    pub role_type: String,
    pub value: RoleValueContent,
}

/// Role value can be either an integer (internalRole) or a string role name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RoleValueContent {
    Internal(i64),
    Named(String),
}

/// Role information (used in role field)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    #[serde(rename = "type")]
    pub role_type: String,
    pub value: RoleValueContent,
}

/// Reason why a node is ignored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoredReason {
    pub name: String,
    #[serde(rename = "value")]
    pub value_type: ValueBool,
}

/// Boolean value wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueBool {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(default)]
    pub value: bool,
}

/// Name sources for accessibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    #[serde(default)]
    pub sources: Vec<NameSource>,
    #[serde(rename = "type")]
    pub name_type: String,
    #[serde(default)]
    pub value: String,
}

/// Source of a name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameSource {
    #[serde(skip_deserializing)]
    pub attribute: Option<String>,
    #[serde(rename = "superseded")]
    pub superseded: Option<bool>,
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(skip_deserializing)]
    pub value: Option<Value>,
}

/// Value wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

/// Property of a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    #[serde(rename = "value")]
    pub value_type: PropertyValue,
}

/// Property value wrapper - handles both simple values and complex objects
#[derive(Debug, Clone)]
pub struct PropertyValue {
    pub value_type: String,
    pub value: PropertyValueContent,
}

impl Serialize for PropertyValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &self.value_type)?;
        match &self.value {
            PropertyValueContent::Boolean(b) => map.serialize_entry("value", b)?,
            PropertyValueContent::SimpleBoolean(b) => map.serialize_entry("value", b)?,
            PropertyValueContent::String(s) => map.serialize_entry("value", s)?,
            PropertyValueContent::Integer(i) => map.serialize_entry("value", i)?,
            PropertyValueContent::Token(t) => map.serialize_entry("value", t)?,
            PropertyValueContent::NodeList(nodes) => map.serialize_entry("value", nodes)?,
            PropertyValueContent::TokenList(tokens) => map.serialize_entry("value", tokens)?,
            PropertyValueContent::Unknown(v) => {
                // Serialize the full unknown value (which may contain "value" field)
                if let serde_json::Value::Object(obj) = v {
                    for (key, val) in obj {
                        map.serialize_entry(&key, &val)?;
                    }
                } else {
                    map.serialize_entry("value", v)?;
                }
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for PropertyValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawPropertyValue {
            #[serde(rename = "type")]
            value_type: String,
            #[serde(flatten)]
            rest: serde_json::Value,
        }

        let raw = RawPropertyValue::deserialize(deserializer)?;

        // Extract the value based on the type
        let value = match raw.value_type.as_str() {
            "booleanOrUndefined" => {
                if let serde_json::Value::Object(mut map) = raw.rest {
                    // Get the "value" field
                    if let Some(v) = map.remove("value") {
                        match serde_json::from_value::<BooleanOrUndefined>(
                            serde_json::json!({ "type": raw.value_type, "value": v })
                        ) {
                            Ok(b) => PropertyValueContent::Boolean(b),
                            Err(_) => return Err(serde::de::Error::custom("invalid boolean value")),
                        }
                    } else {
                        // Handle missing value field gracefully
                        PropertyValueContent::Boolean(BooleanOrUndefined { value_type: "booleanOrUndefined".to_string(), value: false })
                    }
                } else {
                    return Err(serde::de::Error::custom("invalid boolean object"));
                }
            }
            "boolean" => {
                if let Some(v) = raw.rest.get("value") {
                    match serde_json::from_value::<bool>(v.clone()) {
                        Ok(b) => PropertyValueContent::SimpleBoolean(b),
                        Err(_) => return Err(serde::de::Error::custom("invalid boolean value")),
                    }
                } else {
                    // Handle missing value field gracefully
                    PropertyValueContent::SimpleBoolean(false)
                }
            }
            "string" => {
                if let Some(v) = raw.rest.get("value") {
                    match serde_json::from_value::<String>(v.clone()) {
                        Ok(s) => PropertyValueContent::String(s),
                        Err(_) => return Err(serde::de::Error::custom("invalid string value")),
                    }
                } else {
                    // Handle missing value field gracefully
                    PropertyValueContent::String(String::new())
                }
            }
            "integer" => {
                if let Some(v) = raw.rest.get("value") {
                    match serde_json::from_value::<i64>(v.clone()) {
                        Ok(i) => PropertyValueContent::Integer(i),
                        Err(_) => return Err(serde::de::Error::custom("invalid integer value")),
                    }
                } else {
                    // Handle missing value field gracefully
                    PropertyValueContent::Integer(0)
                }
            }
            "token" => {
                if let Some(v) = raw.rest.get("value") {
                    match serde_json::from_value::<String>(v.clone()) {
                        Ok(s) => PropertyValueContent::Token(s),
                        Err(_) => return Err(serde::de::Error::custom("invalid token value")),
                    }
                } else {
                    // Handle missing value field gracefully
                    PropertyValueContent::Token(String::new())
                }
            }
            "nodeList" => {
                if let Some(v) = raw.rest.get("value") {
                    match serde_json::from_value::<Vec<String>>(v.clone()) {
                        Ok(nodes) => PropertyValueContent::NodeList(nodes),
                        Err(_) => return Err(serde::de::Error::custom("invalid node list value")),
                    }
                } else {
                    // Handle missing value field gracefully
                    PropertyValueContent::NodeList(Vec::new())
                }
            }
            "tokenList" => {
                if let Some(v) = raw.rest.get("value") {
                    // Try parsing as Vec<String> first
                    if let Ok(tokens) = serde_json::from_value::<Vec<String>>(v.clone()) {
                        PropertyValueContent::TokenList(tokens)
                    } else if let Ok(nodes) = serde_json::from_value::<Vec<AxNode>>(v.clone()) {
                        // If it's a list of AxNodes, extract node IDs
                        let ids: Vec<String> = nodes.iter().map(|n| n.node_id.clone()).collect();
                        PropertyValueContent::TokenList(ids)
                    } else {
                        // Try to handle any JSON array - try extracting strings
                        match v {
                            serde_json::Value::Array(arr) => {
                                let tokens: Vec<String> = arr.iter()
                                    .map(|item| match item {
                                        serde_json::Value::String(s) => s.clone(),
                                        serde_json::Value::Object(obj) => {
                                            // Try to get a value field or serialize the whole object
                                            obj.get("value")
                                                .and_then(|val| val.as_str())
                                                .map(String::from)
                                                .unwrap_or_else(|| serde_json::to_string(item).unwrap_or_default())
                                        }
                                        _ => serde_json::to_string(item).unwrap_or_default(),
                                    })
                                    .collect();
                                PropertyValueContent::TokenList(tokens)
                            }
                            _ => {
                                // Not an array - serialize to string and return as a single-item token list
                                let str_val = serde_json::to_string(&v).unwrap_or_default();
                                PropertyValueContent::TokenList(vec![str_val])
                            }
                        }
                    }
                } else {
                    // Handle missing value field gracefully
                    PropertyValueContent::TokenList(Vec::new())
                }
            }
            _ => PropertyValueContent::Unknown(raw.rest),
        };

        Ok(PropertyValue {
            value_type: raw.value_type,
            value,
        })
    }
}

/// Content of the property value - handles both wrapped objects and direct values
#[derive(Debug, Clone)]
pub enum PropertyValueContent {
    Boolean(BooleanOrUndefined),
    SimpleBoolean(bool),
    String(String),
    Integer(i64),
    Token(String),
    NodeList(Vec<String>),
    TokenList(Vec<String>),
    Unknown(serde_json::Value),
}

impl Serialize for PropertyValueContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PropertyValueContent::Boolean(b) => b.serialize(serializer),
            PropertyValueContent::SimpleBoolean(b) => b.serialize(serializer),
            PropertyValueContent::String(s) => s.serialize(serializer),
            PropertyValueContent::Integer(i) => i.serialize(serializer),
            PropertyValueContent::Token(t) => t.serialize(serializer),
            PropertyValueContent::NodeList(nodes) => nodes.serialize(serializer),
            PropertyValueContent::TokenList(tokens) => tokens.serialize(serializer),
            PropertyValueContent::Unknown(v) => v.serialize(serializer),
        }
    }
}

/// Boolean or undefined value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanOrUndefined {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: bool,
}

impl AxTree {
    /// Build a parent-child relationship map from the nodes
    #[allow(dead_code)]
    pub fn build_parent_map(&self) -> HashMap<String, Vec<&AxNode>> {
        let mut map: HashMap<String, Vec<&AxNode>> = HashMap::new();
        for node in &self.nodes {
            if let Some(ref parent_id) = node.parent_id {
                map.entry(parent_id.clone()).or_default().push(node);
            }
        }
        map
    }

    /// Get the root node (typically the RootWebArea)
    pub fn find_root(&self) -> Option<&AxNode> {
        self.nodes.iter().find(|n| {
            matches!(n.role.value, RoleValueContent::Named(ref v) if v == "RootWebArea")
        })
    }

    /// Get children of a node by parent ID
    #[allow(dead_code)]
    pub fn get_children(&self, parent_id: &str) -> Vec<&AxNode> {
        self.nodes
            .iter()
            .filter(|n| n.parent_id.as_deref() == Some(parent_id))
            .collect()
    }

    /// Check if a node should be ignored (uninteresting)
    #[allow(dead_code)]
    pub fn is_ignored(&self, node: &AxNode) -> bool {
        node.ignored
            || node
                .ignored_reasons
                .as_ref()
                .map(|reasons| {
                    reasons.iter().any(|r| r.name == "uninteresting")
                })
                .unwrap_or(false)
    }

    /// Find a node by ID
    pub fn find_node(&self, node_id: &str) -> Option<&AxNode> {
        self.nodes.iter().find(|n| n.node_id == node_id)
    }

    /// Check if a role is an internal role (like StaticText with value 158)
    #[allow(dead_code)]
    pub fn is_internal_role(&self, role: &Role) -> bool {
        matches!(role.value, RoleValueContent::Internal(_))
    }

    /// Get the internal role value if it's an internal role
    pub fn get_internal_role_value(&self, role: &Role) -> Option<i64> {
        match &role.value {
            RoleValueContent::Internal(val) => Some(*val),
            _ => None,
        }
    }

    /// Get the named role value if it's a named role
    pub fn get_named_role_value(&self, role: &Role) -> Option<String> {
        match &role.value {
            RoleValueContent::Named(val) => Some(val.clone()),
            _ => None,
        }
    }
}

/// Markdown conversion context
struct ConvertContext {
    /// Nodes that have been processed (to avoid cycles)
    visited: std::collections::HashSet<String>,
}

impl ConvertContext {
    fn new() -> Self {
        ConvertContext {
            visited: std::collections::HashSet::new(),
        }
    }
}

/// Convert an accessibility tree to markdown
pub fn axtree_to_markdown(axtree: &AxTree) -> String {
    let mut ctx = ConvertContext::new();
    let mut result = Vec::new();

    // Find root and start conversion
    if let Some(root) = axtree.find_root() {
        convert_node(axtree, root, &mut ctx, 0, &mut result);
    }

    // Join with newlines and clean up multiple consecutive blank lines
    let output = result.join("\n");
    clean_whitespace(&output)
}

/// Clean up excessive whitespace
fn clean_whitespace(s: &str) -> String {
    let mut result = String::new();
    let mut prev_blank = false;

    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank && !result.is_empty() {
                result.push('\n');
                prev_blank = true;
            }
        } else {
            result.push_str(line);
            result.push('\n');
            prev_blank = false;
        }
    }

    // Remove trailing whitespace
    result.trim_end().to_string()
}

/// Convert a single node and its children to markdown
fn convert_node(
    axtree: &AxTree,
    node: &AxNode,
    ctx: &mut ConvertContext,
    depth: usize,
    result: &mut Vec<String>,
) {
    // Prevent cycles
    if !ctx.visited.insert(node.node_id.clone()) {
        return;
    }

    // Skip ignored nodes (but still process their children if they have any)
    if axtree.is_ignored(node) && !node.child_ids.as_deref().map(|c| c.is_empty()).unwrap_or(true) {
        for child_id in node.child_ids.as_deref().unwrap() {
            if let Some(child) = axtree.find_node(child_id) {
                convert_node(axtree, child, ctx, depth, result);
            }
        }
        return;
    }

    // Get the role as a named string or internal value
    let role_name = axtree.get_named_role_value(&node.role);

    match role_name.as_deref() {
        Some("RootWebArea") | Some("document") => {
            // Process all children of document
            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth, result);
                }
            }
        }

        Some("heading") => {
            // Get heading level
            let level = get_heading_level(node);
            let header_char = if level <= 6 {
                "#".repeat(level as usize)
            } else {
                "#".repeat(6)
            };

            let text = get_text_content(axtree, node);
            if !text.is_empty() {
                result.push(format!("{} {}", header_char, text));
                result.push(String::new()); // Blank line after heading
            }

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("link") => {
            let text = get_text_content(axtree, node);
            if let Some(url) = get_url(node) {
                result.push(format!("[{}]({})", text, url));
            } else if !text.is_empty() {
                result.push(text);
            }

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("button") => {
            let text = get_text_content(axtree, node);
            if !text.is_empty() {
                result.push(format!("[{}]({})", text, "button"));
            }

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("list") => {
            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("listItem") => {
            let text = get_text_content(axtree, node);
            if !text.is_empty() {
                result.push(format!("- {}", text));
            }

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("paragraph") => {
            let text = get_text_content(axtree, node);
            if !text.is_empty() {
                result.push(text);
                result.push(String::new()); // Blank line after paragraph
            }

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("article") => {
            // Process article content
            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("main") => {
            // Process main content
            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("contentinfo") | Some("footer") => {
            // Process footer content but mark it
            result.push(String::new());
            result.push("--- Footer ---".to_string());

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("separator") => {
            // Add horizontal rule for separators
            let role_level = get_role_level(node);
            if role_level == 1 || depth == 0 {
                result.push(String::new());
                result.push("---".to_string());
            }

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("generic") => {
            // Generic containers - process children
            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("none") => {
            // None roles - just process children
            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        Some("image") => {
            let alt_text = get_alt_text(node);
            if !alt_text.is_empty() {
                result.push(format!("![{}]({})", alt_text, get_url(node).unwrap_or_default()));
            }

            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }

        _ => {
            // For other roles (including internal roles like StaticText, InlineTextBox), process children
            // StaticText has internal value 158, InlineTextBox has 101
            for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
                if let Some(child) = axtree.find_node(child_id) {
                    convert_node(axtree, child, ctx, depth + 1, result);
                }
            }
        }
    }
}

/// Get text content from a node (including StaticText children)
fn get_text_content(axtree: &AxTree, node: &AxNode) -> String {
    let mut text = String::new();

    // Check if this node has direct name/value (and is not just a container for StaticText children)
    if let Some(ref name) = node.name {
        if !name.value.is_empty() && !has_only_static_text_children(axtree, node) {
            text.push_str(&name.value);
        }
    }

    // Get text from StaticText children (Internal role with value 158)
    for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
        if let Some(child) = axtree.find_node(child_id) {
            // Check for StaticText role (internal value 158)
            // Can be in chromeRole field OR in role field with type "internalRole"
            let internal_val = child.chrome_role.as_ref()
                .and_then(|cr| match &cr.value {
                    RoleValueContent::Internal(v) => Some(*v),
                    _ => None,
                })
                .or_else(|| axtree.get_internal_role_value(&child.role));

            let named_val = axtree.get_named_role_value(&child.role);

            if matches!(internal_val, Some(158)) || named_val.as_deref() == Some("StaticText") {
                // StaticText - get the text from name
                if let Some(ref name) = child.name {
                    text.push_str(&name.value);
                }
            } else if matches!(internal_val, Some(101)) || named_val.as_deref() == Some("InlineTextBox") {
                // InlineTextBox - just add the text directly
                if let Some(ref name) = child.name {
                    text.push_str(&name.value);
                }
            } else if !axtree.is_ignored(child) {
                text.push_str(&get_text_content(axtree, child));
            }
        }
    }

    // Clean up whitespace - join words with single space
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Check if node has only StaticText children
fn has_only_static_text_children(axtree: &AxTree, node: &AxNode) -> bool {
    for child_id in node.child_ids.as_deref().unwrap_or(&Vec::new()) {
        if let Some(child) = axtree.find_node(child_id) {
            match &child.role.value {
                RoleValueContent::Internal(val) if *val == 158 || *val == 101 => {}
                RoleValueContent::Named(v) if v == "StaticText" || v == "InlineTextBox" => {}
                _ => return false,
            }
        }
    }
    true
}

/// Get URL from a node's properties
fn get_url(node: &AxNode) -> Option<String> {
    if let Some(ref props) = node.properties {
        for prop in props {
            if prop.name == "url" {
                if let PropertyValueContent::String(url) = &prop.value_type.value {
                    return Some(url.clone());
                }
            }
        }
    }
    None
}

/// Get alt text from an image node
fn get_alt_text(node: &AxNode) -> String {
    if let Some(ref props) = node.properties {
        for prop in props {
            if prop.name == "alt" {
                if let PropertyValueContent::String(alt) = &prop.value_type.value {
                    return alt.clone();
                }
            }
        }
    }
    String::new()
}

/// Get heading level from properties
fn get_heading_level(node: &AxNode) -> i64 {
    if let Some(ref props) = node.properties {
        for prop in props {
            if prop.name == "level" {
                if let PropertyValueContent::Integer(level) = &prop.value_type.value {
                    return *level as i64;
                }
            }
        }
    }
    1 // Default to h1
}

/// Get role level from properties (for separators)
fn get_role_level(node: &AxNode) -> i64 {
    if let Some(ref props) = node.properties {
        for prop in props {
            if prop.name == "level" {
                if let PropertyValueContent::Integer(level) = &prop.value_type.value {
                    return *level as i64;
                }
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_axtree() {
        let json = r#"{
            "nodes": [
                {
                    "nodeId": "1",
                    "role": {"type": "role", "value": "RootWebArea"},
                    "childIds": ["2"],
                    "ignored": false
                },
                {
                    "nodeId": "2",
                    "parentId": "1",
                    "role": {"type": "role", "value": "heading"},
                    "name": {"type": "computedString", "value": "Test Heading"},
                    "childIds": ["-1"],
                    "properties": [{"name": "level", "value": {"type": "integer", "value": 2}}]
                },
                {
                    "nodeId": "-1",
                    "parentId": "2",
                    "role": {"type": "internalRole", "value": 158},
                    "name": {"type": "computedString", "value": "Test Heading"}
                }
            ]
        }"#;

        let tree: AxTree = serde_json::from_str(json).unwrap();
        assert_eq!(tree.nodes.len(), 3);
        assert!(tree.find_root().is_some());
    }

    #[test]
    fn test_simple_heading() {
        let json = r#"{
            "nodes": [
                {
                    "nodeId": "1",
                    "role": {"type": "role", "value": "RootWebArea"},
                    "childIds": ["2"],
                    "ignored": false
                },
                {
                    "nodeId": "2",
                    "parentId": "1",
                    "role": {"type": "role", "value": "heading"},
                    "name": {"type": "computedString", "value": "Hello World"},
                    "childIds": ["-1"],
                    "properties": [{"name": "level", "value": {"type": "integer", "value": 1}}]
                },
                {
                    "nodeId": "-1",
                    "parentId": "2",
                    "role": {"type": "internalRole", "value": 158},
                    "name": {"type": "computedString", "value": "Hello World"}
                }
            ]
        }"#;

        let tree: AxTree = serde_json::from_str(json).unwrap();
        let md = axtree_to_markdown(&tree);
        assert!(md.contains("# Hello World"));
    }

    #[test]
    fn test_link_conversion() {
        let json = r#"{
            "nodes": [
                {
                    "nodeId": "1",
                    "role": {"type": "role", "value": "RootWebArea"},
                    "childIds": ["2"],
                    "ignored": false
                },
                {
                    "nodeId": "2",
                    "parentId": "1",
                    "role": {"type": "role", "value": "link"},
                    "name": {"type": "computedString", "value": "Click me"},
                    "childIds": ["-1"],
                    "properties": [{"name": "url", "value": {"type": "string", "value": "https://example.com"}}]
                },
                {
                    "nodeId": "-1",
                    "parentId": "2",
                    "role": {"type": "internalRole", "value": 158},
                    "name": {"type": "computedString", "value": "Click me"}
                }
            ]
        }"#;

        let tree: AxTree = serde_json::from_str(json).unwrap();
        let md = axtree_to_markdown(&tree);
        assert!(md.contains("[Click me](https://example.com)"));
    }

    #[test]
    fn test_paragraph_conversion() {
        let json = r#"{
            "nodes": [
                {
                    "nodeId": "1",
                    "role": {"type": "role", "value": "RootWebArea"},
                    "childIds": ["2"],
                    "ignored": false
                },
                {
                    "nodeId": "2",
                    "parentId": "1",
                    "role": {"type": "role", "value": "paragraph"},
                    "name": {"type": "computedString", "value": ""},
                    "childIds": ["-1"]
                },
                {
                    "nodeId": "-1",
                    "parentId": "2",
                    "role": {"type": "internalRole", "value": 158},
                    "name": {"type": "computedString", "value": "This is a paragraph"}
                }
            ]
        }"#;

        let tree: AxTree = serde_json::from_str(json).unwrap();
        let md = axtree_to_markdown(&tree);
        assert!(md.contains("This is a paragraph"));
    }

    #[test]
    fn test_ignored_nodes() {
        let json = r#"{
            "nodes": [
                {
                    "nodeId": "1",
                    "role": {"type": "role", "value": "RootWebArea"},
                    "childIds": ["2"],
                    "ignored": false
                },
                {
                    "nodeId": "2",
                    "parentId": "1",
                    "role": {"type": "role", "value": "none"},
                    "childIds": ["3"],
                    "ignored": true,
                    "ignoredReasons": [{"name": "uninteresting", "value": {"type": "boolean", "value": true}}]
                },
                {
                    "nodeId": "3",
                    "parentId": "2",
                    "role": {"type": "role", "value": "heading"},
                    "name": {"type": "computedString", "value": "Visible Heading"},
                    "childIds": ["-1"],
                    "properties": [{"name": "level", "value": {"type": "integer", "value": 1}}]
                },
                {
                    "nodeId": "-1",
                    "parentId": "3",
                    "role": {"type": "internalRole", "value": 158},
                    "name": {"type": "computedString", "value": "Visible Heading"}
                }
            ]
        }"#;

        let tree: AxTree = serde_json::from_str(json).unwrap();
        let md = axtree_to_markdown(&tree);
        assert!(md.contains("# Visible Heading"));
    }

    #[test]
    fn test_real_website() {
        let json = std::fs::read_to_string("./src/test_axt_nodes.json").unwrap();
        let tree: AxTree = serde_json::from_str(&json).unwrap();
        let md = axtree_to_markdown(&tree);
        assert!(!md.is_empty());
        // Check that key content is present
        assert!(md.contains("[AK](https://www.alexkehayias.com/)"));
        assert!(md.contains("# Tunnelcast"));
        assert!(md.contains("I love deck building games"));
    }
}
