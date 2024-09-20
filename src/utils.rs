use std::path::Path;

pub(crate) fn get_mime_type(file_path: &Path) -> Option<String> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        //TODO: Convert all document types to PDF
        Some("pdf") => Some("application/pdf".to_string()),

        Some("png") => Some("image/png".to_string()),
        Some("jpg") | Some("jpeg") => Some("image/jpeg".to_string()),

        Some("mp3") => Some("audio/mpeg".to_string()),
        Some("wav") => Some("audio/wav".to_string()),

        Some("mp4") => Some("video/mp4".to_string()),
        Some("mov") => Some("video/quicktime".to_string()),
        Some("mpeg") => Some("video/mpeg".to_string()),
        Some("mpg") => Some("video/mpeg".to_string()),
        Some("avi") => Some("video/x-msvideo".to_string()),
        Some("wmv") => Some("video/x-ms-wmv".to_string()),
        Some("mpegps") => Some("video/mpeg".to_string()),
        Some("flv") => Some("video/x-flv".to_string()),
        // Some("gif") => Some("image/gif".to_string()), TODO: implement gif to video conversion

        //TODO: Convert all text types (including codes) to plain text
        Some("txt") => Some("text/plain".to_string()),
        _ => None,
    }
}
