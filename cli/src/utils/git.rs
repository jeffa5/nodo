use crate::utils::user;
use anyhow::{bail, ensure, Result};
use git2::{ErrorCode, Repository};
use std::path::Path;

pub struct Repo {
    pub repo: Repository,
}

impl Repo {
    pub fn open(root: &Path) -> Result<Self> {
        match Repository::open(root) {
            Ok(repo) => Ok(Self { repo }),
            Err(err) => match err.code() {
                ErrorCode::NotFound => Self::initialise(root),
                _ => bail!(err),
            },
        }
    }

    pub fn initialise(root: &Path) -> Result<Self> {
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
            .map(|s| s.path().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let signature = self.repo.signature()?;
        let parent = head.peel_to_commit()?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Updated {}", changes),
            &tree,
            &[&parent],
        )?;

        Ok(())
    }
}
