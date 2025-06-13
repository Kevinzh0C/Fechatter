//! File Operation Integration Tests
//!
//! Tests file upload, download and storage functionality

use crate::common::TestEnvironment;
use anyhow::Result;
use log::info;
use std::collections::HashSet;

/// File storage path test
#[tokio::test]
async fn test_file_storage_paths() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("file_test").await?;

  // Test file path generation
  let test_content = b"Test file content for path testing";
  let filename = "path_test.txt";

  let file_meta =
    fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, test_content);

  // Verify path format
  let hash_path = file_meta.hash_to_path();
  let url = file_meta.url();

  assert!(hash_path.starts_with(&user.workspace_id.to_string()));
  assert!(url.starts_with("/files/"));
  assert!(url.contains(&user.workspace_id.to_string()));
  assert!(url.ends_with(".txt"));

  info!("✅ File path generation test passed");
  info!("   Hash path: {}", hash_path);
  info!("   URL: {}", url);

  Ok(())
}

/// File content validation test
#[tokio::test]
async fn test_file_content_validation() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("content_test").await?;

  // Test different types of file content
  let test_cases = vec![
    ("text_file.txt", b"Hello, World!".to_vec()),
    ("empty_file.dat", Vec::new()),
    ("binary_file.bin", vec![0xFF, 0xFE, 0xFD, 0xFC]),
    ("large_text.txt", "x".repeat(10000).into_bytes()),
  ];

  for (filename, content) in test_cases {
    let file_meta =
      fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, &content);

    // Verify hash generation
    assert!(!file_meta.hash.is_empty());
    assert!(file_meta.hash.len() >= 32); // SHA-1 hash should be at least 32 characters

    // Verify extension extraction
    let expected_ext = std::path::Path::new(filename)
      .extension()
      .and_then(|s| s.to_str())
      .unwrap_or("")
      .to_string();
    assert_eq!(file_meta.ext, expected_ext);

    info!("✅ File content validation passed for: {}", filename);
  }

  Ok(())
}

/// File hash consistency test
#[tokio::test]
async fn test_file_hash_consistency() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("hash_test").await?;

  let test_content = b"Consistency test content";
  let filename = "consistency_test.txt";

  // Create multiple file objects with same content
  let file1 =
    fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, test_content);

  let file2 =
    fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, test_content);

  let file3 = fechatter_server::models::ChatFile::new(
    user.workspace_id.into(),
    "different_name.txt", // Different filename
    test_content,
  );

  // Same content should produce same hash
  assert_eq!(file1.hash, file2.hash);
  assert_eq!(file1.hash, file3.hash);

  // URL paths should be same (except extension)
  let path1 = file1.hash_to_path();
  let path2 = file2.hash_to_path();
  let path3 = file3.hash_to_path();

  assert_eq!(path1, path2);
  assert_eq!(path1.replace(".txt", ""), path3.replace(".txt", ""));

  info!("✅ File hash consistency test passed");

  Ok(())
}

/// File path security test
#[tokio::test]
async fn test_file_path_security() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("security_test").await?;

  let test_content = b"Security test content";

  // Test potentially malicious filenames
  let malicious_filenames = vec![
    "../../../etc/passwd",
    "..\\..\\windows\\system32\\config\\sam",
    "/etc/shadow",
    "C:\\Windows\\System32\\drivers\\etc\\hosts",
    "file\0name.txt", // Null byte injection
    ".hidden_file",
  ];

  for filename in malicious_filenames {
    let file_meta =
      fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, test_content);

    let url = file_meta.url();
    let hash_path = file_meta.hash_to_path();

    // Add debug info
    info!("Processing filename: {}", filename);
    info!("Generated URL: {}", url);
    info!("Generated hash_path: {}", hash_path);
    info!("Extension: {}", file_meta.ext);
    info!("Hash: {}", file_meta.hash);
    info!("Workspace ID: {}", file_meta.workspace_id);

    // Verify generated paths don't contain directory traversal
    assert!(!url.contains(".."));
    assert!(!url.contains("\\"), "URL contains backslash: {}", url);
    assert!(!hash_path.contains(".."));
    assert!(
      !hash_path.contains("\\"),
      "Hash path contains backslash: {}",
      hash_path
    );

    // Verify paths always stay within workspace
    assert!(hash_path.starts_with(&user.workspace_id.to_string()));
    assert!(url.starts_with(&format!("/files/{}", user.workspace_id)));

    info!("✅ Secured malicious filename: {}", filename);
  }

  info!("✅ File path security test passed");

  Ok(())
}

/// File storage directory structure test
#[tokio::test]
async fn test_file_storage_structure() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("structure_test").await?;

  // Create multiple files to test directory structure
  let test_files = vec![
    ("file1.txt", b"Content 1".to_vec()),
    ("file2.jpg", b"Content 2".to_vec()),
    ("file3.pdf", b"Content 3".to_vec()),
    ("file4.doc", b"Content 4".to_vec()),
    ("file5.png", b"Content 5".to_vec()),
  ];

  let mut created_paths = HashSet::new();
  let mut hash_prefixes = HashSet::new();

  for (filename, content) in test_files {
    let file_meta =
      fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, &content);

    let hash_path = file_meta.hash_to_path();
    let url = file_meta.url();

    // Record created paths
    created_paths.insert(hash_path.clone());

    // Extract hash prefix (for directory sharding)
    let hash_prefix = &file_meta.hash[0..3];
    hash_prefixes.insert(hash_prefix.to_string());

    // Verify path structure
    let parts: Vec<&str> = hash_path.split('/').collect();
    assert!(parts.len() >= 3); // Should contain at least workspace_id/xxx/yyy/filename

    // Verify workspace ID
    assert_eq!(parts[0], user.workspace_id.to_string());

    info!("✅ Created file structure: {}", hash_path);
    info!("   URL: {}", url);
    info!("   Hash prefix: {}", hash_prefix);
  }

  // Verify all paths are unique
  assert_eq!(created_paths.len(), 5);

  info!("✅ File storage structure test passed");
  info!("   Unique paths created: {}", created_paths.len());
  info!("   Hash prefixes used: {}", hash_prefixes.len());

  Ok(())
}

/// Large file handling test
#[tokio::test]
async fn test_large_file_handling() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("large_file_test").await?;

  // Create files of different sizes
  let test_sizes = vec![
    ("small.txt", 1024),        // 1KB
    ("medium.txt", 1024 * 100), // 100KB
    ("large.txt", 1024 * 1024), // 1MB
  ];

  for (filename, size) in test_sizes {
    // Generate content of specified size
    let content: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    let start_time = std::time::Instant::now();

    let file_meta =
      fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, &content);

    let creation_time = start_time.elapsed();

    // Verify hash generation for large files
    assert!(!file_meta.hash.is_empty());

    info!(
      "✅ Large file test passed: {} ({} bytes) in {:?}",
      filename, size, creation_time
    );

    // Performance check: even 1MB files should complete hash generation in reasonable time
    if size >= 1024 * 1024 {
      assert!(creation_time.as_millis() < 1000); // Should complete within 1 second
    }
  }

  Ok(())
}

/// Concurrent file operations test
#[tokio::test]
async fn test_concurrent_file_operations() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("concurrent_file_test").await?;

  // Concurrently create multiple files
  let mut handles = Vec::new();

  for i in 0..10 {
    let user_workspace_id = user.workspace_id;
    let handle = tokio::spawn(async move {
      let content = format!("Concurrent file content {}", i).into_bytes();
      let filename = format!("concurrent_file_{}.txt", i);

      fechatter_server::models::ChatFile::new(user_workspace_id.into(), &filename, &content)
    });

    handles.push(handle);
  }

  // Wait for all file creations to complete
  let mut created_files = Vec::new();
  for handle in handles {
    let file_meta = handle.await?;
    created_files.push(file_meta);
  }

  // Verify all files were created correctly
  assert_eq!(created_files.len(), 10);

  // Verify all files have unique hashes
  let mut hashes = std::collections::HashSet::new();
  for file_meta in &created_files {
    assert!(hashes.insert(file_meta.hash.clone()));
  }

  assert_eq!(hashes.len(), 10);

  info!("✅ Concurrent file operations test passed");
  info!(
    "   Created {} unique files concurrently",
    created_files.len()
  );

  Ok(())
}
