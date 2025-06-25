// Simple extractors for analytics server

use crate::pb::GeoLocation;
use axum::http::request::Parts;

pub struct Geo(pub Option<GeoLocation>);

impl Geo {
  pub fn from_parts(parts: &Parts) -> Self {
    let geo = Self::extract_from_headers(&parts.headers);
    Self(geo)
  }

  /// Extract geographic information from HTTP headers
  /// Supports common proxy headers like those from CloudFlare, AWS, etc.
  fn extract_from_headers(headers: &axum::http::HeaderMap) -> Option<GeoLocation> {
    // Try multiple header combinations for robustness
    let country = Self::get_header_value(
      headers,
      &[
        "cf-ipcountry",              // CloudFlare
        "x-country",                 // Custom
        "cloudfront-viewer-country", // AWS CloudFront
      ],
    );

    let region = Self::get_header_value(
      headers,
      &[
        "cf-region",                        // CloudFlare
        "x-region",                         // Custom
        "cloudfront-viewer-country-region", // AWS CloudFront
      ],
    );

    let city = Self::get_header_value(
      headers,
      &[
        "cf-ipcity", // CloudFlare
        "x-city",    // Custom
      ],
    );

    match (country, region, city) {
      (Some(country), region, city) => Some(GeoLocation {
        country,
        region: region.unwrap_or_default(),
        city: city.unwrap_or_default(),
      }),
      _ => None,
    }
  }

  fn get_header_value(headers: &axum::http::HeaderMap, header_names: &[&str]) -> Option<String> {
    for name in header_names {
      if let Some(value) = headers.get(*name) {
        if let Ok(value_str) = value.to_str() {
          if !value_str.is_empty() {
            return Some(value_str.to_string());
          }
        }
      }
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::HeaderMap;

  #[test]
  fn test_geo_extraction_cloudflare() {
    let mut headers = HeaderMap::new();
    headers.insert("cf-ipcountry", "US".parse().unwrap());
    headers.insert("cf-region", "CA".parse().unwrap());
    headers.insert("cf-ipcity", "San Francisco".parse().unwrap());

    let geo = Geo::extract_from_headers(&headers);
    assert!(geo.is_some());

    let geo = geo.unwrap();
    assert_eq!(geo.country, "US");
    assert_eq!(geo.region, "CA");
    assert_eq!(geo.city, "San Francisco");
  }

  #[test]
  fn test_geo_extraction_custom() {
    let mut headers = HeaderMap::new();
    headers.insert("x-country", "CN".parse().unwrap());
    headers.insert("x-region", "Beijing".parse().unwrap());
    headers.insert("x-city", "Beijing".parse().unwrap());

    let geo = Geo::extract_from_headers(&headers);
    assert!(geo.is_some());

    let geo = geo.unwrap();
    assert_eq!(geo.country, "CN");
    assert_eq!(geo.region, "Beijing");
    assert_eq!(geo.city, "Beijing");
  }

  #[test]
  fn test_geo_extraction_fallback() {
    let mut headers = HeaderMap::new();
    headers.insert("cf-ipcountry", "JP".parse().unwrap());
    // Only country available

    let geo = Geo::extract_from_headers(&headers);
    assert!(geo.is_some());

    let geo = geo.unwrap();
    assert_eq!(geo.country, "JP");
    assert_eq!(geo.region, "");
    assert_eq!(geo.city, "");
  }

  #[test]
  fn test_geo_extraction_none() {
    let headers = HeaderMap::new();
    let geo = Geo::extract_from_headers(&headers);
    assert!(geo.is_none());
  }
}
