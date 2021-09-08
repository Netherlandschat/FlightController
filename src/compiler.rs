use std::path::{PathBuf, Path};
use anyhow::Result;
use std::process::{Command, Stdio};
use std::fs;
use thiserror::Error;
use log::{trace, info};

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Failed to compile module. Stdout: '{stdout:?}'; Stderr: '{stderr:?}'")]
    CompilationFailed {
        stdout: String,
        stderr: String
    },
    #[error("Unable to compile module as the project type is not supported")]
    UnsupportedProject
}

pub struct ModuleCompiler<PS, PT> where PS: AsRef<Path>, PT: AsRef<Path> {
    source_dir: PS,
    target_dir: PT,
    mod_name:   String
}

impl<PS, PT> ModuleCompiler<PS, PT>
where PS: AsRef<Path>, PT: AsRef<Path> {

    pub fn new(source_dir: PS, target_dir: PT) -> Self {
        let tpath = target_dir.as_ref();
        let tpath_components = tpath.to_str()
            .unwrap()
            .replace("\\", "/")
            .split("/")
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        let path_ending = tpath_components.last()
            .unwrap()
            .split("-")
            .map(|f| f.replace(".jar", ""))
            .collect::<Vec<String>>();


        let mod_name = format!("{}/{}", path_ending.get(0).unwrap(), path_ending.get(1).unwrap());

        Self {
            target_dir,
            source_dir,
            mod_name
        }
    }

    fn is_gradle(&self) -> Result<bool> {
        let src = self.source_dir.as_ref();
        trace!("Reading directory {:?} for project type identification", &src);
        let paths = fs::read_dir(src)?;

        for path in paths {
            let p = path?.path();
            if !p.is_file() {
                continue;
            }

            let name = p.file_name().unwrap();
            if name.eq("build.gradle") {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn compile(&self) -> Result<()> {
        info!("Compiling module '{}'", &self.mod_name);

        let compiler: Box<dyn Compiler> = if self.is_gradle()? {
            Box::new(GradleCompiler::new(&self))
        } else {
            return Err(CompilerError::UnsupportedProject.into());
        };

        trace!("Starting compilation for module '{}'", &self.mod_name);
        compiler.compile()?;
        trace!("Module '{}' compiled, copying", &self.mod_name);
        compiler.copy()?;

        Ok(())
    }
}

trait Compiler {
    fn compile(&self) -> Result<()>;
    fn copy(&self) -> Result<()>;
}

struct GradleCompiler<'a, PS, PT> where PS: AsRef<Path>, PT: AsRef<Path> {
    cmp: &'a ModuleCompiler<PS, PT>
}

impl<'a, PS, PT> GradleCompiler<'a, PS, PT> where PS: AsRef<Path>, PT: AsRef<Path> {
    fn new(cmp: &'a ModuleCompiler<PS, PT>) -> Self {
        Self {
            cmp
        }
    }
}

impl<'a, PS, PT> Compiler for GradleCompiler<'a, PS, PT> where PS: AsRef<Path>, PT: AsRef<Path> {
    fn compile(&self) -> Result<()> {
        trace!("Spawning Gradle process for '{}'", &self.cmp.mod_name);
        trace!("Working directory for Gradle: {:?}", &self.cmp.source_dir.as_ref());

        let mut gradlew = PathBuf::from(&self.cmp.source_dir.as_ref());
        #[cfg(windows)]
        gradlew.push("gradlew.bat");
        #[cfg(not(windows))]
        gradlew.push("gradlew");

        #[cfg(not(windows))]
        {
            trace!("Applying 0o770 permissions to gradlew script for module '{}'", &self.cmp.mod_name);
            use std::os::unix::fs::PermissionsExt;
            let perms = PermissionsExt::from_mode(0o770);
            fs::set_permissions(&gradlew, perms)?;
        }

        let compilation_status = Command::new(gradlew)
            .args(&["build"])
            .current_dir(&self.cmp.source_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        trace!("Gradle process completed for '{}'", &self.cmp.mod_name);

        if compilation_status.status.code() != Some(0) {
            let stderr = String::from_utf8(compilation_status.stderr)?;
            let stdout = String::from_utf8(compilation_status.stdout)?;
            return Err(CompilerError::CompilationFailed {
                stdout,
                stderr
            }.into())
        }

        return Ok(());    }

    fn copy(&self) -> Result<()> {
        let mut build_dir = PathBuf::from(self.cmp.source_dir.as_ref());
        build_dir.push("build/libs");

        trace!("Reading contents of build directory for module '{}'", &self.cmp.mod_name);
        let contents = fs::read_dir(&build_dir)?;
        for entry in contents {
            let p = entry?.path();
            if p.is_file(){
                let name = p.file_name().unwrap().to_str().unwrap();
                if name.ends_with("-all.jar") {
                    let target_dir = PathBuf::from(self.cmp.target_dir.as_ref());

                    trace!("Copying artifact {:?} to {:?} for module '{}'", &p, &target_dir, &self.cmp.mod_name);
                    fs::copy(&p, &target_dir)?;
                }
            }
        }

        Ok(())
    }
}