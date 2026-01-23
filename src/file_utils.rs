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

/// 書類タイプのキーワード（これらが最初にある場合、2番目のパーツが会社名）
const DOCUMENT_TYPE_KEYWORDS: &[&str] = &[
    "請求書", "納品書", "見積書", "発注書", "注文書", "領収書",
    "契約書", "報告書", "提案書", "仕様書", "明細書", "通知書",
    "invoice", "estimate", "quotation", "order", "receipt", "report",
];

/// Extract potential company name from filename.
/// Recognizes formats like:
/// - "CompanyName_Document.pdf" -> "CompanyName"
/// - "請求書_CompanyName_Date.pdf" -> "CompanyName" (書類タイプが先頭の場合)
pub fn extract_company_name_from_path(path: &str) -> Option<String> {
    let file_name = std::path::Path::new(path)
        .file_stem()?
        .to_str()?;

    // Split by common delimiters
    let delimiters = ['_', ' ', '(', '（', '【', '['];
    let parts: Vec<&str> = file_name
        .split(&delimiters[..])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return None;
    }

    // 最初のパーツが書類タイプキーワードかチェック
    let first_part_lower = parts[0].to_lowercase();
    let is_first_part_document_type = DOCUMENT_TYPE_KEYWORDS.iter()
        .any(|keyword| first_part_lower == keyword.to_lowercase());

    // 書類タイプが先頭なら2番目のパーツを返す、そうでなければ最初のパーツを返す
    if is_first_part_document_type && parts.len() > 1 {
        Some(parts[1].to_string())
    } else {
        Some(parts[0].to_string())
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
        // 会社名が先頭のパターン
        assert_eq!(extract_company_name_from_path("株式会社サンプル_請求書.pdf"), Some("株式会社サンプル".to_string()));
        assert_eq!(extract_company_name_from_path("テスト商事 報告書.docx"), Some("テスト商事".to_string()));
        assert_eq!(extract_company_name_from_path("Example Corp(2024).pdf"), Some("Example Corp".to_string()));

        // 書類タイプが先頭のパターン（2番目が会社名）
        assert_eq!(extract_company_name_from_path("請求書_日興金属株式会社_20251224.pdf"), Some("日興金属株式会社".to_string()));
        assert_eq!(extract_company_name_from_path("納品書_テスト商事_2024.pdf"), Some("テスト商事".to_string()));
        assert_eq!(extract_company_name_from_path("見積書_ABC株式会社.pdf"), Some("ABC株式会社".to_string()));
    }
}
