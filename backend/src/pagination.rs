//! Shared paging shape for list endpoints.
//!
//! Every list the UI can search or page through returns this envelope, so the
//! client never has to hold the whole table in memory to filter or count it.

use serde::Serialize;

pub const DEFAULT_PER_PAGE: u64 = 20;
pub const MAX_PER_PAGE: u64 = 200;

/// One-based page number, clamped to something sane.
pub fn page_of(requested: Option<u64>) -> u64 {
    requested.unwrap_or(1).max(1)
}

pub fn per_page_of(requested: Option<u64>) -> u64 {
    requested.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE)
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    /// Total rows matching the filters, not just the ones on this page.
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub pages: u64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, per_page: u64) -> Self {
        Self {
            items,
            total,
            page,
            per_page,
            // An empty result is still one (empty) page, so the UI has something
            // coherent to render.
            pages: total.div_ceil(per_page).max(1),
        }
    }
}

/// Trims a search term, treating whitespace-only as no search at all.
pub fn search_term(raw: Option<String>) -> Option<String> {
    raw.map(|q| q.trim().to_owned()).filter(|q| !q.is_empty())
}
