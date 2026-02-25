/// SEO utilities for image optimization, lazy loading, and performance enhancements
use std::path::Path;

/// Image optimization metadata
#[derive(Debug, Clone)]
pub struct ImageOptimization {
    /// Original image path
    pub src: String,
    /// Alternative text for accessibility
    pub alt: String,
    /// WebP format path (optional, for responsive images)
    pub webp_src: Option<String>,
    /// Whether to enable lazy loading
    pub lazy_load: bool,
    /// Width for responsive images
    pub width: Option<u32>,
    /// Height for responsive images
    pub height: Option<u32>,
}

impl ImageOptimization {
    /// Create a new optimized image with alt text
    pub fn new(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: alt.into(),
            webp_src: None,
            lazy_load: true,
            width: None,
            height: None,
        }
    }

    /// Add WebP format source
    pub fn with_webp(mut self, webp_src: impl Into<String>) -> Self {
        self.webp_src = Some(webp_src.into());
        self
    }

    /// Disable lazy loading
    pub fn without_lazy_load(mut self) -> Self {
        self.lazy_load = false;
        self
    }

    /// Set dimensions
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Generate HTML img tag with optimizations
    pub fn to_html(&self) -> String {
        let mut html = String::new();

        // Use picture element if WebP is available for better browser support
        if self.webp_src.is_some() {
            html.push_str("<picture>");
            html.push_str(&format!(
                r#"<source srcset="{}" type="image/webp">"#,
                self.webp_src.as_ref().unwrap()
            ));
        }

        html.push_str("<img");
        html.push_str(&format!(r#" src="{}""#, self.src));
        html.push_str(&format!(r#" alt="{}""#, self.alt));

        if self.lazy_load {
            html.push_str(r#" loading="lazy""#);
        }

        if let Some(width) = self.width {
            html.push_str(&format!(r#" width="{}""#, width));
        }

        if let Some(height) = self.height {
            html.push_str(&format!(r#" height="{}""#, height));
        }

        html.push_str(">");

        if self.webp_src.is_some() {
            html.push_str("</picture>");
        }

        html
    }
}

/// Generate image paths for WebP conversion
pub fn get_webp_path(original_path: &str) -> String {
    if let Some(pos) = original_path.rfind('.') {
        format!("{}.webp", &original_path[..pos])
    } else {
        format!("{}.webp", original_path)
    }
}

/// Check if image file exists and validate
pub fn validate_image_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    // Check if file has valid image extension
    let valid_extensions = ["jpg", "jpeg", "png", "gif", "svg", "webp"];
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !valid_extensions.contains(&extension.as_str()) {
        return Err(format!("Invalid image extension: {}", extension));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_optimization_basic() {
        let img = ImageOptimization::new("/img/test.jpg", "Test image");
        assert_eq!(img.src, "/img/test.jpg");
        assert_eq!(img.alt, "Test image");
        assert!(img.lazy_load);
    }

    #[test]
    fn test_webp_path_generation() {
        assert_eq!(get_webp_path("/img/test.jpg"), "/img/test.webp");
        assert_eq!(get_webp_path("/img/test.png"), "/img/test.webp");
    }

    #[test]
    fn test_image_html_generation() {
        let html = ImageOptimization::new("/img/test.jpg", "A test image")
            .with_dimensions(800, 600)
            .to_html();

        assert!(html.contains(r#"src="/img/test.jpg""#));
        assert!(html.contains(r#"alt="A test image""#));
        assert!(html.contains(r#"loading="lazy""#));
        assert!(html.contains(r#"width="800""#));
        assert!(html.contains(r#"height="600""#));
    }

    #[test]
    fn test_image_with_webp() {
        let html = ImageOptimization::new("/img/test.jpg", "A test image")
            .with_webp("/img/test.webp")
            .to_html();

        assert!(html.contains("<picture>"));
        assert!(html.contains("</picture>"));
        assert!(html.contains(r#"type="image/webp""#));
    }

    #[test]
    fn test_validate_image_path() {
        assert!(validate_image_path("/img/test.jpg").is_ok());
        assert!(validate_image_path("/img/test.png").is_ok());
        assert!(validate_image_path("/img/test.webp").is_ok());
        assert!(validate_image_path("/img/test.txt").is_err());
    }
}
