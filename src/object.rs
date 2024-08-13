
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

pub const GIT_DIRECTORY: &str = ".gitrust";
pub const OBJECTS_DIRECTORY: &str  = ".gitrust/objects";

struct Object {
sha: String,
}
impl Object {
// Constructor function for Object
pub fn new(sha: String) -> Self {
    Self { sha }
}

// The write method that takes a closure to write to the file
pub fn write<F>(&self, block: F) -> io::Result<()>
where
    F: FnOnce(&mut File) -> io::Result<()>,
{
    // Create the object directory based on the SHA
    let object_directory = format!("{}/{}", OBJECTS_DIRECTORY, &self.sha[0..2]);
    fs::create_dir_all(&object_directory)?;

    // Define the object path
    let object_path = format!("{}/{}", object_directory, &self.sha[2..]);

    // Open the file and pass it to the closure
    let mut file = File::create(object_path)?;
    block(&mut file)
}
}
impl Object {
// Private accessor for sha, equivalent to the attr_reader in Ruby
fn sha(&self) -> &str {
    &self.sha
}
}
