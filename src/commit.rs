use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::collections::HashMap;
use std::process::Command;
use sha1::{Sha1, Digest};
use chrono::prelude::*;
use std::path::Path;

pub const GIT_DIRECTORY: &str = ".gitrust";
pub const OBJECTS_DIRECTORY: &str = ".gitrust/objects";

// Define the Object struct directly in commit.rs
struct Object {
    sha: String,
}

impl Object {
    pub fn new(sha: String) -> Self {
        Self { sha }
    }

    pub fn write<F>(&self, block: F) -> io::Result<()>
    where
        F: FnOnce(&mut File) -> io::Result<()>,
    {
        let object_directory = format!("{}/{}", OBJECTS_DIRECTORY, &self.sha[0..2]);
        fs::create_dir_all(&object_directory)?;

        let object_path = format!("{}/{}", object_directory, &self.sha[2..]);

        let mut file = File::create(object_path)?;
        block(&mut file)
    }

    fn sha(&self) -> &str {
        &self.sha
    }
}

// Define an enum to represent either a file or a directory
enum TreeEntry {
    Blob(String), // File, represented by its SHA
    Tree(HashMap<String, TreeEntry>), // Directory, containing more TreeEntries
}

const INDEX_PATH: &str = ".gitrust/index";
const COMMIT_MESSAGE_TEMPLATE: &str = "# Title\n#\n# Body";

fn index_files() -> io::Result<Vec<String>> {
    let file = File::open(INDEX_PATH)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

fn index_tree() -> io::Result<HashMap<String, TreeEntry>> {
    let mut root = HashMap::new();
    for line in index_files()? {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        let (sha, path) = (parts[0], parts[2]);
        let segments: Vec<&str> = path.split('/').collect();
        let mut current = &mut root;

        for (i, segment) in segments.iter().enumerate() {
            if i == segments.len() - 1 {
                // This is the last segment, so it's a file
                current.insert(segment.to_string(), TreeEntry::Blob(sha.to_string()));
            } else {
                // This is a directory, keep traversing
                current = match current.entry(segment.to_string()) {
                    std::collections::hash_map::Entry::Occupied(entry) => {
                        if let TreeEntry::Tree(ref mut map) = entry.into_mut() {
                            map
                        } else {
                            panic!("Unexpected entry type, expected Tree");
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(TreeEntry::Tree(HashMap::new()));
                        if let TreeEntry::Tree(ref mut map) = current.get_mut(&segment.to_string()).unwrap() {
                            map
                        } else {
                            unreachable!();
                        }
                    }
                };
            }
        }
    }
    Ok(root)
}



fn build_tree(name: &str, tree: &HashMap<String, TreeEntry>) -> io::Result<String> {
    let sha = generate_sha1(name);
    let object = Object::new(sha.clone());

    object.write(|file| {
        for (key, entry) in tree {
            match entry {
                TreeEntry::Blob(sha) => {
                    writeln!(file, "blob {} {}", sha, key)?;
                }
                TreeEntry::Tree(subtree) => {
                    let dir_sha = build_tree(key, subtree)?;
                    writeln!(file, "tree {} {}", dir_sha, key)?;
                }
            }
        }
        Ok(())
    })?;

    Ok(sha)
}

fn build_commit(tree: &str) -> io::Result<String> {
    let commit_message_path = format!("{}/COMMIT_EDITMSG", GIT_DIRECTORY);

    fs::write(&commit_message_path, COMMIT_MESSAGE_TEMPLATE)?;

    Command::new("sh")
        .arg("-c")
        .arg(format!("$VISUAL {}", commit_message_path))
        .status()?;

    let message = fs::read_to_string(&commit_message_path)?;
    let committer = "user";
    let sha = generate_sha1(committer);
    let object = Object::new(sha.clone());

    object.write(|file| {
        writeln!(file, "tree {}", tree)?;
        writeln!(file, "author {}", committer)?;
        writeln!(file)?;
        writeln!(file, "{}", message)?;
        Ok(())
    })?;

    Ok(sha)
}

fn update_ref(commit_sha: &str) -> io::Result<()> {
    let head_path = format!("{}/HEAD", GIT_DIRECTORY);
    let head_content = fs::read_to_string(&head_path)?;
    let current_branch = head_content.trim().split_whitespace().last().unwrap();
    let branch_path = format!("{}/{}", GIT_DIRECTORY, current_branch);

    let mut file = File::create(branch_path)?;
    write!(file, "{}", commit_sha)?;

    Ok(())
}

fn clear_index() -> io::Result<()> {
    File::create(INDEX_PATH)?.set_len(0)?;
    Ok(())
}

fn generate_sha1(data: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(Utc::now().to_rfc3339());
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn commit() -> io::Result<()> {
    if index_files()?.is_empty() {
        eprintln!("Nothing to commit");
        return Err(io::Error::new(io::ErrorKind::Other, "Nothing to commit"));
    }

    let root_tree = index_tree()?;
    let root_sha = build_tree("root", &root_tree)?;
    let commit_sha = build_commit(&root_sha)?;
    update_ref(&commit_sha)?;
    clear_index()?;

    Ok(())
}
