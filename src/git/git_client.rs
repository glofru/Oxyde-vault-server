use crate::errors::AppError;
use git2::build::CheckoutBuilder;
use git2::{
    Cred, FetchOptions, ObjectType, Oid, RemoteCallbacks, Repository, TreeWalkMode, TreeWalkResult,
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct GitClient {
    branch: String,
    personal_access_token: String,
    username: String,
    repository: Arc<Mutex<Repository>>,
    latest_commit_id: Option<Oid>,
}

pub struct File {
    pub path: String,
    pub content: Vec<u8>,
}

pub struct GetCommitDataResponse {
    pub last_file_id: Option<String>,
    pub files: Vec<File>,
}

impl GitClient {
    pub fn new(
        branch: impl Into<String>,
        personal_access_token: impl Into<String>,
        username: impl Into<String>,
        repository_name: impl Into<String>,
    ) -> Result<Self, git2::Error> {
        let repository_name: String = repository_name.into();
        let repository = Arc::new(Mutex::new(Repository::open(&repository_name)?));
        Ok(Self {
            branch: branch.into(),
            personal_access_token: personal_access_token.into(),
            username: username.into(),
            repository,
            latest_commit_id: None,
        })
    }

    fn create_callbacks(
        username: String,
        personal_access_token: String,
    ) -> RemoteCallbacks<'static> {
        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(move |_url, _username_from_url, _access_types| {
            Cred::userpass_plaintext(&username, &personal_access_token)
        });

        callbacks
    }

    pub fn get_commit_data(
        &self,
        commit_id: &str,
        from_file_id: Option<String>,
        maximum_size: usize,
    ) -> Result<GetCommitDataResponse, AppError> {
        let repository = self.repository.lock().unwrap();

        tracing::info!("Getting commit {}", commit_id);
        let commit = git_helper::get_commit(&repository, commit_id)?;
        tracing::info!(
            "Got commit {} with message: {}",
            commit_id,
            commit.message().unwrap_or("null")
        );

        let tree = commit.tree()?;

        tracing::info!("Walking tree of commit {}", commit_id);
        let from_file_id =
            from_file_id.inspect(|value| tracing::info!("Starting from file id {}", value));

        let mut files = Vec::new();
        let mut skip = from_file_id.is_some();
        let start_file_id = from_file_id.unwrap_or(String::new());
        let mut end_file_id: Option<String> = None;
        let mut cumulative_size = 0usize;

        let walk_result = tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            // We only care about files (blobs), not directories (trees)
            if entry.kind() != Some(ObjectType::Blob) {
                return TreeWalkResult::Ok;
            }

            if skip {
                if entry.id().to_string() != start_file_id {
                    return TreeWalkResult::Ok;
                }
                skip = false;
            }

            match entry.to_object(&repository) {
                Ok(obj) => {
                    if let Some(blob) = obj.as_blob() {
                        cumulative_size += blob.size();

                        if cumulative_size > maximum_size && files.len() > 1 {
                            end_file_id = Some(entry.id().to_string());
                            return TreeWalkResult::Abort;
                        }

                        let file_path = format!("{}{}", root, entry.name().unwrap_or(""));
                        files.push(File {
                            path: file_path,
                            content: blob.content().to_vec(),
                        })
                    }
                }
                Err(err) => {
                    tracing::error!("Error during TreeWalk: {}", err);
                    return TreeWalkResult::Abort;
                }
            }
            TreeWalkResult::Ok
        });

        tracing::info!("Walked commit {} with {} files", commit_id, files.len());

        match walk_result {
            Ok(_) => Ok(GetCommitDataResponse {
                files,
                last_file_id: None,
            }),
            Err(error) => {
                let Some(last_file_id) = end_file_id else {
                    tracing::error!("Walking tree aborted but last file id is missing");
                    return Err(AppError::InternalServerError(anyhow::Error::new(error)));
                };

                tracing::info!("Last file id {}", last_file_id);

                Ok(GetCommitDataResponse {
                    files,
                    last_file_id: Some(last_file_id),
                })
            }
        }
    }

    pub fn get_latest_commit_id(&mut self) -> Result<Oid, AppError> {
        match self.latest_commit_id {
            None => self.fetch(),
            Some(commit_id) => Ok(commit_id),
        }
    }

    pub fn fetch(&mut self) -> Result<Oid, AppError> {
        let repository = self.repository.lock().unwrap();
        let mut remote = repository.find_remote("origin")?;
        let branch = self.branch.clone();

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(Self::create_callbacks(
            self.username.clone(),
            self.personal_access_token.clone(),
        ));

        tracing::info!("Fetching from origin/{}", branch);
        remote
            .fetch(&*vec![branch], Some(&mut fetch_options), None)
            .unwrap_or_else(|e| tracing::error!("An error occurred while fetching: {}", e));

        let fetch_head = repository.find_reference("FETCH_HEAD")?;
        let fetch_commit = repository.reference_to_annotated_commit(&fetch_head)?;

        self.latest_commit_id = Some(fetch_commit.id());

        tracing::info!("Fetch commit: {}", fetch_commit.id());

        Ok(fetch_commit.id())
    }

    pub fn pull(&mut self) -> Result<Oid, AppError> {
        let commit_id = self.fetch()?;

        tracing::info!("Pull commit: {}", commit_id);

        let repository = self.repository.lock().unwrap();
        let commit = git_helper::get_annotated_commit(&repository, commit_id.to_string().as_str())?;
        let (analysis, _) = repository.merge_analysis(&[&commit])?;

        if analysis.is_up_to_date() {
            tracing::info!("Local branch is already up to date.")
        } else if analysis.is_fast_forward() {
            let refname = format!("refs/heads/{}", self.branch);
            let mut reference = repository.find_reference(&refname)?;

            reference.set_target(commit.id(), "Fast-Forward Pull")?;

            repository.set_head(&refname)?;
            repository.checkout_head(Some(CheckoutBuilder::default().force()))?;

            tracing::info!("Fast-forward complete");
        }

        Ok(commit_id)
    }
}

mod git_helper {
    use crate::errors::AppError;
    use git2::{AnnotatedCommit, Commit, Oid, Repository};

    pub fn get_annotated_commit<'a>(
        repository: &'a Repository,
        commit_id: &str,
    ) -> Result<AnnotatedCommit<'a>, AppError> {
        let Ok(oid) = Oid::from_str(commit_id) else {
            return Err(AppError::BadRequest(format!(
                "{} not a valid id",
                commit_id
            )));
        };
        let Ok(commit) = repository.find_annotated_commit(oid) else {
            return Err(AppError::NotFound(format!("{} not found", commit_id)));
        };
        Ok(commit)
    }

    pub fn get_commit<'a>(
        repository: &'a Repository,
        commit_id: &str,
    ) -> Result<Commit<'a>, AppError> {
        let Ok(obj) = repository.revparse_single(commit_id) else {
            return Err(AppError::NotFound(format!("{} not found", commit_id)));
        };
        let Ok(commit) = obj.peel_to_commit() else {
            return Err(AppError::NotFound(format!("{} not found", commit_id)));
        };
        Ok(commit)
    }
}
