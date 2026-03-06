use anyhow::{Context, Result};
use git2::{
    build::RepoBuilder, Cred, FetchOptions, IndexAddOption, RemoteCallbacks, Repository, Signature,
};
use std::path::Path;

pub struct GitOps {
    repo: Repository,
}

impl GitOps {
    pub fn open(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path).context("Failed to open repository")?;
        Ok(Self { repo })
    }

    pub fn repo_path(&self) -> &Path {
        self.repo.path().parent().unwrap_or(self.repo.path())
    }
}

fn create_remote_callbacks(ssh_key: &str) -> RemoteCallbacks<'_> {
    let ssh_key = ssh_key.to_string();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        let username = username_from_url.unwrap_or("git");
        Cred::ssh_key_from_memory(username, None, &ssh_key, None)
    });
    callbacks
}

fn create_fetch_options<'a>(callbacks: RemoteCallbacks<'a>) -> FetchOptions<'a> {
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);
    fo
}

impl GitOps {
    pub fn clone(
        repo_url: &str,
        base_branch: &str,
        target_dir: &Path,
        ssh_key: &str,
    ) -> Result<Self> {
        let callbacks = create_remote_callbacks(ssh_key);
        let fo = create_fetch_options(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);
        builder.branch(base_branch);

        let repo = builder
            .clone(repo_url, target_dir)
            .context("Failed to clone repository")?;

        Ok(Self { repo })
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        let head = self.repo.head().context("Failed to get HEAD")?;
        let target = head.target().context("HEAD has no target")?;
        let commit = self
            .repo
            .find_commit(target)
            .context("Failed to find commit")?;

        self.repo
            .branch(branch_name, &commit, false)
            .context("Failed to create branch")?;

        let branch = self
            .repo
            .find_branch(branch_name, git2::BranchType::Local)
            .context("Failed to find created branch")?;
        let refname = branch
            .get()
            .name()
            .context("Branch has no name")?
            .to_owned();
        self.repo
            .set_head(&refname)
            .context("Failed to set HEAD to new branch")?;
        self.repo
            .checkout_head(None)
            .context("Failed to checkout HEAD")?;

        Ok(())
    }

    pub fn add_all(&self) -> Result<()> {
        let mut index = self.repo.index().context("Failed to get index")?;
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .context("Failed to add files to index")?;
        index.write().context("Failed to write index")?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index().context("Failed to get index")?;
        let tree_id = index.write_tree().context("Failed to write tree")?;
        let tree = self
            .repo
            .find_tree(tree_id)
            .context("Failed to find tree")?;

        let head = self.repo.head().ok();
        let parents: Vec<_> = match head {
            Some(h) => {
                let target = h.target().context("HEAD has no target")?;
                vec![self
                    .repo
                    .find_commit(target)
                    .context("Failed to find commit")?]
            }
            None => vec![],
        };

        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        let signature = Signature::now("parallel-worker", "worker@parallel.dev")
            .context("Failed to create signature")?;

        let result = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        );

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.code() == git2::ErrorCode::NotFound {
                    Ok(())
                } else {
                    Err(e).context("Failed to create commit")?
                }
            }
        }
    }

    pub fn push(&self, branch_name: &str, ssh_key: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote("origin")
            .context("Failed to find remote origin")?;

        let callbacks = create_remote_callbacks(ssh_key);
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        remote
            .push(
                &[&refspec],
                Some(&mut git2::PushOptions::new().remote_callbacks(callbacks)),
            )
            .context("Failed to push")?;

        Ok(())
    }

    pub fn diff(&self) -> Result<String> {
        let mut diff_buf = Vec::new();

        let head = self.repo.head().context("Failed to get HEAD")?;
        let target = head.target().context("HEAD has no target")?;
        let commit = self
            .repo
            .find_commit(target)
            .context("Failed to find commit")?;
        let tree = commit.tree().context("Failed to get tree")?;

        let diff = self
            .repo
            .diff_tree_to_workdir(Some(&tree), None)
            .context("Failed to create diff")?;

        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_buf.extend_from_slice(line.content());
            true
        })
        .context("Failed to print diff")?;

        Ok(String::from_utf8_lossy(&diff_buf).to_string())
    }

    pub fn fetch(&self, ssh_key: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote("origin")
            .context("Failed to find remote origin")?;

        let callbacks = create_remote_callbacks(ssh_key);
        let mut fo = create_fetch_options(callbacks);

        remote
            .fetch(&[] as &[&str], Some(&mut fo), None)
            .context("Failed to fetch")?;

        Ok(())
    }

    pub fn force_checkout(&self, branch: &str) -> Result<()> {
        let branch_ref = format!("refs/remotes/origin/{}", branch);

        let commit = match self.repo.find_reference(&branch_ref) {
            Ok(r) => {
                let target = r.target().context("Remote branch has no target")?;
                self.repo
                    .find_commit(target)
                    .context("Failed to find commit")?
            }
            Err(_) => {
                let local_ref = format!("refs/heads/{}", branch);
                let r = self
                    .repo
                    .find_reference(&local_ref)
                    .context("Failed to find branch")?;
                let target = r.target().context("Branch has no target")?;
                self.repo
                    .find_commit(target)
                    .context("Failed to find commit")?
            }
        };

        let tree = commit.tree().context("Failed to get tree")?;

        self.repo
            .checkout_tree(tree.as_object(), None)
            .context("Failed to checkout tree")?;

        let branch_ref = format!("refs/heads/{}", branch);
        self.repo
            .set_head(&branch_ref)
            .context("Failed to set HEAD")?;

        Ok(())
    }

    pub fn delete_branch_if_exists(&self, branch: &str) -> Result<()> {
        match self.repo.find_branch(branch, git2::BranchType::Local) {
            Ok(mut b) => {
                b.delete().context("Failed to delete branch")?;
            }
            Err(e) => {
                if e.code() != git2::ErrorCode::NotFound {
                    tracing::debug!("Branch delete info: {}", e);
                }
            }
        }
        Ok(())
    }
}
