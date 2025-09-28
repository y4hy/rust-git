use crate::index::read_index_entries;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Result, Write};
use std::path::PathBuf;

fn write_object(object_type: &str, content: &[u8]) -> Result<String> {
    let mut full = Vec::with_capacity(content.len() + 64);
    let header = format!("{} {}\0", object_type, content.len());
    full.extend_from_slice(header.as_bytes());
    full.extend_from_slice(content);

    let mut hasher = Sha1::new();
    hasher.update(&full);
    let sha1_hex = hex::encode(hasher.finalize());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&full)?;
    let compressed = encoder.finish()?;

    let dir = format!(".git/objects/{}", &sha1_hex[0..2]);
    fs::create_dir_all(&dir)?;
    let path = format!("{}/{}", dir, &sha1_hex[2..]);
    fs::write(path, compressed)?;

    Ok(sha1_hex)
}

pub fn commit(message: &str) -> Result<()> {
    let entries = read_index_entries(".git/index")?;
    if entries.is_empty() {
        println!("nothing to commit");
        return Ok(());
    }

    let mut tree_text = String::new();
    for e in &entries {
        let path_str = String::from_utf8_lossy(&e.path);
        let sha_hex = hex::encode(e.sha1);
        // Simplified tree format: mode path\tsha
        tree_text.push_str(&format!("100644 {}\t{}\n", path_str, sha_hex));
    }
    let tree_sha = write_object("tree", tree_text.as_bytes())?;

    let head_content = fs::read_to_string(".git/HEAD")?;
    let head_ref = head_content
        .strip_prefix("ref: ")
        .map(|s| s.trim())
        .unwrap_or("refs/heads/main");
    let branch_name = head_ref.split('/').next_back().unwrap_or("main").to_string();

    let parent_path = PathBuf::from(".git").join(head_ref);
    let parent_sha = fs::read_to_string(&parent_path).ok().map(|s| s.trim().to_string());

    // Build commit object
    let mut commit_text = String::new();
    commit_text.push_str(&format!("tree {}\n", tree_sha));
    if let Some(parent) = &parent_sha {
        commit_text.push_str(&format!("parent {}\n", parent));
    }
    commit_text.push('\n');
    commit_text.push_str(message);
    commit_text.push('\n');

    let commit_sha = write_object("commit", commit_text.as_bytes())?;

    // Update branch ref
    if let Some(parent_dir) = parent_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(&parent_path, format!("{}\n", commit_sha))?;

    println!(
        "[{} {}] {}",
        branch_name,
        &commit_sha[..7.min(commit_sha.len())],
        message
    );

    Ok(())
}
