use std::fs;
use std::io::Result;
use std::path::Path;

pub fn init(path_str: &str) -> Result<()> {
    // Create the base path for the .git directory
    let root_path = Path::new(path_str);
    let git_dir = root_path.join(".git");

    // Check if it already exists
    if git_dir.is_dir() {
        println!("Repository already initialized at: {}", git_dir.display());
        return Ok(());
    }

    // Create the .git directory and its subdirectories
    fs::create_dir_all(git_dir.join("objects"))?;
    fs::create_dir_all(git_dir.join("refs/heads"))?;
    fs::create_dir_all(git_dir.join("refs/tags"))?;
    fs::create_dir_all(git_dir.join("hooks"))?;

    // Create the necessary files with default content
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
    fs::write(
        git_dir.join("config"),
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n",
    )?;
    fs::write(
        git_dir.join("description"),
        "Unnamed repository; edit this file 'description' to name the repository.\n",
    )?;

    Ok(())
}
