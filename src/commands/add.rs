use crate::index::{read_index_entries, write_index_entries, IndexEntry};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Result, Write};
use std::os::unix::fs::MetadataExt;

pub fn add(path_str: &str) -> Result<()> {
    let sha1_hex = hash_object(path_str)?;
    update_index(path_str, &sha1_hex)?;
    Ok(())
}

fn hash_object(path_str: &str) -> Result<String> {
    let content = fs::read(path_str)?;
    let header = format!("blob {}\0", content.len());
    let mut full_data = Vec::new();
    full_data.extend_from_slice(header.as_bytes());
    full_data.extend_from_slice(&content);

    let mut hasher = Sha1::new();
    hasher.update(&full_data);
    let sha1_bytes = hasher.finalize();
    let sha1_hex = hex::encode(sha1_bytes);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&full_data)?;
    let compressed_data = encoder.finish()?;

    let object_dir = format!(".git/objects/{}", &sha1_hex[0..2]);
    fs::create_dir_all(&object_dir)?;
    let object_path = format!("{}/{}", object_dir, &sha1_hex[2..]);
    fs::write(object_path, compressed_data)?;

    Ok(sha1_hex)
}

fn update_index(path_str: &str, sha1_hex: &str) -> Result<()> {
    let index_path = ".git/index";
    let mut entries = read_index_entries(index_path).unwrap_or_else(|_| Vec::new());

    let metadata = fs::metadata(path_str)?;
    let mut sha1_bytes = [0u8; 20];
    hex::decode_to_slice(sha1_hex, &mut sha1_bytes).expect("Invalid hex string!");

    let new_entry = IndexEntry {
        ctime_seconds: metadata.ctime() as u32,
        ctime_nanos: metadata.ctime_nsec() as u32,
        mtime_seconds: metadata.mtime() as u32,
        mtime_nanos: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        mode: metadata.mode(),
        uid: metadata.uid(),
        gid: metadata.gid(),
        file_size: metadata.len() as u32,
        sha1: sha1_bytes,
        path: path_str.as_bytes().to_vec(),
    };

    let path_bytes = path_str.as_bytes();
    entries.retain(|entry| entry.path != path_bytes);

    entries.push(new_entry);
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    write_index_entries(index_path, &entries)?;

    Ok(())
}

