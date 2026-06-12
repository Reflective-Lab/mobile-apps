//! Mobile-shaped facade over canonical inkling-notes code.
//!
//! Proves that real studio-app logic crosses into a mobile-facing Rust core
//! unmodified. Two reuse seams:
//!
//! 1. **Capture write path** — `organism_notes::vault::ObsidianVault`, the
//!    same vault engine the inkling-notes capture pipeline writes through.
//!    "Captured text → normalized note draft on the device-canonical vault."
//! 2. **Navigation index** — `notes::navigation::build_navigation_index`,
//!    application logic owned by the inkling-notes app crate itself (tag
//!    extraction, wiki-link resolution, orphan detection).
//!
//! Everything exposed here is a plain function over flat, serde-friendly
//! DTOs — the shape a UniFFI `.udl` could bind to Swift/Kotlin later. Runs
//! fully offline: navigation uses default options (no OCR, no external-link
//! fetching), so nothing touches the network at runtime.

use std::path::Path;

use organism_notes::vault::{ObsidianVault, extract_frontmatter_value, path_to_relative_string};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const CAPTURE_DIRECTORY: &str = "Inbox";

#[derive(Debug, Error)]
pub enum FacadeError {
    #[error("vault error: {0}")]
    Vault(String),
    #[error("navigation index error: {0}")]
    Navigation(String),
}

/// A normalized note draft written to the device-canonical vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedNoteDto {
    /// Vault-relative path, e.g. `Inbox/Field Capture.md`.
    pub vault_path: String,
    /// Title as derived from the sanitized filename stem.
    pub title: String,
    /// Full note body including the vault freshness frontmatter.
    pub body: String,
    /// RFC 3339 `vault_created_at` stamp from the frontmatter.
    pub created_at: Option<String>,
}

/// Capture raw text into the vault: sanitize the title into a filename,
/// allocate a unique path under `Inbox/`, and write a markdown note with
/// vault freshness frontmatter. All normalization is canonical
/// `organism-notes` behavior — nothing is re-implemented here.
pub fn capture_text_note(
    vault_root: &str,
    title: &str,
    captured_text: &str,
) -> Result<CapturedNoteDto, FacadeError> {
    let vault = ObsidianVault::from_root(vault_root);
    vault
        .ensure_root()
        .map_err(|error| FacadeError::Vault(error.to_string()))?;
    let relative_path = vault
        .allocate_note_path(Path::new(CAPTURE_DIRECTORY), title)
        .map_err(|error| FacadeError::Vault(error.to_string()))?;
    let body = format!("# {}\n\n{}\n", title.trim(), captured_text.trim());
    let note = vault
        .save_note(&path_to_relative_string(&relative_path), &body)
        .map_err(|error| FacadeError::Vault(error.to_string()))?;
    let created_at = extract_frontmatter_value(&note.body, "vault_created_at");
    Ok(CapturedNoteDto {
        vault_path: note.path,
        title: note.title,
        body: note.body,
        created_at,
    })
}

/// Flat summary of the vault navigation graph for a mobile shell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultNavigationDto {
    pub note_count: usize,
    pub tag_count: usize,
    pub link_count: usize,
    pub backlink_count: usize,
    pub orphan_note_count: usize,
    /// Vault-relative path of the persisted `navigation-index.json`.
    pub index_path: String,
    pub notes: Vec<NavigationNoteDto>,
    pub tags: Vec<TagCountDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavigationNoteDto {
    pub vault_path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub outbound_links: Vec<String>,
    pub inbound_links: Vec<String>,
    pub orphan: bool,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagCountDto {
    pub tag: String,
    pub note_count: usize,
}

/// Build the navigation index over the vault by delegating to the canonical
/// `notes` app crate, then flatten the report into mobile DTOs. Uses default
/// options: no image OCR, no external-link fetching — fully offline.
pub fn build_vault_navigation(vault_root: &str) -> Result<VaultNavigationDto, FacadeError> {
    let vault = ObsidianVault::from_root(vault_root);
    let report =
        notes::navigation::build_navigation_index(&vault).map_err(FacadeError::Navigation)?;
    Ok(VaultNavigationDto {
        note_count: report.note_count,
        tag_count: report.tag_count,
        link_count: report.link_count,
        backlink_count: report.backlink_count,
        orphan_note_count: report.orphan_note_count,
        index_path: report.index_path,
        notes: report
            .notes
            .into_iter()
            .map(|note| NavigationNoteDto {
                vault_path: note.path,
                title: note.title,
                tags: note.tags,
                outbound_links: note.outbound_links,
                inbound_links: note.inbound_links,
                orphan: note.orphan,
                snippet: note.snippet,
            })
            .collect(),
        tags: report
            .tags
            .into_iter()
            .map(|tag| TagCountDto {
                tag: tag.tag,
                note_count: tag.note_count,
            })
            .collect(),
    })
}
