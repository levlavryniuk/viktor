use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{fs, task};

pub struct Crawler {
    root_path: PathBuf,
    matcher: SkimMatcherV2,
}

impl Crawler {
    /// Creates a new `Crawler`. Always succeeds:
    /// if `canonicalize` fails, we just keep the raw path.
    pub async fn new<P: AsRef<Path>>(root_path: P) -> Self {
        let raw = root_path.as_ref().to_path_buf();
        let canonical = raw.clone().canonicalize().unwrap_or(raw.clone());
        Crawler {
            root_path: canonical,
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Fuzzy searches under `root_path`. On any walker error,
    /// it just skips entries, so this always returns a Vec.
    pub fn fuzzy_search_paths(&self, queries: &[&str]) -> Vec<(i64, PathBuf)> {
        let mut best: HashMap<PathBuf, i64> = HashMap::new();
        let walker = WalkBuilder::new(&self.root_path)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .hidden(true)
            .build();

        for res in walker {
            let entry = match res {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry
                .file_type()
                .map_or(false, |ft| ft.is_file() || ft.is_dir())
            {
                continue;
            }
            let path = entry.path();
            let rel = match path
                .strip_prefix(&self.root_path)
                .ok()
                .and_then(|p| p.to_str())
            {
                Some(s) => s,
                None => continue,
            };

            let mut best_score = 0;
            for &q in queries {
                if let Some(score) = self.matcher.fuzzy_match(rel, q) {
                    best_score = best_score.max(score);
                }
            }
            if best_score > 0 {
                best.entry(path.to_path_buf())
                    .and_modify(|e| *e = (*e).max(best_score))
                    .or_insert(best_score);
            }
        }

        let mut out: Vec<_> = best.into_iter().map(|(p, s)| (s, p)).collect();
        out.sort_by(|a, b| b.0.cmp(&a.0));
        out
    }

    /// Reads a file’s contents. Any failure => empty String.
    pub async fn read_file_contents<P: AsRef<Path>>(&self, rel: P) -> String {
        let full = self.root_path.join(rel.as_ref());
        let canon = full.clone().canonicalize().unwrap_or(full.clone());
        if !canon.starts_with(&self.root_path) || !full.is_file() {
            return String::new();
        }
        fs::read_to_string(&full).await.unwrap_or_default()
    }

    /// Lists children of `rel`, recursing up to `depth` levels.
    pub async fn list_directory_contents<P: AsRef<Path>>(
        &self,
        rel: P,
        depth: usize,
    ) -> Vec<PathBuf> {
        // 1) Resolve and check boundaries
        let full = self.root_path.join(rel.as_ref());
        let canon = full.clone().canonicalize().unwrap_or(full.clone());
        if !canon.starts_with(&self.root_path) || !full.is_dir() {
            return Vec::new();
        }

        // 2) Spawn a blocking task for the WalkBuilder
        let root = full.clone();
        let max_depth = depth + 1; // 0 => only direct children
        task::spawn_blocking(move || {
            let mut out = Vec::new();
            let walker = WalkBuilder::new(&root)
                .git_ignore(true)
                .git_exclude(true)
                .git_global(true)
                .hidden(true)
                .max_depth(Some(max_depth))
                .build();

            for res in walker.flatten() {
                let entry = res;
                let path = entry.path();
                // skip the root itself
                if path != root {
                    out.push(path.to_path_buf());
                }
            }
            dbg!(&out);
            out
        })
        .await
        .unwrap_or_default()
    }

    /// Expose the (canonical) root path.
    pub fn root_path(&self) -> &std::path::Path {
        &self.root_path
    }
}
