use std::fs;
use std::io::Error;


/// Initializes a new rusty_git repository
pub fn init() -> Result<(), Error> {
    let git_dir = ".gitrust";
    let objects_dir = format!("{}/objects", git_dir);
    let refs_dir = format!("{}/refs", git_dir);


    if fs::metadata(git_dir).is_ok() {
        return Err(Error::new(std::io::ErrorKind::AlreadyExists, "Already a git repository"));
    }

    fs::create_dir(git_dir)?;
    build_objects_dir(&objects_dir)?;
    build_refs_dir(&refs_dir)?;
    initialize_head(&git_dir)?;

    //print success message
    println!("Initialized empty git repository in {}", git_dir);
    Ok(())
}


fn build_objects_dir(objects_dir: &str) -> Result<(), Error> {
    fs::create_dir(objects_dir)?;
    fs::create_dir(format!("{}/info", objects_dir))?;
    fs::create_dir(format!("{}/pack", objects_dir))?;
    Ok(())
}

fn build_refs_dir(refs_dir: &str) -> Result<(), Error> {
    fs::create_dir(refs_dir)?;
    fs::create_dir(format!("{}/heads", refs_dir))?;
    fs::create_dir(format!("{}/tags", refs_dir))?;
    Ok(())
}

fn initialize_head(refs_dir: &str) -> Result<(), Error> {
    let head = format!("{}/HEAD", refs_dir);
    fs::write(head, "ref: refs/heads/master\n")?;
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::fs;
    use std::io::ErrorKind;

    #[test]
    fn test_build_objects_dir() {
        let objects_dir = "test_objects";
        let result = build_objects_dir(objects_dir);
        assert!(result.is_ok());
        fs::remove_dir_all(objects_dir).unwrap();
    }

    #[test]
    fn test_build_refs_dir() {
        let refs_dir = "test_refs";
        let result = build_refs_dir(refs_dir);
        assert!(result.is_ok());
        fs::remove_dir_all(refs_dir).unwrap();
    }

    #[test]
    fn test_initialize_head() {
        let refs_dir = "test_refs";
        let result = build_refs_dir(refs_dir);
        assert!(result.is_ok());
        let result = initialize_head(refs_dir);
        assert!(result.is_ok());
        fs::remove_dir_all(refs_dir).unwrap();
    }

    #[test]
    fn test_init() {
        let git_dir = ".gitrust";
        let result = init();
        assert!(result.is_ok());
        assert!(fs::metadata(git_dir).is_ok());
        fs::remove_dir_all(git_dir).unwrap();
    }

    #[test]
    fn test_init_already_exists() {
        let git_dir = ".gitrust";
        fs::create_dir(git_dir).unwrap();
        let result = init();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), ErrorKind::AlreadyExists);
        fs::remove_dir_all(git_dir).unwrap();
    }

}