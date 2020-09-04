use crate::utils::user;
use anyhow::{bail, ensure, Result};
use git2::{ErrorCode, Repository, Status};
use std::path::Path;

pub struct Repo {
    pub repo: Repository,
}

impl Repo {
    pub fn open(root: &Path) -> Result<Self> {
        match Repository::discover(root) {
            Ok(repo) => {
                if repo.path().starts_with(root) {
                    Ok(Self { repo })
                } else {
                    Self::initialise(root)
                }
            }
            Err(err) => match err.code() {
                ErrorCode::NotFound => Self::initialise(root),
                _ => bail!(err),
            },
        }
    }

    fn initialise(root: &Path) -> Result<Self> {
        ensure!(
            user::confirm("Repo not configured with git, would you like to initialise one?")?,
            "Git repo not configured and not initialising one"
        );
        let remote_url = user::input("Remote URL to sync with", None)?;
        let repo = Repository::init(root)?;
        repo.remote("origin", &remote_url)?;
        println!("Repo initialised");
        Ok(Self { repo })
    }

    pub fn add_path(&mut self, path: &Path) -> Result<&mut Self> {
        let root = self.repo.workdir().unwrap();
        let rel_path = path.strip_prefix(root)?;

        let status = self.repo.status_file(rel_path)?;
        if status.is_empty() {
            return Ok(self);
        }

        let mut index = self.repo.index()?;
        index.add_path(rel_path)?;
        index.write()?;

        Ok(self)
    }

    pub fn remove_path(&mut self, path: &Path) -> Result<&mut Self> {
        let root = self.repo.workdir().unwrap();
        let rel_path = path.strip_prefix(root)?;

        let status = self.repo.status_file(rel_path)?;
        if status.is_empty() {
            return Ok(self);
        }

        let mut index = self.repo.index()?;
        index.remove_path(rel_path)?;
        index.write()?;

        Ok(self)
    }

    pub fn add_all(&mut self) -> Result<&mut Self> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(self)
    }

    pub fn commit(&mut self) -> Result<()> {
        let head = self.repo.head()?;
        let tree_oid = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let statuses = self.repo.statuses(None)?;
        if statuses.is_empty() {
            return Ok(());
        }

        let changes = statuses
            .iter()
            .filter_map(|s| {
                let status = s.status();
                let path = s.path().unwrap().to_string();
                if status.contains(Status::INDEX_NEW) {
                    Some(format!("Add {}", path))
                } else if status.intersects(
                    Status::INDEX_MODIFIED | Status::INDEX_RENAMED | Status::INDEX_TYPECHANGE,
                ) {
                    Some(format!("Update {}", path))
                } else if status.contains(Status::INDEX_DELETED) {
                    Some(format!("Remove {}", path))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let signature = self.repo.signature()?;
        let parent = head.peel_to_commit()?;

        let msg = {
            let items = if changes.len() == 1 { "item" } else { "items" };
            format!(
                "Change {} {}\n\n{}",
                changes.len(),
                items,
                changes.join("\n")
            )
        };

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &msg,
            &tree,
            &[&parent],
        )?;

        Ok(())
    }
}
