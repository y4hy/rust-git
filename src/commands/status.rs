use crate::index::read_index_entries;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs;
use std::io::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn status() -> Result<()> {
    let head_content = fs::read_to_string(".git/HEAD")?;
    let branch = head_content.trim().split('/').next_back().unwrap_or("master");
    println!("On branch {}", branch);

    let index_entries = read_index_entries(".git/index")?;
    let index_files: HashMap<PathBuf, String> = index_entries
        .into_iter()
        .map(|entry| {
            let path = PathBuf::from(String::from_utf8_lossy(&entry.path).into_owned());
            let sha1 = hex::encode(entry.sha1);
            (path, sha1)
        })
        .collect();

    let workspace_files = walk_workspace()?;
    let mut untracked_files = Vec::new();
    let mut changes_to_commit = Vec::new();

    for path in workspace_files.keys() {
        if !index_files.contains_key(path) {
            untracked_files.push(path.display().to_string());
        }
    }

    for path in index_files.keys() {
        changes_to_commit.push(format!("new file:   {}", path.display()));
    }

    if !changes_to_commit.is_empty() {
        println!("\nChanges to be committed:");
        for file in &changes_to_commit {
            println!("\t{}", file);
        }
    }

    if !untracked_files.is_empty() {
        println!("\nUntracked files:");
        for file in &untracked_files {
            println!("\t{}", file);
        }
    }

    if untracked_files.is_empty() && index_files.is_empty() {
        println!("\nnothing to commit, working tree clean");
    }

    Ok(())
}

fn walk_workspace() -> Result<HashMap<PathBuf, String>> {
    let mut workspace_files = HashMap::new();
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| !e.path().starts_with("./.git") && !e.path().starts_with("./target"))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let content = fs::read(path)?;
            let header = format!("blob {}\0", content.len());

            let mut hasher = Sha1::new();
            hasher.update(header.as_bytes());
            hasher.update(&content);
            let sha1_hex = hex::encode(hasher.finalize());

            let relative_path = path.strip_prefix("./").unwrap_or(path);
            workspace_files.insert(relative_path.to_path_buf(), sha1_hex);
        }
    }
    Ok(workspace_files)
}

