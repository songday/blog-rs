use std::path::Path;
use std::vec::Vec;

use blog_common::dto::git::GitRepositoryInfo;
use git2::{
    Commit, Direction, Error as GitError, ObjectType, Oid, Repository, RepositoryState, Signature, StatusOptions,
    StatusShow,
};

fn sync_to_remote(info: &GitRepositoryInfo) -> Result<(), ()> {
    // export posts data to file system
    // todo
    // open git repository
    let repo = match Repository::open(&info.path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    // perform committing
    // todo
    // try pushing
    // todo
    Ok(())
}

fn get_signature(repo: &Repository) -> Result<Signature, GitError> {
    let config = repo.config()?;
    let name = config.get_str("user.name")?;
    let email = config.get_str("user.email")?;
    Ok(Signature::now(name, email)?)
}

fn get_changed_files(repo: &Repository) -> Result<Vec<String>, GitError> {
    let state = repo.state();
    if state.eq(&RepositoryState::Clean) {
        return Ok(vec![]);
    }
    let mut status_options = StatusOptions::new();
    let status = repo.statuses(Some(status_options.show(StatusShow::Index)))?;
    for f in status.iter() {}
    Ok(vec![])
}

fn find_last_commit(repo: &Repository) -> Result<Commit, GitError> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| GitError::from_str("Couldn't find commit"))
}

fn add_and_commit(
    info: &GitRepositoryInfo,
    repo: &Repository,
    files: Vec<&Path>,
    message: &str,
) -> Result<Oid, GitError> {
    let signature = get_signature(repo)?;
    let mut index = repo.index()?;
    for file in files.iter() {
        index.add_path(file)?;
    }
    let oid = index.write_tree()?;
    let parent_commit = find_last_commit(&repo)?;
    let tree = repo.find_tree(oid)?;
    repo.commit(
        Some("HEAD"), //  point HEAD to our new commit
        &signature,   // author
        &signature,   // committer
        message,      // commit message
        &tree,        // tree
        &[&parent_commit],
    ) // parents
}

fn push(repo: &Repository, url: &str) -> Result<(), GitError> {
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => repo.remote("origin", url)?,
    };
    remote.connect(Direction::Push)?;
    remote.push(&["refs/heads/master:refs/heads/master"], None)
}
