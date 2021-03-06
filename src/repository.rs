use reqwest;

use crate::app::AppError;

/// Supported hosts. [GitHub][RepositoryHost::GitHub] is the default one.
#[derive(Debug)]
pub(crate) enum RepositoryHost {
  GitHub,
  GitLab,
  BitBucket,
}

impl Default for RepositoryHost {
  fn default() -> Self {
    RepositoryHost::GitHub
  }
}

/// Container for a repository host.
#[derive(Debug)]
pub(crate) enum Host {
  Known(RepositoryHost),
  Unknown,
}

impl Default for Host {
  fn default() -> Self {
    Host::Known(RepositoryHost::default())
  }
}

/// Repository meta, i.e. *ref*.
///
/// This newtype exists solely for providing the default value.
#[derive(Debug)]
pub(crate) struct RepositoryMeta(pub String);

impl Default for RepositoryMeta {
  fn default() -> Self {
    // TODO: Get the default value for meta from somewhere else, e.g. from env variables or config.
    RepositoryMeta("master".to_string())
  }
}

#[derive(Debug)]
pub(crate) struct Repository {
  pub host: RepositoryHost,
  pub user: String,
  pub repo: String,
  pub meta: RepositoryMeta,
}

impl Repository {
  /// Resolves a URL depending on the host and other repository fields.
  pub(crate) fn get_tar_url(&self) -> String {
    let Repository {
      host,
      user,
      repo,
      meta,
    } = self;

    let RepositoryMeta(meta) = meta;

    match host {
      | RepositoryHost::GitHub => {
        format!("https://github.com/{user}/{repo}/archive/{meta}.tar.gz")
      },
      | RepositoryHost::GitLab => {
        format!("https://gitlab.com/{user}/{repo}/repository/archive.tar.gz?ref={meta}")
      },
      | RepositoryHost::BitBucket => {
        format!("https://bitbucket.org/{user}/{repo}/get/{meta}.tar.gz")
      },
    }
  }

  /// Fetches the tarball using the resolved URL, and reads it into bytes (`Vec<u8>`).
  pub(crate) async fn fetch(&self) -> Result<Vec<u8>, AppError> {
    let response = reqwest::get(self.get_tar_url()).await.map_err(|err| {
      err
        .status()
        .map_or(AppError(format!("Request failed.")), |status| {
          AppError(format!(
            "Request failed with the code: {code}.",
            code = status.as_u16()
          ))
        })
    })?;

    let bytes = response
      .bytes()
      .await
      .map(|bytes| bytes.to_vec())
      .map_err(|_| AppError(format!("Couldn't get the response body as bytes.")))?;

    Ok(bytes)
  }
}
