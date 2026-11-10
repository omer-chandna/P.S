//! Local prompt-optimization core for P.S.
//!
//! The desktop shell, global shortcuts, and clipboard I/O live outside this crate.
//! Nothing here stores prompt text or contacts a non-local endpoint.

pub mod clipboard;
pub mod endpoint;
pub mod error;
pub mod hardware;
pub mod models;
pub mod ollama;
pub mod operation;
pub mod optimizer;
pub mod preservation;
pub mod prompts;
pub mod settings;
pub mod shortcuts;
pub mod workflow;

pub use error::{PsError, PsResult};
pub use optimizer::{optimize_prompt, OptimizationAnalysis, OptimizationStatus};
pub use settings::AppSettings;
