pub mod chat;
pub mod recommender;
pub mod routing_assist;

pub struct AIEngine {
    model_available: bool,
}

impl AIEngine {
    pub fn new() -> Self {
        let models_dir = dirs_or_default();
        let has_model = check_models(&models_dir);
        Self { model_available: has_model }
    }

    pub fn is_available(&self) -> bool {
        self.model_available
    }
}

fn dirs_or_default() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/.tracecraft/models", home)
}

fn check_models(dir: &str) -> bool {
    let paths = ["recommender.onnx", "chat.gguf"];
    paths.iter().any(|p| std::path::Path::new(dir).join(p).exists())
}
