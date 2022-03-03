use std::path::Path;
use std::vec::Vec;

use blog_common::dto::git::GitRepositoryInfo;
use git2::{
    Commit, Direction, Error as GitError, ObjectType, Oid, Repository, RepositoryState, Signature, StatusOptions,
    StatusShow,
};

use crate::db::management;
use crate::db::model::Setting;

pub const SETTING_ITEM_NAME: &'static str = "git-pages";

pub async fn new_repository(info: GitRepositoryInfo) -> Result<(), String> {
    // clone repository
    let path = std::env::current_dir().unwrap();
    let path = path.join(&info.repository_name);
    if path.exists() {
        return Err(format!("Target directory {} already exists", path.as_path().display()));
    }
    if let Err(e) = std::fs::create_dir(path.as_path()) {
        return Err(format!("Failed creating directory: {}", path.as_path().display()));
    }
    if let Err(e) = Repository::clone(&info.remote_url, path.as_path()) {
        return Err(format!("Failed clone git repository: {}", e));
    };
    // save to db
    let r = serde_json::to_string(&info);
    let setting = Setting {
        item: String::from(SETTING_ITEM_NAME),
        content: r.unwrap(),
    };
    match management::update_setting(setting).await {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(format!("Failed updating settings: {:?}", e.0))
        },
    }
}

pub fn sync_to_remote(info: &GitRepositoryInfo) -> Result<(), String> {
    // open git repository
    let mut path = std::env::current_dir().unwrap();
    path.join(&info.repository_name);
    let repo = match Repository::open(path.as_path()) {
        Ok(repo) => repo,
        Err(e) => {
            return Err(format!("failed to open git repository: {}", e));
        },
    };
    // perform committing
    let changed_files = match get_changed_files(&repo) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Failed to get changed files: {}", e));
        }
    };
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
    let mut files: Vec<String> = Vec::with_capacity(25);
    for f in status.iter() {
        let path = dbg!(f.path().unwrap());
        files.push(String::from(path));
    }
    Ok(files)
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
