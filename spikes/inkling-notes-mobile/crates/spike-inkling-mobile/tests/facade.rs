//! Integration tests calling through the mobile facade, asserting behavior
//! that only the canonical inkling-notes / organism-notes code produces.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use spike_inkling_mobile::{build_vault_navigation, capture_text_note};

/// Unique temp vault per test, removed on drop. std-only on purpose: the
/// spike adds no third-party test deps beyond serde_json.
struct TempVault(PathBuf);

impl TempVault {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "spike-inkling-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temp vault root");
        Self(root)
    }

    fn root_str(&self) -> &str {
        self.0.to_str().expect("utf-8 temp path")
    }

    fn write_note(&self, relative_path: &str, body: &str) {
        let path = self.0.join(relative_path);
        fs::create_dir_all(path.parent().expect("note parent")).expect("create note dir");
        fs::write(path, body).expect("write note");
    }
}

impl Drop for TempVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn capture_normalizes_messy_title_into_vault_filename() {
    let vault = TempVault::new("capture-title");
    let note = capture_text_note(
        vault.root_str(),
        "Field Capture: Pier 7?",
        "Crane idle since 06:00. Two pallets unaccounted for.",
    )
    .expect("capture succeeds");

    // Exact canonical sanitization from organism-notes: a forbidden filename
    // character plus following whitespace collapses to a single "-", and the
    // note lands in Inbox/.
    assert_eq!(note.vault_path, "Inbox/Field Capture-Pier 7-.md");
    assert_eq!(note.title, "Field Capture-Pier 7-");
    assert!(note.body.contains("# Field Capture: Pier 7?"));
    assert!(note.body.contains("Two pallets unaccounted for."));
}

#[test]
fn capture_stamps_vault_freshness_frontmatter() {
    let vault = TempVault::new("capture-freshness");
    let note = capture_text_note(vault.root_str(), "Standup Notes", "Blocked on sync review.")
        .expect("capture succeeds");

    assert!(note.body.starts_with("---\n"));
    assert!(note.body.contains("vault_created_at:"));
    assert!(note.body.contains("vault_touched_at:"));
    let created_at = note.created_at.expect("created_at extracted");
    // RFC 3339 stamp, e.g. 2026-06-12T07:00:00+00:00.
    assert!(created_at.contains('T'), "not RFC 3339: {created_at}");
}

#[test]
fn capture_allocates_unique_paths_for_duplicate_titles() {
    let vault = TempVault::new("capture-unique");
    let first = capture_text_note(vault.root_str(), "Standup Notes", "first")
        .expect("first capture succeeds");
    let second = capture_text_note(vault.root_str(), "Standup Notes", "second")
        .expect("second capture succeeds");

    assert_eq!(first.vault_path, "Inbox/Standup Notes.md");
    // Canonical organism-notes dedup suffix: " 2".
    assert_eq!(second.vault_path, "Inbox/Standup Notes 2.md");
}

#[test]
fn navigation_resolves_wiki_links_and_tags() {
    let vault = TempVault::new("nav-links");
    vault.write_note(
        "Projects/Source.md",
        "---\ntags: [Project, inbox/review]\n---\n\nSee [[Target Note]] for details. Body #Area\n",
    );
    vault.write_note("Projects/Target Note.md", "# Target\n\nPlain body text here.\n");

    let navigation = build_vault_navigation(vault.root_str()).expect("index builds");

    assert_eq!(navigation.note_count, 2);
    assert!(navigation.index_path.ends_with("navigation-index.json"));

    let source = navigation
        .notes
        .iter()
        .find(|note| note.vault_path == "Projects/Source.md")
        .expect("source note indexed");
    // Canonical wiki-link resolution: stem "Target Note" → sibling note path.
    assert_eq!(source.outbound_links, vec!["Projects/Target Note.md"]);
    // Canonical tag normalization: frontmatter list + inline #Area, lowercased.
    for tag in ["project", "inbox/review", "area"] {
        assert!(source.tags.iter().any(|value| value == tag), "missing tag {tag}");
    }

    let target = navigation
        .notes
        .iter()
        .find(|note| note.vault_path == "Projects/Target Note.md")
        .expect("target note indexed");
    assert!(
        target.inbound_links.contains(&"Projects/Source.md".to_string()),
        "backlink missing"
    );

    let project_tag = navigation
        .tags
        .iter()
        .find(|tag| tag.tag == "project")
        .expect("project tag summarized");
    assert_eq!(project_tag.note_count, 1);
}

#[test]
fn navigation_flags_orphan_notes() {
    let vault = TempVault::new("nav-orphan");
    vault.write_note("Resources/Loose End.md", "# Loose End\n\nNo links at all.\n");

    let navigation = build_vault_navigation(vault.root_str()).expect("index builds");

    assert_eq!(navigation.note_count, 1);
    assert_eq!(navigation.orphan_note_count, 1);
    assert!(navigation.notes[0].orphan);
}

#[test]
fn dtos_are_serde_friendly() {
    let vault = TempVault::new("serde");
    let note = capture_text_note(vault.root_str(), "Serde Check", "round trip")
        .expect("capture succeeds");
    let json = serde_json::to_string(&note).expect("serializes");
    let back: spike_inkling_mobile::CapturedNoteDto =
        serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, note);

    let navigation = build_vault_navigation(vault.root_str()).expect("index builds");
    let json = serde_json::to_string(&navigation).expect("serializes");
    let back: spike_inkling_mobile::VaultNavigationDto =
        serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, navigation);
}
