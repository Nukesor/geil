use std::path::PathBuf;

use anyhow::Result;
use git2::*;

pub mod credentials;

pub fn show_stats(path: &PathBuf) -> Result<()> {
    let _repository = Repository::open(path)?;

    Ok(())
}

pub fn update_repo(repository: &Repository, remote: &str, branch: &str) -> Result<()> {
    let mut remote = repository.find_remote(&remote)?;
    let fetch_commit = fetch_all(&repository, &[&branch], &mut remote)?;
    // Do a merge analysis.
    let analysis = repository.merge_analysis(&[&fetch_commit])?;

    // Fast forward, if it's possible.
    if analysis.0.is_fast_forward() {
        // Check if the local branch really exists.
        // This error case shouldn't be possible.
        let refname = format!("refs/heads/{}", branch);
        match repository.find_reference(&refname) {
            Ok(mut refs) => {
                fast_forward(&repository, branch, &mut refs, &fetch_commit)?;
            }
            Err(_) => println!(
                "Cannot find remote branch {} for repository {:?}.",
                branch,
                repository.path()
            ),
        };
    }

    Ok(())
}

/// Fetch all branches and tags of the current repository.
fn fetch_all<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {
    // Use default fetch options.
    // Specify to also download all Tags.
    let mut fetch_options = FetchOptions::new();
    fetch_options.download_tags(AutotagOption::All);

    // Do the acutal fetch.
    remote.fetch(refs, Some(&mut fetch_options), None)?;

    // Get the latest fetch head.
    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

/// Apply a fast forward merge to the current branch.
fn fast_forward(
    repository: &Repository,
    name: &str,
    local_branch: &mut git2::Reference,
    remote_commit: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    // Set the current branch head to the remote branch head
    let msg = format!(
        "Fast-Forward: Setting {} to id: {}",
        name,
        remote_commit.id()
    );
    local_branch.set_target(remote_commit.id(), &msg)?;

    // Set the repository head to the new branch head.
    repository.set_head(&name)?;
    repository.checkout_head(Some(git2::build::CheckoutBuilder::default().safe()))?;
    Ok(())
}
