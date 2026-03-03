// Ranking logic for duplff-core

use crate::models::{DuplicateGroup, HashedFile, KeepReason, RankedFile};
use std::path::Path;

/// Rank a single group of identical files and produce a DuplicateGroup.
///
/// The file with the highest score is chosen to keep. Scoring criteria (in order):
/// 1. In a user-specified priority directory
/// 2. Deepest path (most path components)
/// 3. Newest modification time
/// 4. Lexicographically first path (tiebreaker)
pub fn rank_group(files: Vec<HashedFile>, priority_paths: &[impl AsRef<Path>]) -> DuplicateGroup {
    assert!(files.len() >= 2, "rank_group requires at least 2 files");

    let hash = files[0].hash;
    let size = files[0].entry.size;

    // Score each file
    type FileScore = (bool, usize, u64, std::cmp::Reverse<std::path::PathBuf>);
    let mut scored: Vec<(HashedFile, FileScore)> = files
        .into_iter()
        .map(|hf| {
            let in_priority = priority_paths
                .iter()
                .any(|p| hf.entry.path.starts_with(p.as_ref()));
            let depth = hf.entry.path.components().count();
            let mtime = hf
                .entry
                .modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // For lexicographic tiebreaker: Reverse so that smaller paths sort higher
            let score = (
                in_priority,
                depth,
                mtime,
                std::cmp::Reverse(hf.entry.path.clone()),
            );
            (hf, score)
        })
        .collect();

    // Sort descending by score (highest = best candidate to keep)
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // Determine the reason: compare winner vs runner-up to find what differentiated them
    let winner_score = &scored[0].1;
    let runner_up_score = &scored[1].1;
    let reason = if winner_score.0 && !runner_up_score.0 {
        KeepReason::PriorityPath
    } else if winner_score.1 != runner_up_score.1 {
        KeepReason::DeepestPath
    } else if winner_score.2 != runner_up_score.2 {
        KeepReason::NewestModification
    } else {
        KeepReason::LexicographicFirst
    };

    let mut iter = scored.into_iter();
    let (winner_hf, _) = iter.next().unwrap();
    let keep = RankedFile {
        entry: winner_hf.entry,
        reason,
    };

    let duplicates: Vec<RankedFile> = iter
        .map(|(hf, _)| RankedFile {
            entry: hf.entry,
            reason: KeepReason::LexicographicFirst,
        })
        .collect();

    DuplicateGroup {
        hash,
        size,
        keep,
        duplicates,
    }
}

/// Rank all duplicate groups.
pub fn rank_groups(
    groups: Vec<Vec<HashedFile>>,
    priority_paths: &[impl AsRef<Path>],
) -> Vec<DuplicateGroup> {
    groups
        .into_iter()
        .map(|g| rank_group(g, priority_paths))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileEntry, HashedFile};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn make_hashed(path: &str, modified: SystemTime) -> HashedFile {
        HashedFile {
            entry: FileEntry {
                path: path.into(),
                size: 100,
                modified,
            },
            hash: [0u8; 32],
        }
    }

    #[test]
    fn priority_path_wins() {
        let group = vec![
            make_hashed("/tmp/copy.txt", UNIX_EPOCH + Duration::from_secs(200)),
            make_hashed("/src/original.txt", UNIX_EPOCH + Duration::from_secs(100)),
        ];
        let priority = vec![std::path::PathBuf::from("/src")];
        let ranked = rank_group(group, &priority);
        assert_eq!(
            ranked.keep.entry.path.to_str().unwrap(),
            "/src/original.txt"
        );
        assert_eq!(ranked.keep.reason, KeepReason::PriorityPath);
    }

    #[test]
    fn deepest_path_wins_when_no_priority() {
        let group = vec![
            make_hashed("/a/file.txt", UNIX_EPOCH),
            make_hashed("/a/b/c/file.txt", UNIX_EPOCH),
        ];
        let ranked = rank_group(group, &Vec::<std::path::PathBuf>::new());
        assert_eq!(ranked.keep.entry.path.to_str().unwrap(), "/a/b/c/file.txt");
        assert_eq!(ranked.keep.reason, KeepReason::DeepestPath);
    }

    #[test]
    fn newest_mtime_wins_when_same_depth() {
        let group = vec![
            make_hashed("/a/old.txt", UNIX_EPOCH + Duration::from_secs(100)),
            make_hashed("/b/new.txt", UNIX_EPOCH + Duration::from_secs(200)),
        ];
        let ranked = rank_group(group, &Vec::<std::path::PathBuf>::new());
        assert_eq!(ranked.keep.entry.path.to_str().unwrap(), "/b/new.txt");
        assert_eq!(ranked.keep.reason, KeepReason::NewestModification);
    }

    #[test]
    fn lexicographic_tiebreaker() {
        let t = UNIX_EPOCH;
        let group = vec![make_hashed("/z/file.txt", t), make_hashed("/a/file.txt", t)];
        let ranked = rank_group(group, &Vec::<std::path::PathBuf>::new());
        assert_eq!(ranked.keep.entry.path.to_str().unwrap(), "/a/file.txt");
        assert_eq!(ranked.keep.reason, KeepReason::LexicographicFirst);
    }

    #[test]
    fn deterministic_across_input_order() {
        let t = UNIX_EPOCH;
        let group1 = vec![make_hashed("/b.txt", t), make_hashed("/a.txt", t)];
        let group2 = vec![make_hashed("/a.txt", t), make_hashed("/b.txt", t)];
        let r1 = rank_group(group1, &Vec::<std::path::PathBuf>::new());
        let r2 = rank_group(group2, &Vec::<std::path::PathBuf>::new());
        assert_eq!(r1.keep.entry.path, r2.keep.entry.path);
    }

    #[test]
    fn rank_groups_produces_duplicate_groups() {
        let groups = vec![vec![
            make_hashed("/a.txt", UNIX_EPOCH),
            make_hashed("/b.txt", UNIX_EPOCH),
        ]];
        let result = rank_groups(groups, &Vec::<std::path::PathBuf>::new());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].duplicates.len(), 1);
    }
}
