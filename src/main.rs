use std::fs;
use std::path::Path;
use clap::builder::OsStr;
use clap::Parser;
use dialoguer::{Confirm, Input};
use git2::Repository;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	#[arg(short, long, default_value = "/home/Rime/IdeaProjects/TemplateMod")]
	repo_to_clone: Option<String>,

	#[arg(short, long)]
	mod_name: Option<String>,
}

const MOD_NAME: &str = "Template Mod";
const MOD_CLASS_NAME: &str = "TemplateMod";
const MOD_NAMESPACE: &str = "templatemod";

fn main() {
	let cli = Args::parse();

	let url = cli.repo_to_clone.unwrap(); // Since there is a default value, this will always be Some

	let mod_name = cli.mod_name.unwrap_or_else(|| {
		Input::new()
			.with_prompt("Enter the mod name")
			.interact_text()
			.unwrap()
	});
	let mod_class_name = mod_name.replace(" ", "");
	let mod_namespace = mod_class_name.to_lowercase();

	let directory = match Path::new(&url).parent() {
		None => {
			eprintln!("Invalid repository URL");
			std::process::exit(1);
		}
		Some(value ) => {
			value.join(&mod_name)
		}
	};

	if Confirm::new()
		.with_prompt(format!("Clone the repository into '{}' with the name '{}'?", directory.to_str().unwrap(), mod_name))
		.interact()
		.unwrap() {
		match Repository::clone(&url, &directory) {
			Ok(_) => {
				println!("Cloned repository")
			}
			Err(value) => {
				eprintln!("Failed to clone repository: {}", value.message());
				clean_up_directory(&directory);
				std::process::exit(1);
			}
		}
	} else {
		main();
	}

	// Clean git information since we don't need the template mod's git information
	let git_dir = directory.join(".git");
	if git_dir.exists() {
		fs::remove_dir_all(&git_dir).expect("Failed to remove .git directory");
	}
	let build_dir = directory.join("build");
	if build_dir.exists() {
		fs::remove_dir_all(&build_dir).expect("Failed to remove build directory");
	}
	let run_dir = directory.join("run");
	if run_dir.exists() {
		fs::remove_dir_all(&run_dir).expect("Failed to remove run directory");
	}
	let idea_dir = directory.join(".idea");
	if idea_dir.exists() {
		fs::remove_dir_all(&idea_dir).expect("Failed to remove .idea directory");
	}
	let dot_gradle_dir = directory.join(".gradle");
	if dot_gradle_dir.exists() {
		fs::remove_dir_all(&dot_gradle_dir).expect("Failed to remove .gradle directory");
	}

	// Replaces string contents first
	for entry in WalkDir::new(&directory).into_iter().filter_map(|e| e.ok()) {
		let path = entry.path();
		if path.extension() == Some(&OsStr::from("jar")) {
			continue
		}
		if path.is_file() {
			let content = match fs::read_to_string(path) {
				Ok(val) => { val }
				Err(err) => {
					eprintln!("Failed to read file '{}': {}", path.to_str().unwrap(), err);
					clean_up_directory(&directory);
					std::process::exit(1);
				}
			};
			let new_content = content
				.replace(MOD_CLASS_NAME, &mod_class_name)
				.replace(MOD_NAMESPACE, &mod_namespace)
				.replace(MOD_NAME, &mod_name);
			match fs::write(path, new_content) {
				Ok(_) => {}
				Err(error) => {
					eprintln!("Failed to write file '{}': {}", path.to_str().unwrap(), error);
					clean_up_directory(&directory);
					std::process::exit(1);
				}
			}
		}
	}

	// Collect paths to rename
	let mut rename_paths = Vec::new();
	for entry in WalkDir::new(&directory).into_iter().filter_map(|e| e.ok()) {
		let path = entry.path();
		if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
			let mut new_file_name = file_name.to_string();

			if new_file_name.contains(MOD_CLASS_NAME) {
				new_file_name = new_file_name.replace(MOD_CLASS_NAME, &mod_class_name);
			}
			if new_file_name.contains(MOD_NAMESPACE) {
				new_file_name = new_file_name.replace(MOD_NAMESPACE, &mod_namespace);
			}
			if new_file_name.contains(MOD_NAME) {
				new_file_name = new_file_name.replace(MOD_NAME, &mod_name);
			}

			if new_file_name != file_name {
				let new_path = path.with_file_name(new_file_name);
				rename_paths.push((path.to_path_buf(), new_path));
			}
		}
	}

	// Sort paths to rename directories after their contents
	rename_paths.sort_by(|a, b| b.0.cmp(&a.0));

	// Rename files
	for (old_path, new_path) in rename_paths {
		match fs::rename(&old_path, &new_path) {
			Ok(_) => {}
			Err(error) => {
				eprintln!("Failed to rename file from '{}' to '{}': {}", old_path.to_str().unwrap(), new_path.to_str().unwrap(), error);
				clean_up_directory(&directory);
				std::process::exit(1);
			}
		}
	}
}

/**
 * Cleans up the directory after a failure so the user can try again without having to manually delete the directory
 */
fn clean_up_directory(directory: &Path) {
	fs::remove_dir_all(directory).expect("Failed to remove directory");
}
