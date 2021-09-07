use anyhow::Result;
use std::path::{Path, PathBuf};
use git2::Repository;
use crate::config::Config;
use temp_dir::TempDir;
use log::{trace, info};

pub struct LocalRepo<P: AsRef<Path>> {
    path:   P,
    owner:  String,
    name:   String,
}

impl<P> LocalRepo<P>
where P: AsRef<Path>{
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn path(&self) -> &P {
        &self.path
    }

    pub fn clone<U>(url: U, path: P, owner: String, name: String) -> Result<Self>
    where U: AsRef<str> {

        Repository::clone(url.as_ref(), &path)?;
        Ok(Self {
            path,
            owner,
            name
        })
    }

    pub fn new(path: P, owner: String, name: String) -> Self {
        Self {
            path,
            owner,
            name
        }
    }
}

pub(crate) fn clone_repo(url: &str, config: &Config) -> anyhow::Result<LocalRepo<PathBuf>> {
    let components: Vec<&str> = url.split("/").collect();
    let owner = components.get(components.len() - 2)
        .expect("Unable to get repository owner, is the URL valid?")
        .to_lowercase();
    let name = components.get(components.len() - 1)
        .expect("Unable to get repository name, is the URL valid?")
        .replace(".git", "")
        .to_lowercase();

    if !config.recompile {
        trace!("Checking if repository '{}/{}' already exists on disk", &owner, &name);
        let mut maybe_exist_path = PathBuf::from(&config.mod_folder);
        maybe_exist_path.push(format!("{}-{}.jar", &owner, &name));
        if maybe_exist_path.exists() {
            trace!("Repository '{}/{}' already exists at {:?}", &owner, &name, &maybe_exist_path);
            return Ok(LocalRepo::new(maybe_exist_path, owner, name));
        }

        trace!("Repository '{}/{}' does not exist locally", &owner, &name);
    }

    trace!("Creating temporary directory for repository '{}/{}'", &owner, &name);
    let tmpdir = TempDir::new()?;
    let path = PathBuf::from(tmpdir.path());

    info!("Cloning repository '{}/{}'", &owner, &name);
    trace!("Cloning into {:?}", &path);
    let repo = LocalRepo::clone(url, path, owner, name)?;

    Ok(repo)
}