//! Resolution of file links to filesystem paths.
//!
//! A link is relative to the directory of the document containing it, except
//! that a link beginning with `/` is relative to the working directory.

use std::path::{Path, PathBuf};

/// Returns the path that `target`, linked from the document at `source`,
/// refers to.
#[must_use]
pub fn resolve(target: &str, source: Option<&Path>) -> PathBuf {
    if let Some(rooted) = target.strip_prefix('/') {
        return PathBuf::from(rooted);
    }
    match source.and_then(Path::parent) {
        Some(directory) => directory.join(target),
        None => PathBuf::from(target),
    }
}

/// Returns the path that `target` refers to if it names an existing file,
/// so that links which are not files (such as URLs) can be told apart.
#[must_use]
pub fn existing_file(target: &str, source: Option<&Path>) -> Option<PathBuf> {
    Some(resolve(target, source)).filter(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_to_source_directory() {
        let source = Path::new("docs/guide/index.md");
        assert_eq!(
            resolve("other.md", Some(source)),
            PathBuf::from("docs/guide/other.md")
        );
        assert_eq!(
            resolve("../top.md", Some(source)),
            PathBuf::from("docs/guide/../top.md")
        );
    }

    #[test]
    fn source_in_working_directory() {
        assert_eq!(
            resolve("other.md", Some(Path::new("index.md"))),
            PathBuf::from("other.md")
        );
    }

    #[test]
    fn rooted_at_working_directory() {
        assert_eq!(
            resolve("/notes/a.md", Some(Path::new("docs/index.md"))),
            PathBuf::from("notes/a.md")
        );
    }

    #[test]
    fn without_source() {
        assert_eq!(resolve("other.md", None), PathBuf::from("other.md"));
    }

    #[test]
    fn existing_file_rejects_urls() {
        assert_eq!(existing_file("https://example.com/a.html", None), None);
    }
}
