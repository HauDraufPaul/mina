pub mod system;
pub mod network;
pub mod process;
pub mod homebrew;
pub mod system_utils;
pub mod ollama;
pub mod news;

pub use system::SystemProvider;
pub use network::NetworkProvider;
pub use process::ProcessProvider;
pub use homebrew::HomebrewProvider;
pub use system_utils::SystemUtilsProvider;
pub use ollama::{OllamaProvider, OllamaModel, ChatMessage};

