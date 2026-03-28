use git2::build::CheckoutBuilder;
use git2::{AnnotatedCommit, Cred, FetchOptions, Oid, RemoteCallbacks, Repository};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct GitClient {
    branch: String,
    personal_access_token: String,
    username: String,
    repository_name: String,
    repository: Arc<Mutex<Repository>>,
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
            repository_name: repository_name.into(),
            repository,
        })
    }

    fn create_callbacks(&self) -> RemoteCallbacks<'_> {
        let mut callbacks = RemoteCallbacks::new();

        let username = self.username.clone();
        let personal_access_token = self.personal_access_token.clone();

        callbacks.credentials(move |_url, _username_from_url, _access_types| {
            Cred::userpass_plaintext(&username, &personal_access_token)
        });

        callbacks
    }

    pub fn fetch(&self) -> Result<Oid, git2::Error> {
        let repository = self.repository.lock().unwrap();
        let mut remote = repository.find_remote("origin")?;
        let branch = self.branch.clone();

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(self.create_callbacks());

        tracing::info!("Fetching from origin/{}", branch);
        remote
            .fetch(&*vec![branch], Some(&mut fetch_options), None)
            .unwrap_or_else(|e| tracing::error!("An error occurred while fetching: {}", e));

        let fetch_head = repository.find_reference("FETCH_HEAD")?;
        let fetch_commit = repository.reference_to_annotated_commit(&fetch_head)?;

        tracing::info!("Fetch commit: {}", fetch_commit.id());

        Ok(fetch_commit.id())
    }

    pub fn pull(&self) -> Result<Oid, git2::Error> {
        let commit_id = self.fetch()?;

        tracing::info!("Pull commit: {}", commit_id);

        let repository = self.repository.lock().unwrap();
        let commit = repository.find_annotated_commit(commit_id)?;
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
