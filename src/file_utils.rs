use std::fs::File;
use std::io::Read;
use base64::{Engine as _, engine::general_purpose};

/// Read a file and encode it to Base64
pub fn encode_file_to_base64(file_path: &str) -> Result<String, String> {
    let mut file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    Ok(general_purpose::STANDARD.encode(&buffer))
}

/// Get MIME type from file extension
pub fn get_mime_type(file_name: &str) -> String {
    let extension = file_name.split('.').last().unwrap_or("").to_lowercase();
    
    match extension.as_str() {
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Check if file size is within Gmail's 15MB limit
pub fn check_file_size(file_path: &str) -> Result<u64, String> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let size = metadata.len();
    const MAX_SIZE: u64 = 15 * 1024 * 1024; // 15MB
    
    if size > MAX_SIZE {
        return Err(format!("File size ({:.2} MB) exceeds Gmail's 15MB limit", size as f64 / (1024.0 * 1024.0)));
    }
    
    Ok(size)
}

/// Extract potential company name from filename.
/// Recognizes formats like "CompanyName_Document.pdf", "CompanyName Report.pdf", etc.
pub fn extract_company_name_from_path(path: &str) -> Option<String> {
    let file_name = std::path::Path::new(path)
        .file_stem()?
        .to_str()?;

    // Split by common delimiters and take the first part
    let delimiters = ['_', ' ', '(', '（', '【', '['];
    let first_part = file_name.split(&delimiters[..]).next()?.trim();

    if first_part.is_empty() {
        None
    } else {
        Some(first_part.to_string())
    }
}

/// Extract all parts from filename for template matching.
/// Returns all segments split by common delimiters.
/// Example: "日興金属_納品書_2024.pdf" -> ["日興金属", "納品書", "2024"]
pub fn extract_filename_parts(path: &str) -> Vec<String> {
    let file_name = match std::path::Path::new(path).file_stem().and_then(|s| s.to_str()) {
        Some(name) => name,
        None => return Vec::new(),
    };

    let delimiters = ['_', ' ', '(', '（', '【', '[', ')', '）', '】', ']', '-', '－'];
    file_name
        .split(&delimiters[..])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_company_name() {
        assert_eq!(extract_company_name_from_path("株式会社サンプル_請求書.pdf"), Some("株式会社サンプル".to_string()));
        assert_eq!(extract_company_name_from_path("テスト商事 報告書.docx"), Some("テスト商事".to_string()));
        assert_eq!(extract_company_name_from_path("Example Corp(2024).pdf"), Some("Example Corp".to_string()));
    }
}
