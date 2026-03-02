use crate::error::{DuplffError, Result};
use crate::models::DuplicateGroup;
use std::path::PathBuf;

/// Plan produced by a dry run -- describes what would be deleted.
#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub files_to_delete: Vec<PathBuf>,
    pub bytes_to_reclaim: u64,
}

/// Record of a single action taken.
#[derive(Debug, Clone)]
pub struct ActionRecord {
    pub path: PathBuf,
    pub action: ActionType,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Trashed,
}

/// Log of all actions taken during a trash operation.
#[derive(Debug, Clone)]
pub struct ActionLog {
    pub actions: Vec<ActionRecord>,
    pub bytes_reclaimed: u64,
}

/// Produce a dry-run plan: which files would be deleted and how much space reclaimed.
pub fn dry_run(groups: &[DuplicateGroup]) -> ActionPlan {
    let mut files_to_delete = Vec::new();
    let mut bytes_to_reclaim = 0u64;

    for group in groups {
        for dup in &group.duplicates {
            files_to_delete.push(dup.entry.path.clone());
            bytes_to_reclaim += group.size;
        }
    }

    ActionPlan {
        files_to_delete,
        bytes_to_reclaim,
    }
}

/// Move duplicate files to the OS trash. The keep file is never touched.
pub fn trash_duplicates(groups: &[DuplicateGroup]) -> Result<ActionLog> {
    let mut actions = Vec::new();
    let mut bytes_reclaimed = 0u64;

    for group in groups {
        for dup in &group.duplicates {
            trash::delete(&dup.entry.path).map_err(|e| {
                DuplffError::TrashError(format!("{}: {}", dup.entry.path.display(), e))
            })?;
            actions.push(ActionRecord {
                path: dup.entry.path.clone(),
                action: ActionType::Trashed,
            });
            bytes_reclaimed += group.size;
        }
    }

    Ok(ActionLog {
        actions,
        bytes_reclaimed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use std::time::SystemTime;

    fn make_group(paths: &[&str]) -> DuplicateGroup {
        let keep = RankedFile {
            entry: FileEntry { path: paths[0].into(), size: 100, modified: SystemTime::UNIX_EPOCH },
            reason: KeepReason::LexicographicFirst,
        };
        let duplicates: Vec<RankedFile> = paths[1..]
            .iter()
            .map(|p| RankedFile {
                entry: FileEntry { path: (*p).into(), size: 100, modified: SystemTime::UNIX_EPOCH },
                reason: KeepReason::LexicographicFirst,
            })
            .collect();
        DuplicateGroup {
            hash: [0u8; 32],
            size: 100,
            keep,
            duplicates,
        }
    }

    #[test]
    fn dry_run_lists_files_to_delete() {
        let groups = vec![make_group(&["/keep.txt", "/delete1.txt", "/delete2.txt"])];
        let plan = dry_run(&groups);
        assert_eq!(plan.files_to_delete.len(), 2);
        assert_eq!(plan.bytes_to_reclaim, 200);
        assert!(plan.files_to_delete.contains(&std::path::PathBuf::from("/delete1.txt")));
    }

    #[test]
    fn dry_run_never_includes_keep_file() {
        let groups = vec![make_group(&["/keep.txt", "/dup.txt"])];
        let plan = dry_run(&groups);
        assert!(!plan.files_to_delete.contains(&std::path::PathBuf::from("/keep.txt")));
    }

    #[test]
    fn trash_duplicates_moves_files() {
        use std::fs;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let keep = dir.path().join("keep.txt");
        let dup = dir.path().join("dup.txt");
        fs::write(&keep, "content").unwrap();
        fs::write(&dup, "content").unwrap();

        let groups = vec![DuplicateGroup {
            hash: [0u8; 32],
            size: 7,
            keep: RankedFile {
                entry: FileEntry { path: keep.clone(), size: 7, modified: SystemTime::UNIX_EPOCH },
                reason: KeepReason::LexicographicFirst,
            },
            duplicates: vec![RankedFile {
                entry: FileEntry { path: dup.clone(), size: 7, modified: SystemTime::UNIX_EPOCH },
                reason: KeepReason::LexicographicFirst,
            }],
        }];

        let log = trash_duplicates(&groups).unwrap();
        assert!(keep.exists(), "keep file should still exist");
        assert!(!dup.exists(), "duplicate should be trashed");
        assert_eq!(log.actions.len(), 1);
    }
}
