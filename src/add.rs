use std::fs::{self, File, OpenOptions};
use std::io::{self, Error, Read, Write};
use std::path::{Path, PathBuf};
use sha1::{Sha1, Digest};
use flate2::write::DeflateEncoder;
use flate2::Compression;


///TODO: implement full index binary format
/*
DIRC <version_number> <number of entries>

<ctime> <mtime> <dev> <ino> <mode> <uid> <gid> <SHA> <flags> <path>
<ctime> <mtime> <dev> <ino> <mode> <uid> <gid> <SHA> <flags> <path>
<ctime> <mtime> <dev> <ino> <mode> <uid> <gid> <SHA> <flags> <path>
...
 */

/// Makes a snapshot of the working directory and adds it to the index
pub fn add(path: String) -> Result<(), Error> {
    let git_dir = ".gitrust";
    let objects_dir = format!("{}/objects", git_dir);
    let index_path = format!("{}/index", git_dir);

    if !fs::metadata(git_dir).is_ok() {
        return Err(Error::new(io::ErrorKind::AlreadyExists, "Not an initialized rusty_git repository"));
    }

    // Check if the provided path exists
    let path = Path::new(&path);
    if !path.exists() {
        return Err(Error::new(io::ErrorKind::NotFound, "The specified file or directory does not exist"));
    }

    // Recursively add files
    if path.is_file() {
        process_file(path, &objects_dir, &index_path)?;
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_file() {
                process_file(&entry_path, &objects_dir, &index_path)?;
            } else if entry_path.is_dir() {
                add(entry_path.to_str().unwrap().to_string())?;
            }
        }
    }

    Ok(())
}

fn process_file(path: &Path, objects_dir: &str, index_path: &str) -> Result<(), Error> {
    // Read file contents
    let mut file = File::open(path)?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)?;

    // Get SHA1 hash
    let mut hasher = Sha1::new();
    hasher.update(&file_contents);
    let sha = hasher.finalize();
    let sha_hex = format!("{:x}", sha);

    // Deflate the file contents
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&file_contents)?;
    let blob = encoder.finish()?;

    // Define the object path based on the SHA1 hash
    let object_directory = format!("{}/{}", objects_dir, &sha_hex[0..2]);
    fs::create_dir_all(&object_directory)?;

    // Define the blob path
    let blob_path = format!("{}/{}", object_directory, &sha_hex[2..]);
    let mut blob_file = File::create(&blob_path)?;
    blob_file.write_all(&blob)?;

    // Ensure the index file exists and append the SHA1 and file path
    let mut index_file = OpenOptions::new()
        .create(true) // Create the file if it doesn't exist
        .append(true) // Open the file in append mode
        .open(&index_path)?;

    writeln!(index_file, "{} {}", sha_hex, path.display())?;

    Ok(())
}
