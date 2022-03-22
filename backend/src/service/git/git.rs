use std::path::{Path, PathBuf};
use std::vec::Vec;

use blog_common::dto::git::GitRepositoryInfo;
use git2::{
    BranchType, Commit, Cred, Direction, Error as GitError, ObjectType, Oid, PushOptions, Remote, RemoteCallbacks,
    Repository, RepositoryState, Signature, StatusOptions, StatusShow,
};

use crate::db::management;
use crate::db::model::Setting;

const SETTING_ITEM_NAME: &'static str = "git-pages";

pub fn get_repository_path(info: &GitRepositoryInfo) -> PathBuf {
    let path = std::env::current_dir().unwrap();
    let path = path.join("git-pages");
    path.join(&info.repository_name)
}

pub(crate) async fn must_get_repository_info() -> Result<GitRepositoryInfo, String> {
    let setting = management::get_setting(SETTING_ITEM_NAME)
        .await
        .map_err(|e| format!("Failed get git repository info: {:?}", e.0))?;
    if setting.is_none() || setting.as_ref().unwrap().content.is_empty() {
        Err(String::from("Cannot find git repository setting"))
    } else {
        let setting = setting.unwrap();
        serde_json::from_str::<GitRepositoryInfo>(&setting.content)
            .map_err(|e| format!("Failed deserialize git repository info: {}", e))
    }
}

pub(crate) async fn update_git_repository_info(info: &GitRepositoryInfo) -> Result<(), String> {
    let r = serde_json::to_string(info).map_err(|e| format!("Failed to serialize git repository info: {}", e))?;
    update_setting(r).await
}

pub async fn new_repository(info: GitRepositoryInfo) -> Result<(), String> {
    // clone repository
    let path = get_repository_path(&info);
    if path.exists() {
        return Err(format!("Target directory {} already exists", path.as_path().display()));
    }
    if let Err(e) = std::fs::create_dir_all(path.as_path()) {
        return Err(format!("Failed creating directory: {}, {}", path.as_path().display(), e));
    }
    if let Err(e) = Repository::clone(&info.remote_url, path.as_path()) {
        return Err(format!("Failed clone git repository: {}", e));
    };
    // save to db
    update_git_repository_info(&info).await
}

pub async fn remove_repository(info: GitRepositoryInfo) -> Result<(), String> {
    let path = get_repository_path(&info);
    if path.exists() {
        if let Err(e) = crate::util::io::remove_dir(path) {
            return Err(format!("Failed to remove git folder: {:?}", e));
        }
    }
    update_setting(String::new()).await
}

pub(crate) fn get_repo(info: &GitRepositoryInfo) -> Result<Repository, GitError> {
    let path = get_repository_path(info);
    // Repository::open(path.as_path()).map_err(|e| format!("failed to open git repository: {}", e))
    Repository::open(path.as_path())
}

pub fn get_branches(info: &GitRepositoryInfo) -> Result<Vec<String>, GitError> {
    let repo = get_repo(info)?;
    let remote_branches = repo.branches(Some(BranchType::Remote))?;
    let mut branches: Vec<String> = Vec::with_capacity(10);
    for branch in remote_branches {
        if let Ok((branch, _branch_type)) = branch {
            if let Ok(Some(name)) = branch.name() {
                // if matches!(branch.name(), Ok(Some(name)) if !name.contains("HEAD")) {
                if !name.contains("HEAD") {
                    let b = name.replace("origin/", "");
                    branches.push(b);
                }
            }
        }
    }
    Ok(branches)
}

// https://newbedev.com/how-to-get-the-behaviour-of-git-checkout-in-rust-git2
pub fn set_branch(info: &GitRepositoryInfo, branch_name: &str) -> Result<(), GitError> {
    let repo = get_repo(info)?;
    let head = repo.head()?;
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid)?;
    let _branch = repo.branch(branch_name, &commit, false)?;
    let refs = format!("refs/heads/{}", branch_name);
    let obj = repo.revparse_single(&refs)?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&refs)
    /*
    let ref_name = format!("refs/heads/{}", branch);
    let reference = repo.find_reference(&ref_name)?;
    let name = match reference.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(reference.name_bytes()).to_string(),
    };
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default().safe(),
    ))?;
    */
    /*
    let (object, reference) = repo.revparse_ext(branch)?;
    repo.checkout_tree(&object, None)?;
    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }?;
    */
    // Ok(())
}

async fn update_setting(content: String) -> Result<(), String> {
    let setting = Setting {
        item: String::from(SETTING_ITEM_NAME),
        content,
    };
    management::update_setting(setting)
        .await
        .map_err(|e| format!("Failed updating settings: {:?}", e.0))
}

pub fn sync_to_remote(info: &GitRepositoryInfo, password: &str) -> Result<(), GitError> {
    // open git repository
    let repo = get_repo(info)?;
    // find changed files
    let changed_files = get_changed_files(&repo)?;
    if changed_files.len() > 0 {
        // perform committing
        add_and_commit(info, &repo, changed_files, "Update posts")?;
        // try pushing
        push(&repo, info, password)?;
    }
    Ok(())
}

fn get_signature<'a>(repo: &'a Repository, info: &'a GitRepositoryInfo) -> Result<Signature<'a>, GitError> {
    let sig = match repo.signature() {
        Ok(sig) => sig,
        Err(e) => {
            println!("{:?}", e);
            let mut config = repo.config()?;
            config.set_str("user.name", &info.name)?;
            config.set_str("user.email", &info.email)?;
            Signature::now(&info.name, &info.email)?
        },
    };
    Ok(sig)
}

fn get_changed_files(repo: &Repository) -> Result<Vec<String>, GitError> {
    let state = dbg!(repo.state());
    if !state.eq(&RepositoryState::Clean) {
        println!("Git repository is not clean");
        return Ok(vec![]);
    }
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    // let status = repo.statuses(Some(status_options.show(StatusShow::Index)))?;
    let status = repo.statuses(Some(&mut opts))?;
    let mut files: Vec<String> = Vec::with_capacity(25);
    for f in status.iter().filter(|e| e.status() != git2::Status::IGNORED) {
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
    files: Vec<String>,
    message: &str,
) -> Result<Oid, GitError> {
    let signature = get_signature(repo, info)?;
    // let signature = repo.signature()?;
    let mut index = repo.index()?;
    for file in files.iter() {
        index.add_path(Path::new(&file))?;
    }
    index.write()?;
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
    )
}

fn create_callbacks<'a, 'b: 'a>(
    info: &'a GitRepositoryInfo,
    password: &'b str,
) -> Result<RemoteCallbacks<'a>, GitError> {
    let mut callbacks = RemoteCallbacks::new();
    let _ = callbacks.credentials(|str, str_opt, cred_type| Cred::userpass_plaintext(&info.name, password));
    Ok(callbacks)
}

fn push(repo: &Repository, info: &GitRepositoryInfo, password: &str) -> Result<(), GitError> {
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => repo.remote("origin", &info.remote_url)?,
    };
    remote.connect_auth(Direction::Push, Some(create_callbacks(info, password)?), None)?;
    let branch_name = info.branch_name.as_ref().unwrap();
    let refs = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    repo.remote_add_push("origin", &refs)?;
    let mut push_options = PushOptions::default();
    push_options.remote_callbacks(create_callbacks(info, password)?);

    remote.push(&[&refs], Some(&mut push_options))
}
