use std::{env, error::Error, fs};

/// Attempts to load the contents of `.viktor/guidelines.md`.
///
/// Returns:
/// - Ok(Some(String)) ⇒ file existed (even if empty)
/// - Ok(None) ⇒ `.viktor/guidelines.md` does not exist
/// - Err(_) ⇒ I/O or env error
pub fn load_guidelines() -> Result<Option<String>, Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let path = cwd.join(".viktor").join("guidelines.md");
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    Ok(Some(content))
}

/// Loads and prints the guidelines to stdout.
///
/// If no file exists, prints a hint to run `viktor init`.
pub fn print_guidelines() -> Result<(), Box<dyn Error>> {
    match load_guidelines()? {
        Some(s) if !s.trim().is_empty() => {
            println!("{}", s);
        }
        Some(_) => {
            println!(
                "ℹ️  `.viktor/guidelines.md` exists but is empty.\n\
         Edit it to add your project context."
            );
        }
        None => {
            println!(
                "⚠️  No guidelines found at `.viktor/guidelines.md`.\n\
         Run `viktor init` to create one."
            );
        }
    }
    Ok(())
}
