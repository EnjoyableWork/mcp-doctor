use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::contract::{RenderedReportArtifact, ReportArtifactFormat};

const STAGE_CREATE_ATTEMPTS: u64 = 16;
static NEXT_STAGE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReportArtifactError {
    InvalidPath,
    ExistingOutput,
    NonRegularOutput,
    ParentUnavailable,
    DuplicateDestination,
    AliasedDestination,
    Create,
    Write,
    Publish,
    Cleanup,
    RenderContract,
}

impl fmt::Display for ReportArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPath => "a report destination is not a valid new-file target",
            Self::ExistingOutput => {
                "a report destination already exists; overwrite is not supported"
            }
            Self::NonRegularOutput => "a report destination names a non-regular filesystem entry",
            Self::ParentUnavailable => {
                "each report destination parent must already exist as a directory"
            }
            Self::DuplicateDestination => "JSON and JUnit require distinct report destinations",
            Self::AliasedDestination => {
                "requested artifact destinations alias the same filesystem path"
            }
            Self::Create => "report destinations could not be prepared safely",
            Self::Write => "requested report artifacts could not be written completely",
            Self::Publish => "requested report artifacts could not be published without overwrite",
            Self::Cleanup => "requested report artifact cleanup did not complete",
            Self::RenderContract => "rendered report artifacts did not match the requested formats",
        })
    }
}

impl Error for ReportArtifactError {}

#[derive(Debug)]
struct Destination {
    format: ReportArtifactFormat,
    path: PathBuf,
    stage_path: PathBuf,
    stage: Option<File>,
    published: bool,
    committed: bool,
}

impl Drop for Destination {
    fn drop(&mut self) {
        if self.published
            && !self.committed
            && output_is_owned(self).is_ok_and(|owned| owned == Some(true))
        {
            let _ = fs::remove_file(&self.path);
        }
        self.stage.take();
        let _ = fs::remove_file(&self.stage_path);
    }
}

#[derive(Debug)]
pub(crate) struct ReportArtifactDestinations {
    destinations: Vec<Destination>,
}

impl ReportArtifactDestinations {
    pub(crate) fn prepare(
        json: Option<PathBuf>,
        junit: Option<PathBuf>,
        reserved_paths: &[&Path],
    ) -> Result<Self, ReportArtifactError> {
        if json.is_some() && json == junit {
            return Err(ReportArtifactError::DuplicateDestination);
        }

        let reserved_identities = reserved_paths
            .iter()
            .map(|path| destination_identity(path))
            .collect::<Result<Vec<_>, _>>()?;

        let mut destinations = Vec::with_capacity(2);
        for (format, path) in [
            (ReportArtifactFormat::Json, json),
            (ReportArtifactFormat::Junit, junit),
        ] {
            let Some(path) = path else {
                continue;
            };
            let identity = destination_identity(&path)?;
            if destinations
                .iter()
                .any(|existing: &Destination| identities_alias(&existing.path, &identity))
                || reserved_identities
                    .iter()
                    .any(|reserved| identities_alias(&identity, reserved))
            {
                return Err(ReportArtifactError::AliasedDestination);
            }
            reject_existing_output(&identity)?;
            let (stage_path, stage) = create_stage(&identity)?;
            destinations.push(Destination {
                format,
                path: identity,
                stage_path,
                stage: Some(stage),
                published: false,
                committed: false,
            });
        }

        let mut prepared = Self { destinations };
        prepared.prove_distinct_no_clobber_paths(&reserved_identities)?;
        Ok(prepared)
    }

    pub(crate) fn requests_json(&self) -> bool {
        self.destinations
            .iter()
            .any(|destination| destination.format == ReportArtifactFormat::Json)
    }

    pub(crate) fn requests_junit(&self) -> bool {
        self.destinations
            .iter()
            .any(|destination| destination.format == ReportArtifactFormat::Junit)
    }

    pub(crate) fn persist(
        mut self,
        artifacts: Vec<RenderedReportArtifact>,
    ) -> Result<(), ReportArtifactError> {
        if artifacts.len() != self.destinations.len()
            || artifacts
                .iter()
                .zip(&self.destinations)
                .any(|(artifact, destination)| artifact.format != destination.format)
        {
            self.rollback()?;
            return Err(ReportArtifactError::RenderContract);
        }

        if internal_test_write_failure() {
            self.rollback()?;
            return Err(ReportArtifactError::Write);
        }

        for (artifact, destination) in artifacts.iter().zip(&mut self.destinations) {
            let Some(stage) = destination.stage.as_mut() else {
                self.rollback()?;
                return Err(ReportArtifactError::RenderContract);
            };
            if stage
                .write_all(artifact.output.as_bytes())
                .and_then(|()| stage.sync_all())
                .is_err()
            {
                self.rollback()?;
                return Err(ReportArtifactError::Write);
            }
        }

        for destination in &mut self.destinations {
            if fs::hard_link(&destination.stage_path, &destination.path).is_err() {
                self.rollback()?;
                return Err(ReportArtifactError::Publish);
            }
            destination.published = true;
        }

        for destination in &mut self.destinations {
            if fs::remove_file(&destination.stage_path).is_err() {
                self.rollback()?;
                return Err(ReportArtifactError::Cleanup);
            }
        }
        if internal_test_cleanup_failure() {
            self.rollback_without_hook()?;
            return Err(ReportArtifactError::Cleanup);
        }
        for destination in &mut self.destinations {
            destination.committed = true;
            destination.stage.take();
        }
        Ok(())
    }

    pub(crate) fn cancel(mut self) -> Result<(), ReportArtifactError> {
        self.rollback()
    }

    fn prove_distinct_no_clobber_paths(
        &mut self,
        reserved_identities: &[PathBuf],
    ) -> Result<(), ReportArtifactError> {
        for index in 0..self.destinations.len() {
            let destination = &mut self.destinations[index];
            if fs::hard_link(&destination.stage_path, &destination.path).is_err() {
                let error = if fs::symlink_metadata(&destination.path).is_ok() {
                    ReportArtifactError::AliasedDestination
                } else {
                    ReportArtifactError::Create
                };
                self.rollback_without_hook()?;
                return Err(error);
            }
            destination.published = true;
            if reserved_identities
                .iter()
                .any(|reserved| fs::symlink_metadata(reserved).is_ok())
            {
                self.rollback_without_hook()?;
                return Err(ReportArtifactError::AliasedDestination);
            }
        }
        self.rollback_outputs()?;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ReportArtifactError> {
        let result = self.rollback_without_hook();
        if result.is_err() || internal_test_cleanup_failure() {
            Err(ReportArtifactError::Cleanup)
        } else {
            Ok(())
        }
    }

    fn rollback_without_hook(&mut self) -> Result<(), ReportArtifactError> {
        let outputs = self.rollback_outputs();
        let mut stages_clean = true;
        for destination in &mut self.destinations {
            destination.stage.take();
            match fs::remove_file(&destination.stage_path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(_) => stages_clean = false,
            }
        }
        if outputs.is_err() || !stages_clean {
            Err(ReportArtifactError::Cleanup)
        } else {
            Ok(())
        }
    }

    fn rollback_outputs(&mut self) -> Result<(), ReportArtifactError> {
        let mut clean = true;
        for destination in self.destinations.iter_mut().rev() {
            if !destination.published {
                continue;
            }
            match output_is_owned(destination) {
                Ok(Some(true)) => {}
                Ok(None) => {
                    destination.published = false;
                    continue;
                }
                Ok(Some(false)) | Err(_) => {
                    clean = false;
                    continue;
                }
            }
            match fs::remove_file(&destination.path) {
                Ok(()) => destination.published = false,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    destination.published = false;
                }
                Err(_) => clean = false,
            }
        }
        if clean {
            Ok(())
        } else {
            Err(ReportArtifactError::Cleanup)
        }
    }
}

fn output_is_owned(destination: &Destination) -> io::Result<Option<bool>> {
    let Some(stage) = destination.stage.as_ref() else {
        return Ok(Some(false));
    };
    let stage = stage.metadata()?;
    let output = match fs::metadata(&destination.path) {
        Ok(output) => output,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        Ok(Some(
            stage.dev() == output.dev() && stage.ino() == output.ino(),
        ))
    }

    #[cfg(not(unix))]
    {
        let _ = (stage, output);
        Ok(Some(true))
    }
}

fn reject_existing_output(path: &Path) -> Result<(), ReportArtifactError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Err(ReportArtifactError::ExistingOutput),
        Ok(_) => Err(ReportArtifactError::NonRegularOutput),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ReportArtifactError::InvalidPath),
    }
}

fn destination_identity(path: &Path) -> Result<PathBuf, ReportArtifactError> {
    if path == Path::new("-") {
        return Err(ReportArtifactError::InvalidPath);
    }
    let Some(file_name) = path.file_name() else {
        return Err(ReportArtifactError::InvalidPath);
    };
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if !fs::metadata(parent).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(ReportArtifactError::ParentUnavailable);
    }
    let parent = fs::canonicalize(parent).map_err(|_| ReportArtifactError::ParentUnavailable)?;
    Ok(parent.join(file_name))
}

fn identities_alias(left: &Path, right: &Path) -> bool {
    left == right
        || (left.parent() == right.parent()
            && left
                .file_name()
                .zip(right.file_name())
                .is_some_and(|(left, right)| {
                    left.to_string_lossy()
                        .eq_ignore_ascii_case(&right.to_string_lossy())
                }))
}

fn create_stage(identity: &Path) -> Result<(PathBuf, File), ReportArtifactError> {
    if internal_test_create_failure() {
        return Err(ReportArtifactError::Create);
    }
    let parent = identity
        .parent()
        .expect("a prepared destination identity has a parent");
    for _ in 0..STAGE_CREATE_ATTEMPTS {
        let id = NEXT_STAGE_ID.fetch_add(1, Ordering::Relaxed);
        let stage_path = parent.join(format!(
            ".mcp-doctor-report-{}-{id}.tmp",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&stage_path) {
            Ok(stage) => return Ok((stage_path, stage)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(ReportArtifactError::Create),
        }
    }
    Err(ReportArtifactError::Create)
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_create_failure() -> bool {
    internal_test_flag("MCP_DOCTOR_INTERNAL_TEST_REPORT_CREATE_FAILURE")
}

#[cfg(not(feature = "internal-test-fixtures"))]
const fn internal_test_create_failure() -> bool {
    false
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_write_failure() -> bool {
    internal_test_flag("MCP_DOCTOR_INTERNAL_TEST_REPORT_WRITE_FAILURE")
}

#[cfg(not(feature = "internal-test-fixtures"))]
const fn internal_test_write_failure() -> bool {
    false
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_cleanup_failure() -> bool {
    internal_test_flag("MCP_DOCTOR_INTERNAL_TEST_REPORT_CLEANUP_FAILURE")
}

#[cfg(not(feature = "internal-test-fixtures"))]
const fn internal_test_cleanup_failure() -> bool {
    false
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_flag(name: &str) -> bool {
    std::env::var_os("MCP_DOCTOR_TEST_MODE").as_deref() == Some(std::ffi::OsStr::new("1"))
        && std::env::var_os(name).as_deref() == Some(std::ffi::OsStr::new("1"))
}

#[cfg(test)]
mod tests {
    use super::{ReportArtifactDestinations, ReportArtifactError};
    use crate::contract::{RenderedReportArtifact, ReportArtifactFormat};
    use std::fs;
    use tempfile::TempDir;

    fn artifacts() -> Vec<RenderedReportArtifact> {
        vec![
            RenderedReportArtifact {
                format: ReportArtifactFormat::Json,
                output: "{\"synthetic\":true}\n".to_owned(),
            },
            RenderedReportArtifact {
                format: ReportArtifactFormat::Junit,
                output: "<?xml version=\"1.0\"?><testsuites/>\n".to_owned(),
            },
        ]
    }

    fn assert_no_stages(root: &std::path::Path) {
        let stages = fs::read_dir(root)
            .expect("the disposable root should be readable")
            .map(|entry| entry.expect("the disposable entry should be readable"))
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".mcp-doctor-report-")
            })
            .collect::<Vec<_>>();
        assert!(stages.is_empty(), "owned report stages should be removed");
    }

    #[test]
    fn distinct_destinations_publish_exact_complete_bytes() {
        let root = TempDir::new().expect("a disposable root should be created");
        let json = root.path().join("report.json");
        let junit = root.path().join("report.xml");
        let destinations =
            ReportArtifactDestinations::prepare(Some(json.clone()), Some(junit.clone()), &[])
                .expect("distinct new destinations should prepare");
        assert!(destinations.requests_json());
        assert!(destinations.requests_junit());

        destinations
            .persist(artifacts())
            .expect("complete report bytes should publish");

        assert_eq!(
            fs::read_to_string(json).expect("JSON should be readable"),
            "{\"synthetic\":true}\n"
        );
        assert_eq!(
            fs::read_to_string(junit).expect("JUnit should be readable"),
            "<?xml version=\"1.0\"?><testsuites/>\n"
        );
        assert_no_stages(root.path());
    }

    #[test]
    fn preparation_rejects_existing_nonregular_duplicate_alias_and_reserved_paths() {
        let root = TempDir::new().expect("a disposable root should be created");
        let existing = root.path().join("existing.json");
        fs::write(&existing, "unchanged").expect("the existing fixture should be written");
        assert_eq!(
            ReportArtifactDestinations::prepare(Some(existing.clone()), None, &[])
                .expect_err("an existing destination should fail"),
            ReportArtifactError::ExistingOutput
        );

        let directory = root.path().join("directory");
        fs::create_dir(&directory).expect("the directory fixture should be created");
        assert_eq!(
            ReportArtifactDestinations::prepare(Some(directory), None, &[])
                .expect_err("a non-regular destination should fail"),
            ReportArtifactError::NonRegularOutput
        );

        let duplicate = root.path().join("duplicate");
        assert_eq!(
            ReportArtifactDestinations::prepare(Some(duplicate.clone()), Some(duplicate), &[],)
                .expect_err("duplicate destinations should fail"),
            ReportArtifactError::DuplicateDestination
        );

        let subdirectory = root.path().join("subdirectory");
        fs::create_dir(&subdirectory).expect("the alias parent should be created");
        let direct = root.path().join("aliased");
        let indirect = subdirectory.join("..").join("aliased");
        assert_eq!(
            ReportArtifactDestinations::prepare(Some(direct.clone()), Some(indirect), &[])
                .expect_err("resolved aliases should fail"),
            ReportArtifactError::AliasedDestination
        );
        assert_eq!(
            ReportArtifactDestinations::prepare(Some(direct.clone()), None, &[&direct])
                .expect_err("a reserved artifact path should fail"),
            ReportArtifactError::AliasedDestination
        );

        assert_eq!(
            fs::read_to_string(existing).expect("the existing fixture should remain"),
            "unchanged"
        );
        assert_no_stages(root.path());
    }

    #[test]
    fn a_commit_race_and_render_contract_mismatch_roll_back_owned_files() {
        let root = TempDir::new().expect("a disposable root should be created");
        let raced = root.path().join("raced.json");
        let destinations = ReportArtifactDestinations::prepare(Some(raced.clone()), None, &[])
            .expect("the initial destination should prepare");
        fs::write(&raced, "external").expect("the race fixture should be written");
        let failure = destinations
            .persist(vec![artifacts().remove(0)])
            .expect_err("publication must not overwrite a raced destination");
        assert_eq!(failure, ReportArtifactError::Publish);
        assert_eq!(
            fs::read_to_string(&raced).expect("the raced destination should remain"),
            "external"
        );

        let mismatched = root.path().join("mismatched.json");
        let destinations = ReportArtifactDestinations::prepare(Some(mismatched.clone()), None, &[])
            .expect("the mismatch destination should prepare");
        assert_eq!(
            destinations
                .persist(Vec::new())
                .expect_err("missing rendered output should fail"),
            ReportArtifactError::RenderContract
        );
        assert!(!mismatched.exists());
        assert_no_stages(root.path());
    }

    #[test]
    fn a_second_destination_race_rolls_back_the_first_without_touching_the_raced_file() {
        let root = TempDir::new().expect("a disposable root should be created");
        let json = root.path().join("report.json");
        let junit = root.path().join("report.xml");
        let destinations =
            ReportArtifactDestinations::prepare(Some(json.clone()), Some(junit.clone()), &[])
                .expect("both initial destinations should prepare");
        fs::write(&junit, "external").expect("the second destination should be raced");

        assert_eq!(
            destinations
                .persist(artifacts())
                .expect_err("the raced set must not publish partially"),
            ReportArtifactError::Publish
        );
        assert!(!json.exists(), "the first owned output should roll back");
        assert_eq!(
            fs::read_to_string(&junit).expect("the raced destination should remain"),
            "external"
        );
        assert_no_stages(root.path());
    }
}
