use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=extra-syntax-files");

    if env::var("CARGO_FEATURE_EXTRA_SYNTAX_FILES").is_ok() {
        install_syntax_files()?;
    }

    Ok(())
}

fn install_syntax_files() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Installing iedit syntax files...");

    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let target_dir = home_dir.join(".config/iedit");
    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let syntax_dir = crate_root
        .ancestors()
        .nth(1)
        .to_owned()
        .unwrap()
        .join("syntax");

    // Create target directory if it doesn't exist
    if !target_dir.exists() {
        Command::new("mkdir").arg("-p").arg(&target_dir).status()?;
        eprintln!("Created directory: {}", target_dir.display());
    }

    Command::new("cp")
        .arg("-r")
        .arg(&syntax_dir)
        .arg(&target_dir)
        .status()?;

    // Update config file (keeping this part the same)
    let config_path = home_dir.join(".iedit.conf");
    update_config_file(
        &config_path,
        &target_dir,
    )?;

    Ok(())
}

// update_config_file function remains the same as before
fn update_config_file(
    config_path: &PathBuf,
    syntax_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Write;

    let absolute_syntax_path =
        fs::canonicalize(syntax_dir).unwrap_or_else(|_| syntax_dir.to_path_buf());

    let config_line = format!(
        "syntax_highlighting_dir = {}/syntax\n",
        absolute_syntax_path.display()
    );

    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let mut lines: Vec<String> = content.lines().map(String::from).collect();

        let mut found = false;
        for line in lines.iter_mut() {
            if line.trim_start().starts_with("syntax_highlighting_dir") {
                *line = config_line.trim().to_string();
                found = true;
                break;
            }
        }

        if !found {
            lines.push(config_line);
        }

        fs::write(config_path, lines.join("\n"))?;
        eprintln!("Updated config file: {}", config_path.display());
    } else {
        let mut file = fs::File::create(config_path)?;
        file.write_all(config_line.as_bytes())?;
        eprintln!("Created config file: {}", config_path.display());
    }

    Ok(())
}
