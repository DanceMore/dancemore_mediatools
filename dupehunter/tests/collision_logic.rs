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

    let canonical_root = fs::canonicalize(dir_path).unwrap();
    let collisions = &scanner.collisions[&canonical_root];
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

    let canonical_root = fs::canonicalize(root_path).unwrap();
    let collisions = &scanner.collisions[&canonical_root];
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
    let canonical_artist = fs::canonicalize(&artist).unwrap();
    let collisions = &scanner.collisions[&canonical_artist];
    let entries = &collisions["greatest hits"];
    
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_transitive_collision() {
    let root = tempdir().unwrap();
    let root_path = root.path();

    // Scenario:
    // Artist/Song.mp3
    // artist/song.mp3
    // This is a double collision: the directory AND the file.
    let dir1 = root_path.join("Artist");
    let dir2 = root_path.join("artist");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();
    
    fs::write(dir1.join("Song.mp3"), "c1").unwrap();
    fs::write(dir2.join("song.mp3"), "c2").unwrap();

    let args = Args {
        directory: root_path.to_path_buf(),
        recursive: true,
    };

    let mut scanner = Scanner::new(args);
    scanner.scan().unwrap();

    // 1. Should detect the directory collision in the root
    let canonical_root = fs::canonicalize(root_path).unwrap();
    let root_collisions = &scanner.collisions[&canonical_root];
    assert!(root_collisions.contains_key("artist"));
    
    // 2. Both subdirectories should be scanned but won't have internal collisions
    let canonical_dir1 = fs::canonicalize(&dir1).unwrap();
    let canonical_dir2 = fs::canonicalize(&dir2).unwrap();
    
    assert!(!scanner.collisions.contains_key(&canonical_dir1));
    assert!(!scanner.collisions.contains_key(&canonical_dir2));
}

#[test]
fn test_cross_type_collision() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Scenario: A folder and a file with same name
    // song.mp3 (File)
    // Song.mp3/ (Directory)
    fs::write(dir_path.join("song.mp3"), "c").unwrap();
    fs::create_dir(dir_path.join("Song.mp3")).unwrap();

    let args = Args {
        directory: dir_path.to_path_buf(),
        recursive: false,
    };

    let mut scanner = Scanner::new(args);
    scanner.scan().unwrap();

    let canonical_root = fs::canonicalize(dir_path).unwrap();
    let collisions = &scanner.collisions[&canonical_root];
    let entries = &collisions["song.mp3"];
    
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|(n, t)| n == "song.mp3" && *t == EntryType::File));
    assert!(entries.iter().any(|(n, t)| n == "Song.mp3" && *t == EntryType::Directory));
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
