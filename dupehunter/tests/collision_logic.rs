use dupehunter::{Args, Scanner, EntryType};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_local_file_clash_detection() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("Lady GaGa.mp3"), "content").unwrap();
    fs::write(dir_path.join("Lady Gaga.mp3"), "content").unwrap();

    let args = Args {
        directory: dir_path.to_path_buf(),
        recursive: false,
    };

    let mut scanner = Scanner::new(args);
    scanner.scan().unwrap();

    let collisions = &scanner.collisions[&dir_path.to_path_buf()];
    let entries = &collisions["lady gaga.mp3"];
    
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|(n, _)| n == "Lady GaGa.mp3"));
    assert!(entries.iter().all(|(_, t)| *t == EntryType::File));
}

#[test]
fn test_structural_clash_detection() {
    let root = tempdir().unwrap();
    let root_path = root.path();

    // Scenario: Artist/ vs artist/
    let path1 = root_path.join("Amtrac");
    let path2 = root_path.join("AMTRAC");
    fs::create_dir(&path1).unwrap();
    fs::create_dir(&path2).unwrap();

    let args = Args {
        directory: root_path.to_path_buf(),
        recursive: false,
    };

    let mut scanner = Scanner::new(args);
    scanner.scan().unwrap();

    let collisions = &scanner.collisions[&root_path.to_path_buf()];
    let entries = &collisions["amtrac"];
    
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|(n, t)| n == "Amtrac" && *t == EntryType::Directory));
    assert!(entries.iter().any(|(n, t)| n == "AMTRAC" && *t == EntryType::Directory));
}

#[test]
fn test_recursive_structural_clash() {
    let root = tempdir().unwrap();
    let root_path = root.path();

    let artist = root_path.join("Artist");
    fs::create_dir(&artist).unwrap();

    // Inside Artist/, we have two album folders that collide
    let album1 = artist.join("Greatest Hits");
    let album2 = artist.join("greatest hits");
    fs::create_dir(&album1).unwrap();
    fs::create_dir(&album2).unwrap();

    let args = Args {
        directory: root_path.to_path_buf(),
        recursive: true,
    };

    let mut scanner = Scanner::new(args);
    scanner.scan().unwrap();

    // Check collision inside Artist folder
    let collisions = &scanner.collisions[&artist.to_path_buf()];
    let entries = &collisions["greatest hits"];
    
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_no_clash() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("Unique1.mp3"), "c").unwrap();
    fs::create_dir(dir_path.join("UniqueDir")).unwrap();

    let args = Args {
        directory: dir_path.to_path_buf(),
        recursive: true,
    };

    let mut scanner = Scanner::new(args);
    scanner.scan().unwrap();

    assert!(scanner.collisions.is_empty());
}
