use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cost {
    pub prompt: f64,
    pub completion: f64,
    pub request: f64,
}

impl Cost {
    pub fn new(prompt: f64, completion: f64, request: f64) -> Self {
        Self {
            prompt,
            completion,
            request,
        }
    }

    pub fn from_modelsdev(input_per_million: f64, output_per_million: f64) -> Self {
        Self {
            prompt: input_per_million / 1_000_000.0,
            completion: output_per_million / 1_000_000.0,
            request: 0.0,
        }
    }

    pub fn calculate(&self, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        self.prompt * prompt_tokens as f64
            + self.completion * completion_tokens as f64
            + self.request
    }
}

impl Default for Cost {
    fn default() -> Self {
        Self {
            prompt: 0.0,
            completion: 0.0,
            request: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}
