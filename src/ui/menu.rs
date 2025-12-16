//! Terminal UI utilities

use crate::Result;
use dialoguer::{Confirm, Input};

/// Display welcome banner
pub fn display_banner() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║        GE DRI Protocol Parser - Prototype v0.1.0         ║");
    println!("║                                                           ║");
    println!("║  Compatible with: CARESCAPE B650/B850, S/5 Monitors      ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
}

/// Confirm action with user
pub fn confirm(prompt: &str) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(prompt)
        .default(true)
        .interact()?)
}

/// Get string input from user
pub fn get_input(prompt: &str, default: &str) -> Result<String> {
    Ok(Input::new()
        .with_prompt(prompt)
        .default(default.to_string())
        .interact_text()?)
}

/// Display progress message
pub fn progress(message: &str) {
    println!("⏳ {}", message);
}

/// Display success message
pub fn success(message: &str) {
    println!("✅ {}", message);
}

/// Display error message
pub fn error(message: &str) {
    eprintln!("❌ {}", message);
}

/// Display info message
pub fn info(message: &str) {
    println!("ℹ️  {}", message);
}
