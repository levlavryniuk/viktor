use std::{env, error::Error, fs, path::PathBuf};

pub struct ViktorInit {
    cwd: PathBuf,
    viktor_dir: PathBuf,
}

impl ViktorInit {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let cwd = env::current_dir()?;
        let viktor_dir = cwd.join(".viktor");

        Ok(Self { cwd, viktor_dir })
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        if self.is_already_initialized() {
            self.handle_existing_project()?;
            return Ok(());
        }

        self.create_viktor_directory()?;
        self.create_guidelines_file()?;
        self.update_gitignore()?;
        self.print_success_message();

        Ok(())
    }

    fn is_already_initialized(&self) -> bool {
        self.viktor_dir.exists()
    }

    fn handle_existing_project(&self) -> Result<(), Box<dyn Error>> {
        println!(
            "✓ Viktor project already initialized in {}",
            self.viktor_dir.display()
        );

        let guidelines_path = self.viktor_dir.join("guidelines.md");
        if !guidelines_path.exists() {
            println!("⚠️  Missing guidelines.md, creating empty file...");
            self.create_guidelines_file()?;
        }

        Ok(())
    }

    fn create_viktor_directory(&self) -> Result<(), Box<dyn Error>> {
        fs::create_dir(&self.viktor_dir)?;
        println!("✓ Created .viktor directory");
        Ok(())
    }

    fn create_guidelines_file(&self) -> Result<(), Box<dyn Error>> {
        let guidelines_path = self.viktor_dir.join("guidelines.md");
        fs::write(&guidelines_path, "")?;
        println!("✓ Created empty guidelines.md");

        Ok(())
    }

    fn update_gitignore(&self) -> Result<(), Box<dyn Error>> {
        let gitignore_path = self.cwd.join(".gitignore");
        let viktor_entry = ".viktor/\n";

        if gitignore_path.exists() {
            let content = fs::read_to_string(&gitignore_path)?;
            if !content.contains(".viktor") {
                fs::write(&gitignore_path, format!("{}{}", content, viktor_entry))?;
                println!("✓ Added .viktor/ to existing .gitignore");
            } else {
                println!("✓ .viktor/ already in .gitignore");
            }
        } else {
            fs::write(&gitignore_path, viktor_entry)?;
            println!("✓ Created .gitignore with .viktor/ entry");
        }

        Ok(())
    }

    fn print_success_message(&self) {
        let guidelines_path = self.viktor_dir.join("guidelines.md");

        println!("\n🎉 Viktor project initialized!");
        println!("📝 Edit your guidelines: {}", guidelines_path.display());
        println!("\n💡 Add project context to guidelines.md for better AI assistance.");
    }
}
