use std::fs;
use std::path::Path;
use clap_markdown::MarkdownOptions;
use zerv::cli::parser::Cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("generate-docs") => {
            let markdown = clap_markdown::help_markdown_custom::<Cli>(&MarkdownOptions::new());

            // Use provided path or default to docs/AUTO.md
            let output_path = args.get(2).map(|s| s.as_str()).unwrap_or("docs/AUTO.md");

            // Create parent directory if it doesn't exist
            if let Some(parent) = Path::new(output_path).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).expect("Failed to create output directory");
                }
            }

            // Write to file
            fs::write(output_path, markdown).expect("Failed to write CLI documentation");
            println!("Generated CLI documentation: {}", output_path);
        }
        _ => {
            eprintln!("Usage: cargo xtask <TASK> [OPTIONS]");
            eprintln!("Tasks:");
            eprintln!("  generate-docs [PATH]    Generate CLI documentation (default: docs/AUTO.md)");
        }
    }
}
