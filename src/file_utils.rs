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
