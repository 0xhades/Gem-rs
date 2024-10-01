//! Utility functions for the Gem-rs library.
//!
//! This module contains various utility functions used internally by the Gem-rs library,
//! primarily for handling MIME types of different file formats.

use std::path::Path;

/// Determines the MIME type of a file based on its extension.
///
/// This function takes a file path and attempts to determine its MIME type
/// by examining the file extension. It supports various file formats including
/// documents, images, audio, and video files.
///
/// # Arguments
///
/// * `file_path` - A reference to a `Path` representing the file path.
///
/// # Returns
///
/// An `Option<String>` containing the MIME type if recognized, or `None` if the
/// file type is not supported or recognized.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use gem_rs::utils::get_mime_type;
///
/// let path = Path::new("example.pdf");
/// assert_eq!(get_mime_type(path), Some("application/pdf".to_string()));
///
/// let path = Path::new("image.png");
/// assert_eq!(get_mime_type(path), Some("image/png".to_string()));
///
/// let path = Path::new("unknown.xyz");
/// assert_eq!(get_mime_type(path), None);
/// ```
///
/// # Notes
///
/// - Currently, all document types are treated as PDF. This may change in future updates.
/// - GIF files are currently not supported, as there's a TODO to implement GIF to video conversion.
/// - All text types (including code files) are currently treated as plain text.
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
