use martian_adapters::{
    ContentEntry, ContentEntryData, ContentTurn, Conversation, ConversationRole, FunctionCall,
    ImageUrl, ToolCall, Turn, TurnType,
};

fn main() {
    println!("🗨️  Conversation Building Examples\n");

    println!("1. Simple Text Conversation:");
    let mut simple_conv = Conversation::new();
    simple_conv.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::System,
        content: "You are a helpful assistant.".to_string(),
    }));
    simple_conv.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::User,
        content: "What is the capital of France?".to_string(),
    }));
    simple_conv.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::Assistant,
        content: "The capital of France is Paris.".to_string(),
    }));
    println!("   Turns: {}", simple_conv.len());
    println!(
        "   Is vision query: {}\n",
        simple_conv.is_last_turn_vision_query()
    );

    println!("2. Multi-modal Conversation (with image):");
    let mut vision_conv = Conversation::new();
    vision_conv.add_turn(TurnType::Content(ContentTurn {
        role: ConversationRole::User,
        content: vec![
            ContentEntry {
                entry_type: "text".to_string(),
                data: ContentEntryData::Text {
                    text: "What do you see in this image?".to_string(),
                },
            },
            ContentEntry {
                entry_type: "image_url".to_string(),
                data: ContentEntryData::Image {
                    image_url: ImageUrl {
                        url: "data:image/png;base64,iVBORw0KGgo...".to_string(),
                        detail: Some("high".to_string()),
                    },
                },
            },
        ],
    }));
    println!("   Turns: {}", vision_conv.len());
    println!(
        "   Is vision query: {}\n",
        vision_conv.is_last_turn_vision_query()
    );

    println!("3. Tool Call Conversation:");
    let mut tool_conv = Conversation::new();
    tool_conv.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::User,
        content: "What's the weather in Paris?".to_string(),
    }));
    tool_conv.add_turn(TurnType::ToolCalls {
        role: ConversationRole::Assistant,
        content: None,
        tool_calls: vec![ToolCall {
            id: "call_abc123".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: r#"{"location": "Paris"}"#.to_string(),
            },
        }],
    });
    tool_conv.add_turn(TurnType::ToolOutput {
        role: ConversationRole::Tool,
        content: Some(r#"{"temperature": 18, "condition": "Cloudy"}"#.to_string()),
        tool_call_id: "call_abc123".to_string(),
    });
    tool_conv.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::Assistant,
        content: "The weather in Paris is currently 18°C and cloudy.".to_string(),
    }));
    println!("   Turns: {}", tool_conv.len());
    println!("   Contains tool calls: true\n");

    println!("4. Serialization Example:");
    let json = serde_json::to_string_pretty(&simple_conv).unwrap();
    println!("   JSON representation:");
    println!("{}\n", json);

    println!("5. Conversation Properties:");
    println!("   Simple conversation length: {}", simple_conv.len());
    println!("   Is empty: {}", simple_conv.is_empty());
    println!(
        "   Vision conversation is vision query: {}",
        vision_conv.is_last_turn_vision_query()
    );
}
