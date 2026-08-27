//! Authorship gate for directory ingestion — the Architect's security
//! check: reject any file whose last git commit author is not a known
//! BahyWay.Ecosystem team member.
//!
//! Built on git history deliberately, not a new invented identity scheme:
//! `kupru` has real Ed25519 signing/verification primitives
//! (`SovereignVerifier::verify`), but no document format in this repo
//! attaches a seal to a plain Markdown file today, and `DocumentStructure`
//! carries no author field. Git's own commit author is the one piece of
//! real, already-present provenance every tracked file has. A future
//! iteration could upgrade this to `kupru`-verified signatures once
//! documents actually carry seals — this is the honest v1, not the final
//! form, and it says so rather than pretending to be more than it is.
//!
//! Fails closed: a file with no determinable author (untracked, no git
//! history, git unavailable) is treated as unauthorized, never admitted
//! by default.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// A set of git author emails recognized as BahyWay.Ecosystem team
/// members, compared case-insensitively.
pub struct TeamAllowlist {
    emails: HashSet<String>,
}

impl TeamAllowlist {
    /// The Architect's own real git commit identities, found by
    /// inspecting this repository's own `git log` history directly
    /// (`bfadam@bahyway.com`, `bfadam@bahyway.example`, and the GitHub
    /// noreply address behind the `bahyway` account) — verified ground
    /// truth, not guessed. `bahaa.fadam@gmail.com` is included too, as
    /// the Architect's known real address even though no commit in this
    /// repo's history uses it yet (a future machine/config might).
    ///
    /// Deliberately excludes `noreply@anthropic.com` (this agent's own
    /// commit identity): the entire point of a human-authorship gate is
    /// to distinguish reviewed human content from anything else,
    /// including AI-authored commits — auto-trusting the agent doing the
    /// committing would defeat it. This is a bootstrap default, not a
    /// claim of a complete roster — extend via [`TeamAllowlist::from_file`]
    /// as real team members are added.
    pub fn seed() -> Self {
        let mut emails = HashSet::new();
        emails.insert("bahaa.fadam@gmail.com".to_string());
        emails.insert("bfadam@bahyway.com".to_string());
        emails.insert("bfadam@bahyway.example".to_string());
        emails.insert("122043936+bahyway@users.noreply.github.com".to_string());
        TeamAllowlist { emails }
    }

    /// One email per line; blank lines and lines starting with `#`
    /// ignored. Case-insensitive.
    pub fn from_file(path: &Path) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        let emails = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.to_lowercase())
            .collect();
        Ok(TeamAllowlist { emails })
    }

    /// Load from the file named by `ENKIDDB_TEAM_ALLOWLIST` if that
    /// variable is set and the file is readable; otherwise fall back to
    /// [`TeamAllowlist::seed`].
    pub fn from_env_or_seed() -> Self {
        std::env::var("ENKIDDB_TEAM_ALLOWLIST")
            .ok()
            .and_then(|p| TeamAllowlist::from_file(Path::new(&p)).ok())
            .unwrap_or_else(TeamAllowlist::seed)
    }

    pub fn contains(&self, email: &str) -> bool {
        self.emails.contains(&email.to_lowercase())
    }

    pub fn len(&self) -> usize {
        self.emails.len()
    }

    pub fn is_empty(&self) -> bool {
        self.emails.is_empty()
    }
}

/// The result of checking one file's authorship: whether it is
/// authorized, and the git author email found (if any) — surfaced even
/// on rejection so the operator can see exactly who was denied.
#[derive(Debug, Clone)]
pub struct AuthorshipCheck {
    pub authorized: bool,
    pub author: Option<String>,
}

/// Run `git log -1 --format=%ae -- <file>` from the file's own
/// directory. Returns `None` if git is unavailable, the file isn't
/// tracked, or it has no commits yet — any of which fail the check
/// closed in [`check_authorship`].
pub fn git_author_email(file: &Path) -> Option<String> {
    let dir = file.parent()?;
    let name = file.file_name()?.to_str()?;
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .arg("log")
        .arg("-1")
        .arg("--format=%ae")
        .arg("--")
        .arg(name)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

/// Check one file's authorship against `allowlist`. An undeterminable
/// author is never authorized.
pub fn check_authorship(file: &Path, allowlist: &TeamAllowlist) -> AuthorshipCheck {
    let author = git_author_email(file);
    let authorized = author.as_deref().is_some_and(|e| allowlist.contains(e));
    AuthorshipCheck { authorized, author }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command as Cmd;

    fn run(dir: &Path, args: &[&str]) {
        let status = Cmd::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    fn init_repo_with_commit(dir: &Path, file_name: &str, author_email: &str) {
        fs::create_dir_all(dir).unwrap();
        run(dir, &["init", "-q"]);
        run(dir, &["config", "user.email", author_email]);
        run(dir, &["config", "user.name", "Test Author"]);
        fs::write(dir.join(file_name), "# Doc\n\nBody.\n").unwrap();
        run(dir, &["add", file_name]);
        run(dir, &["commit", "-q", "-m", "add doc"]);
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_authorship_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn seed_allowlist_contains_the_architect() {
        let allow = TeamAllowlist::seed();
        // Every real git author identity found in this repo's own
        // history, plus the Architect's known address.
        assert!(allow.contains("bahaa.fadam@gmail.com"));
        assert!(allow.contains("bfadam@bahyway.com"));
        assert!(allow.contains("BFADAM@BAHYWAY.EXAMPLE"));
        assert!(allow.contains("122043936+bahyway@users.noreply.github.com"));
        // The agent's own commit identity is deliberately not trusted by
        // default -- see the doc comment on `seed()`.
        assert!(!allow.contains("noreply@anthropic.com"));
        assert!(!allow.contains("someone-else@example.com"));
    }

    #[test]
    fn from_file_parses_emails_skipping_comments_and_blanks() {
        let dir = scratch_dir("allowlist_file");
        fs::create_dir_all(&dir).unwrap();
        let list_path = dir.join("team.txt");
        fs::write(
            &list_path,
            "# BahyWay team\n\nAlice@Example.com\nbob@example.com\n",
        )
        .unwrap();

        let allow = TeamAllowlist::from_file(&list_path).unwrap();
        assert_eq!(allow.len(), 2);
        assert!(allow.contains("alice@example.com"));
        assert!(allow.contains("bob@example.com"));
    }

    #[test]
    fn git_author_email_reads_real_commit_author() {
        let dir = scratch_dir("git_author");
        init_repo_with_commit(&dir, "doc.md", "author@example.com");

        let email = git_author_email(&dir.join("doc.md"));
        assert_eq!(email.as_deref(), Some("author@example.com"));
    }

    #[test]
    fn git_author_email_is_none_for_untracked_file() {
        let dir = scratch_dir("untracked");
        fs::create_dir_all(&dir).unwrap();
        run(&dir, &["init", "-q"]);
        fs::write(dir.join("untracked.md"), "# X\n").unwrap();

        assert_eq!(git_author_email(&dir.join("untracked.md")), None);
    }

    #[test]
    fn check_authorship_authorizes_known_team_member() {
        let dir = scratch_dir("authorized");
        init_repo_with_commit(&dir, "doc.md", "bahaa.fadam@gmail.com");

        let allow = TeamAllowlist::seed();
        let result = check_authorship(&dir.join("doc.md"), &allow);
        assert!(result.authorized);
        assert_eq!(result.author.as_deref(), Some("bahaa.fadam@gmail.com"));
    }

    #[test]
    fn check_authorship_rejects_unknown_author() {
        let dir = scratch_dir("rejected");
        init_repo_with_commit(&dir, "doc.md", "outsider@example.com");

        let allow = TeamAllowlist::seed();
        let result = check_authorship(&dir.join("doc.md"), &allow);
        assert!(!result.authorized);
        assert_eq!(result.author.as_deref(), Some("outsider@example.com"));
    }

    #[test]
    fn check_authorship_rejects_undeterminable_author() {
        let dir = scratch_dir("no_git");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("orphan.md"), "# X\n").unwrap();

        let allow = TeamAllowlist::seed();
        let result = check_authorship(&dir.join("orphan.md"), &allow);
        assert!(!result.authorized);
        assert_eq!(result.author, None);
    }
}
