use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConversationRole {
    User,
    Assistant,
    System,
    Function,
    Tool,
}

impl std::fmt::Display for ConversationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversationRole::User => write!(f, "user"),
            ConversationRole::Assistant => write!(f, "assistant"),
            ConversationRole::System => write!(f, "system"),
            ConversationRole::Function => write!(f, "function"),
            ConversationRole::Tool => write!(f, "tool"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub role: ConversationRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(flatten)]
    pub data: ContentEntryData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentEntryData {
    Text { text: String },
    Image { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTurn {
    pub role: ConversationRole,
    pub content: Vec<ContentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TurnType {
    Basic(Turn),
    Content(ContentTurn),
    ToolOutput {
        role: ConversationRole,
        content: Option<String>,
        tool_call_id: String,
    },
    ToolCalls {
        role: ConversationRole,
        content: Option<String>,
        tool_calls: Vec<ToolCall>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub turns: Vec<TurnType>,
}

impl Conversation {
    pub fn new() -> Self {
        Self { turns: Vec::new() }
    }

    pub fn with_turns(turns: Vec<TurnType>) -> Self {
        Self { turns }
    }

    pub fn add_turn(&mut self, turn: TurnType) {
        self.turns.push(turn);
    }

    pub fn is_last_turn_vision_query(&self) -> bool {
        if let Some(TurnType::Content(content_turn)) = self.turns.last() {
            content_turn
                .content
                .iter()
                .any(|entry| matches!(entry.data, ContentEntryData::Image { .. }))
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.turns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}
