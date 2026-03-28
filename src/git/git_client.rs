use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};

pub struct GitClient {
    branch: String,
    personal_access_token: String,
    username: String,
    repository_name: String,
}

impl GitClient {
    pub fn new(
        branch: impl Into<String>,
        personal_access_token: impl Into<String>,
        username: impl Into<String>,
        repository_name: impl Into<String>,
    ) -> Self {
        Self {
            branch: branch.into(),
            personal_access_token: personal_access_token.into(),
            username: username.into(),
            repository_name: repository_name.into(),
        }
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

    pub fn pull(&self) -> Result<(), git2::Error> {
        let repo = Repository::open(&self.repository_name)?;
        let mut remote = repo.find_remote("origin")?;
        let branch = self.branch.clone();

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(self.create_callbacks());

        tracing::info!("Fetching from origin/{}", branch);
        remote
            .fetch(&*vec![branch], Some(&mut fetch_options), None)
            .unwrap_or_else(|e| tracing::error!("An error occurred while fetching: {}", e));

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        tracing::info!("Fetch commit: {}", fetch_commit.id());

        Ok(())
    }
}
