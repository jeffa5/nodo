use crate::{commands::GlobalOpts, utils::user};
use anyhow::{anyhow, Result};
use clap::Clap;
use git2::{Cred, ErrorCode, Remote, Repository};
use log::{debug, info};
use std::{io, io::Write, path::Path};

#[derive(Clap, Debug)]
pub struct Sync {}

impl Sync {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let repo = match Repository::open(&g.root) {
            Ok(repo) => repo,
            Err(err) => match err.code() {
                ErrorCode::NotFound => init_repo(&g.root)?,
                _ => return Err(err.into()),
            },
        };

        let mut remote = repo.find_remote("origin")?;
        let branch = "master";

        pull(&repo, &mut remote, branch)?;

        push(&mut remote, branch)?;

        Ok(())
    }
}

fn init_repo(root: &Path) -> Result<Repository> {
    if user::confirm("Repo not configured with git, would you like to initialise one?")? {
        let remote_url = user::input("Remote URL to sync with", None)?;
        let repo = Repository::init(root)?;
        repo.remote("origin", &remote_url)?;
        println!("Repo initialised");
        Ok(repo)
    } else {
        Err(anyhow!("Git repo not configured and not initialising one"))
    }
}

fn push(remote: &mut Remote, branch: &str) -> Result<()> {
    let mut cb = git2::RemoteCallbacks::new();
    set_credentials_callback(&mut cb);
    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            println!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            println!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        io::stdout().lock().flush().unwrap();
        true
    });

    let mut opts = git2::PushOptions::new();
    opts.remote_callbacks(cb);
    info!("Pushing changes to {}", remote.name().unwrap());
    remote.push(
        &[format!(
            "refs/heads/{}:refs/remotes/origin/{}",
            branch, branch
        )],
        Some(&mut opts),
    )?;

    Ok(())
}

fn pull(repo: &Repository, remote: &mut Remote, branch: &str) -> Result<()> {
    let fetch_commit = do_fetch(&repo, &[branch], remote)?;
    do_merge(&repo, &branch, fetch_commit)?;
    Ok(())
}

fn set_credentials_callback(cb: &mut git2::RemoteCallbacks) {
    cb.credentials(|_url, username_from_url, _allowed_types| {
        debug!(
            "Fetching credentials for user {}",
            username_from_url.unwrap()
        );
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            &dirs::home_dir().unwrap().join(".ssh/id_rsa"),
            None,
        )
    });
}

fn do_fetch<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {
    let mut cb = git2::RemoteCallbacks::new();

    set_credentials_callback(&mut cb);

    // Print out our transfer progress.
    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            print!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        io::stdout().lock().flush().unwrap();
        true
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb);
    info!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;
    debug!("Fetch complete");

    // If there are local objects (we got a thin pack), then tell the user
    // how many objects we saved from having to cross the network.
    let stats = remote.stats();
    if stats.local_objects() > 0 {
        println!(
            "\rReceived {}/{} objects in {} bytes (used {} local objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        println!(
            "\rReceived {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    println!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        println!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        println!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    } else {
        println!("Nothing to do...");
    }
    Ok(())
}
