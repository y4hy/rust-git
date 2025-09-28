use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use sha1::{Digest, Sha1};
use std::io::{self, Cursor, Read, Result, Write};
use std::{fs};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexEntry {
    pub ctime_seconds: u32,
    pub ctime_nanos: u32,
    pub mtime_seconds: u32,
    pub mtime_nanos: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub sha1: [u8; 20],
    pub path: Vec<u8>,
}

pub fn read_index_entries(path_str: &str) -> Result<Vec<IndexEntry>> {
    let index_content = match fs::read(path_str) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    // Check header signature
    if &index_content[0..4] != b"DIRC" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid index file signature",
        ));
    }

    let mut cursor = Cursor::new(&index_content[8..]); // Skip signature and version
    let entry_count = cursor.read_u32::<BigEndian>()?;

    let mut entries = Vec::with_capacity(entry_count as usize);
    let mut current_pos = 12; // Start after header

    for _ in 0..entry_count {
        let mut entry_cursor = Cursor::new(&index_content[current_pos..]);

        let ctime_seconds = entry_cursor.read_u32::<BigEndian>()?;
        let ctime_nanos = entry_cursor.read_u32::<BigEndian>()?;
        let mtime_seconds = entry_cursor.read_u32::<BigEndian>()?;
        let mtime_nanos = entry_cursor.read_u32::<BigEndian>()?;
        let dev = entry_cursor.read_u32::<BigEndian>()?;
        let ino = entry_cursor.read_u32::<BigEndian>()?;
        let mode = entry_cursor.read_u32::<BigEndian>()?;
        let uid = entry_cursor.read_u32::<BigEndian>()?;
        let gid = entry_cursor.read_u32::<BigEndian>()?;
        let file_size = entry_cursor.read_u32::<BigEndian>()?;

        let mut sha1 = [0u8; 20];
        entry_cursor.read_exact(&mut sha1)?;

        let path_start = current_pos + 62;
        let path_terminator = index_content[path_start..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Corrupt entry: no path terminator")
            })?;
        let path_end = path_start + path_terminator;

        let path = index_content[path_start..path_end].to_vec();

        entries.push(IndexEntry {
            ctime_seconds,
            ctime_nanos,
            mtime_seconds,
            mtime_nanos,
            dev,
            ino,
            mode,
            uid,
            gid,
            file_size,
            sha1,
            path,
        });

        let entry_len_without_padding = 62 + path_end - path_start + 1;
        let padding = (8 - (entry_len_without_padding % 8)) % 8;
        current_pos += entry_len_without_padding + padding;
    }

    Ok(entries)
}

pub fn write_index_entries(path_str: &str, entries: &[IndexEntry]) -> Result<()> {
    let mut file_content = Vec::new();

    file_content.write_all(b"DIRC")?;
    file_content.write_u32::<BigEndian>(2)?;
    file_content.write_u32::<BigEndian>(entries.len() as u32)?;

    for entry in entries {
        file_content.write_u32::<BigEndian>(entry.ctime_seconds)?;
        file_content.write_u32::<BigEndian>(entry.ctime_nanos)?;
        file_content.write_u32::<BigEndian>(entry.mtime_seconds)?;
        file_content.write_u32::<BigEndian>(entry.mtime_nanos)?;
        file_content.write_u32::<BigEndian>(entry.dev)?;
        file_content.write_u32::<BigEndian>(entry.ino)?;
        file_content.write_u32::<BigEndian>(entry.mode)?;
        file_content.write_u32::<BigEndian>(entry.uid)?;
        file_content.write_u32::<BigEndian>(entry.gid)?;
        file_content.write_u32::<BigEndian>(entry.file_size)?;
        file_content.write_all(&entry.sha1)?;

        let flags = entry.path.len() as u16;
        file_content.write_u16::<BigEndian>(flags)?;
        file_content.write_all(&entry.path)?;

        let entry_len = 62 + entry.path.len();
        let padding_len = 8 - (entry_len % 8);
        if padding_len < 8 {
            let padding = vec![0u8; padding_len];
            file_content.write_all(&padding)?;
        }
    }

    let mut hasher = Sha1::new();
    hasher.update(&file_content);
    let checksum = hasher.finalize();
    file_content.write_all(&checksum)?;

    fs::write(path_str, file_content)?;

    Ok(())
}

