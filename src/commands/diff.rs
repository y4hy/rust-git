use crate::index::read_index_entries;
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::{self, Read, Result};

fn read_blob_content(sha_hex: &str) -> Result<Vec<u8>> {
    let object_path = format!(".git/objects/{}/{}", &sha_hex[0..2], &sha_hex[2..]);
    let compressed = fs::read(object_path)?;
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;

    if let Some(pos) = data.iter().position(|&b| b == 0) {
        Ok(data[pos + 1..].to_vec())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid blob object: missing header terminator",
        ))
    }
}

#[derive(Debug)]
enum Op<'a> {
    Keep(&'a str),
    Add(&'a str),
    Remove(&'a str),
}

fn diff_lines<'a>(old_text: &'a str, new_text: &'a str) -> Vec<Op<'a>> {
    let old: Vec<&str> = old_text.lines().collect();
    let new: Vec<&str> = new_text.lines().collect();
    let m = old.len();
    let n = new.len();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..m {
        for j in 0..n {
            if old[i] == new[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    let mut ops = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            ops.push(Op::Keep(old[i - 1]));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            ops.push(Op::Remove(old[i - 1]));
            i -= 1;
        } else {
            ops.push(Op::Add(new[j - 1]));
            j -= 1;
        }
    }
    while i > 0 {
        ops.push(Op::Remove(old[i - 1]));
        i -= 1;
    }
    while j > 0 {
        ops.push(Op::Add(new[j - 1]));
        j -= 1;
    }

    ops.reverse();
    ops
}

pub fn diff() -> Result<()> {
    let entries = read_index_entries(".git/index")?;

    for e in entries {
        let path_str = String::from_utf8_lossy(&e.path).into_owned();
        let tracked_sha = hex::encode(e.sha1);
        let old_content = match read_blob_content(&tracked_sha) {
            Ok(c) => c,
            Err(_) => Vec::new(),
        };
        let new_content = fs::read(&path_str).unwrap_or_default();

        if old_content == new_content {
            continue; // no changes
        }

        let old_text = String::from_utf8_lossy(&old_content);
        let new_text = String::from_utf8_lossy(&new_content);

        println!("diff --gir {}", &path_str);
        println!("--- a/{}", &path_str);
        println!("+++ b/{}", &path_str);

        let ops = diff_lines(&old_text, &new_text);
        for op in ops {
            match op {
                Op::Add(s) => println!("+{}", s),
                Op::Remove(s) => println!("-{}", s),
                Op::Keep(_) => {}
            }
        }
    }

    Ok(())
}
