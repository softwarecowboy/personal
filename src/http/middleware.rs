/// HTTP middleware and utilities for security headers and performance optimizations
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

/// Middleware to add security and performance headers for SEO compliance
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Strict-Transport-Security (HSTS) - Enforce HTTPS
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains; preload"
            .parse()
            .unwrap(),
    );

    // Content Security Policy - Improve security and SEO
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://cdnjs.cloudflare.com; style-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self'; frame-ancestors 'none';"
            .parse()
            .unwrap(),
    );

    // X-Content-Type-Options - Prevent MIME sniffing
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());

    // X-Frame-Options - Prevent clickjacking
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());

    // X-XSS-Protection - Enable XSS protection
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    // Referrer-Policy - Control referrer information
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());

    // Permissions-Policy - Control browser features
    headers.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=(), payment=()".parse().unwrap(),
    );

    // Cache-Control header for static assets (set per route as needed)
    // Don't override existing Cache-Control

    response
}

/// Extract security headers for verification (useful for testing)
pub fn extract_security_headers(headers: &HeaderMap) -> SecurityHeaders {
    SecurityHeaders {
        hsts: headers
            .get("Strict-Transport-Security")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        csp: headers
            .get("Content-Security-Policy")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        x_content_type_options: headers
            .get("X-Content-Type-Options")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        x_frame_options: headers
            .get("X-Frame-Options")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct SecurityHeaders {
    pub hsts: Option<String>,
    pub csp: Option<String>,
    pub x_content_type_options: Option<String>,
    pub x_frame_options: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_headers_struct() {
        let headers = SecurityHeaders {
            hsts: Some("max-age=31536000".to_string()),
            csp: Some("default-src 'self'".to_string()),
            x_content_type_options: Some("nosniff".to_string()),
            x_frame_options: Some("DENY".to_string()),
        };

        assert!(headers.hsts.is_some());
        assert!(headers.csp.is_some());
        assert_eq!(headers.x_content_type_options, Some("nosniff".to_string()));
    }
}
