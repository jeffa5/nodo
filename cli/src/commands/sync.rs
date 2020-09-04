use crate::{
    commands::GlobalOpts,
    utils::{git, user},
};
use anyhow::{bail, Result};
use colored::Colorize;
use git2::{Cred, Remote, Repository};
use log::{debug, info};
use std::{io, io::Write};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Sync {}

impl Sync {
    #[allow(clippy::unused_self)]
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let mut repo = git::Repo::open(&g.root)?;

        {
            let mut remote = repo.repo.find_remote("origin")?;
            let branch = "master";

            println!("Pulling latest from remote");
            pull(&repo.repo, &mut remote, branch)?;
        }

        let statuses_clean = {
            let statuses = repo.repo.statuses(None)?;
            if !statuses.is_empty() {
                println!("Found the following dirty statuses:");
                for s in statuses.iter() {
                    println!("{} {:?}", s.path().unwrap(), s.status())
                }
            }
            statuses.is_empty()
        };

        if !statuses_clean {
            if user::confirm("Would you like to add and commit all of these before syncing?")? {
                repo.add_all()?.commit()?
            } else {
                bail!("Not syncing a dirty repo")
            }
        }

        let mut remote = repo.repo.find_remote("origin")?;
        let branch = "master";

        println!("Pushing our changes up");
        push(&mut remote, branch)?;

        Ok(())
    }
}

fn push(remote: &mut Remote, branch: &str) -> Result<()> {
    let mut cb = git2::RemoteCallbacks::new();
    set_credentials_callback(&mut cb);

    cb.push_update_reference(|ref_name, status_message| {
        match status_message {
            None => println!("Pushed {}", ref_name),
            Some(s) => println!("Error pushing {}: {}", ref_name, s),
        }
        Ok(())
    });

    let mut opts = git2::PushOptions::new();
    opts.remote_callbacks(cb);
    info!("Pushing changes to {}", remote.name().unwrap());
    remote.push(
        &[format!(
            "refs/heads/{}:refs/remotes/{}/{}",
            branch,
            remote.name().unwrap_or("origin"),
            branch
        )],
        Some(&mut opts),
    )?;

    Ok(())
}

fn pull(repo: &Repository, remote: &mut Remote, branch: &str) -> Result<()> {
    {
        let fetch_commit = do_fetch(repo, &[branch], remote)?;

        do_merge(repo, branch, &fetch_commit)?;
    }

    let head_tree = repo.head()?.peel_to_tree()?;
    let remote_tree = repo
        .find_reference(&format!(
            "refs/remotes/{}/{}",
            remote.name().unwrap(),
            branch
        ))?
        .peel_to_tree()?;
    let diff = repo.diff_tree_to_tree(Some(&remote_tree), Some(&head_tree), None)?;
    let diff_stats = diff.stats()?;
    let files_changed = diff_stats.files_changed();
    if files_changed > 0 {
        println!("{} files changed", files_changed.to_string().bold());
        println!("{}", format!("+ {}", diff_stats.insertions()).green());
        println!("{}", format!("- {}", diff_stats.deletions()).red());
    }
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
    info!("Fetch complete");

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
    fetch_commit: &git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[fetch_commit])?;

    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        println!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        if let Ok(mut r) = repo.find_reference(&refname) {
            fast_forward(repo, &mut r, fetch_commit)?;
        } else {
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
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(repo, &head_commit, fetch_commit)?;
    } else {
        println!("Already have latest from remote");
    }
    Ok(())
}
