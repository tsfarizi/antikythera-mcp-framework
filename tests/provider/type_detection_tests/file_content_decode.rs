#[test]
fn test_file_content_is_pdf() {
    let file = FileContent {
        metadata: FileMetadata {
            filename: "doc.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            size_bytes: 100,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        },
        data: "test".to_string(),
    };

    assert!(file.is_pdf());
    assert!(!file.is_image());
}

#[test]
fn test_content_item_to_file_content() {
    let item = ContentItem {
        content_type: "resource".to_string(),
        text: Some("Generated file: test.pdf".to_string()),
        data: Some("cGRmZGF0YQ==".to_string()),
        mime_type: Some("application/pdf".to_string()),
        metadata: None,
    };

    let file = item.to_file_content("2024-01-01T00:00:00Z").unwrap();
    assert_eq!(file.metadata.filename, "test.pdf");
    assert_eq!(file.metadata.mime_type, "application/pdf");
}
