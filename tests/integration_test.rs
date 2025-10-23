use martian_adapters::{
    delete_none_values, Conversation, ConversationRole, Cost, ModelCapabilities, TokenUsage, Turn,
    TurnType,
};
use serde_json::json;

#[test]
fn test_conversation_creation() {
    let mut conversation = Conversation::new();
    assert_eq!(conversation.len(), 0);
    assert!(conversation.is_empty());

    conversation.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::User,
        content: "Hello".to_string(),
    }));

    assert_eq!(conversation.len(), 1);
    assert!(!conversation.is_empty());
}

#[test]
fn test_cost_calculation() {
    let cost = Cost::new(0.000001, 0.000002, 0.0);
    let total = cost.calculate(1000, 500);
    assert_eq!(total, 0.000001 * 1000.0 + 0.000002 * 500.0);
}

#[test]
fn test_cost_from_modelsdev() {
    let cost = Cost::from_modelsdev(1.5, 6.0);
    assert_eq!(cost.prompt, 1.5 / 1_000_000.0);
    assert_eq!(cost.completion, 6.0 / 1_000_000.0);
    assert_eq!(cost.request, 0.0);
}

#[test]
fn test_token_usage() {
    let usage = TokenUsage::new(100, 50);
    assert_eq!(usage.prompt_tokens, 100);
    assert_eq!(usage.completion_tokens, 50);
    assert_eq!(usage.total_tokens, 150);
}

#[test]
fn test_delete_none_values() {
    let mut value = json!({
        "a": 1,
        "b": null,
        "c": {
            "d": 2,
            "e": null,
        },
        "f": [1, null, 3],
    });

    delete_none_values(&mut value);

    assert_eq!(
        value,
        json!({
            "a": 1,
            "c": {
                "d": 2,
            },
            "f": [1, null, 3],
        })
    );
}

#[test]
fn test_model_capabilities_default() {
    let capabilities = ModelCapabilities::default();
    assert!(capabilities.supports_user);
    assert!(capabilities.supports_streaming);
    assert!(capabilities.supports_system);
    assert!(!capabilities.supports_vision);
    assert!(!capabilities.supports_tools);
}

#[test]
fn test_conversation_role_display() {
    assert_eq!(ConversationRole::User.to_string(), "user");
    assert_eq!(ConversationRole::Assistant.to_string(), "assistant");
    assert_eq!(ConversationRole::System.to_string(), "system");
}
