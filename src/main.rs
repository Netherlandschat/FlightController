use log::{info, trace, warn};
use crate::config::Config;
use std::path::PathBuf;
use crate::compiler::{ModuleCompiler, CompilerError};
use rayon::iter::{IntoParallelRefIterator, IndexedParallelIterator, ParallelIterator};

mod config;
mod git;
mod compiler;

fn main() {
    env_logger::init();

    info!("Flight Controller preparing for takeoff");
    let start_timer = std::time::Instant::now();

    trace!("Reading command line arguments");
    let config = Config::new();
    config.repos.par_iter().enumerate().for_each(|(_, repo_url)| {
        let repo = match git::clone_repo(repo_url, &config) {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to clone repository at URL '{}': {}", &repo_url, e);
                return;
            }
        };

        let mut mod_path = PathBuf::from(&config.mod_folder);
        mod_path.push(format!("{}-{}.jar", &repo.owner(), &repo.name()));

        if mod_path.exists() {
            trace!("Module '{}/{}' is already compiled", repo.owner(), repo.name());
            if !config.recompile {
                trace!("Recompiling is disabled, not recompiling module '{}/{}'", repo.owner(), repo.name());
                info!("Module '{}/{}' exists, skipping", repo.owner(), repo.name());
                return;
            }

            trace!("Recompilation is enabled, recompiling module '{}/{}'", repo.owner(), repo.name());
        }

        let compiler = ModuleCompiler::new(repo.path(), mod_path);
        match compiler.compile() {
            Ok(_) => {},
            Err(e) => {
                match e.downcast_ref::<CompilerError>() {
                    Some(ce) => {
                        match ce {
                            CompilerError::CompilationFailed { stdout, stderr } => {
                                warn!("Failed to compile module '{}/{}':\n\
                                \tstdout: {}\n\
                                \tstderr: {}",
                                    repo.owner(),
                                    repo.name(),
                                    stdout.replace(r#"\r\n"#, r#"\n"#),
                                    stderr.replace(r#"\r\n"#, r#"\n"#)
                                )
                            },
                            CompilerError::UnsupportedProject => warn!("Failed to compile module '{}/{}': {:?}", repo.owner(), repo.name(), e)
                        }
                    },
                    None => {
                        warn!("Failed to compile module '{}/{}': {:?}", repo.owner(), repo.name(), e);
                        return;
                    }
                }
            }
        }
    });

    let elapsed = std::time::Instant::now().duration_since(start_timer);
    info!("Done. Took {} seconds", elapsed.as_secs_f32());
    info!("Ready for takeoff!");
}