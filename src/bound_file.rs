use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum BoundFileErrorKind {
    Unavailable,
    NotRegular,
    IdentityChanged,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct BoundFileError {
    kind: BoundFileErrorKind,
}

impl BoundFileError {
    const fn new(kind: BoundFileErrorKind) -> Self {
        Self { kind }
    }

    pub(crate) const fn kind(self) -> BoundFileErrorKind {
        self.kind
    }
}

impl fmt::Display for BoundFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            BoundFileErrorKind::Unavailable => "selected file is unavailable",
            BoundFileErrorKind::NotRegular => "selected file is not a regular file",
            BoundFileErrorKind::IdentityChanged => "selected file identity changed",
        })
    }
}

impl std::error::Error for BoundFileError {}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum FileIdentity {
    #[cfg(unix)]
    Unix { device: u64, inode: u64 },
    #[cfg(windows)]
    Windows { volume: u64, file_id: [u8; 16] },
    #[cfg(not(any(unix, windows)))]
    Unsupported,
}

impl FileIdentity {
    pub(crate) fn for_file(file: &File) -> io::Result<Self> {
        let metadata = file.metadata()?;
        if !is_regular_non_link(&metadata) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "opened handle is not a regular file",
            ));
        }
        file_identity(file, &metadata)
    }

    pub(crate) fn matches_path(&self, path: &Path) -> io::Result<Option<bool>> {
        let verification = match open_no_follow(path) {
            Ok(file) => file,
            Err(error) => {
                return match fs::symlink_metadata(path) {
                    Err(metadata_error) if metadata_error.kind() == io::ErrorKind::NotFound => {
                        Ok(None)
                    }
                    Ok(metadata) if !is_regular_non_link(&metadata) => Ok(Some(false)),
                    _ => Err(error),
                };
            }
        };
        let metadata = verification.metadata()?;
        if !is_regular_non_link(&metadata) {
            return Ok(Some(false));
        }
        Ok(Some(&file_identity(&verification, &metadata)? == self))
    }
}

#[derive(Debug)]
pub(crate) struct BoundFile {
    file: File,
    metadata: Metadata,
    identity: FileIdentity,
}

impl BoundFile {
    pub(crate) fn open(path: &Path) -> Result<Self, BoundFileError> {
        Self::open_with_hook(path, || internal_test_identity_gate(path))
    }

    fn open_with_hook(
        path: &Path,
        after_initial_open: impl FnOnce() -> io::Result<()>,
    ) -> Result<Self, BoundFileError> {
        let file = open_no_follow(path).map_err(|_| classify_open_failure(path))?;
        let metadata = file
            .metadata()
            .map_err(|_| BoundFileError::new(BoundFileErrorKind::Unavailable))?;
        if !is_regular_non_link(&metadata) {
            return Err(BoundFileError::new(BoundFileErrorKind::NotRegular));
        }
        let identity = file_identity(&file, &metadata)
            .map_err(|_| BoundFileError::new(BoundFileErrorKind::Unavailable))?;

        after_initial_open().map_err(|_| BoundFileError::new(BoundFileErrorKind::Unavailable))?;

        match identity.matches_path(path) {
            Ok(Some(true)) => {}
            Ok(Some(false) | None) => {
                return Err(BoundFileError::new(BoundFileErrorKind::IdentityChanged));
            }
            Err(_) => return Err(BoundFileError::new(BoundFileErrorKind::Unavailable)),
        }

        Ok(Self {
            file,
            metadata,
            identity,
        })
    }

    pub(crate) const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub(crate) const fn identity(&self) -> &FileIdentity {
        &self.identity
    }

    pub(crate) fn into_file(self) -> File {
        self.file
    }
}

fn classify_open_failure(path: &Path) -> BoundFileError {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !is_regular_non_link(&metadata) => {
            BoundFileError::new(BoundFileErrorKind::NotRegular)
        }
        _ => BoundFileError::new(BoundFileErrorKind::Unavailable),
    }
}

#[cfg(not(windows))]
fn is_regular_non_link(metadata: &Metadata) -> bool {
    !metadata.file_type().is_symlink() && metadata.is_file()
}

#[cfg(windows)]
fn is_regular_non_link(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt as _;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.is_file() && metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0
}

#[cfg(unix)]
fn open_no_follow(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
}

#[cfg(windows)]
fn open_no_follow(path: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt as _;

    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

#[cfg(not(any(unix, windows)))]
fn open_no_follow(_path: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "native file identity is unavailable",
    ))
}

#[cfg(unix)]
fn metadata_identity(metadata: &Metadata) -> FileIdentity {
    use std::os::unix::fs::MetadataExt as _;

    FileIdentity::Unix {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

#[cfg(unix)]
fn file_identity(_file: &File, metadata: &Metadata) -> io::Result<FileIdentity> {
    Ok(metadata_identity(metadata))
}

#[cfg(windows)]
fn file_identity(file: &File, _metadata: &Metadata) -> io::Result<FileIdentity> {
    let (volume, file_id) = windows_file_identity::query(file)?;
    Ok(FileIdentity::Windows { volume, file_id })
}

#[cfg(not(any(unix, windows)))]
fn file_identity(_file: &File, _metadata: &Metadata) -> io::Result<FileIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "native file identity is unavailable",
    ))
}

#[cfg(windows)]
mod windows_file_identity {
    use std::ffi::c_void;
    use std::fs::File;
    use std::io;
    use std::mem::{MaybeUninit, size_of};
    use std::os::windows::io::AsRawHandle as _;

    const FILE_ID_INFO_CLASS: i32 = 18;

    #[repr(C)]
    struct FileId128 {
        identifier: [u8; 16],
    }

    #[repr(C)]
    struct FileIdInfo {
        volume_serial_number: u64,
        file_id: FileId128,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandleEx(
            file: *mut c_void,
            information_class: i32,
            information: *mut c_void,
            information_bytes: u32,
        ) -> i32;
    }

    pub(super) fn query(file: &File) -> io::Result<(u64, [u8; 16])> {
        let mut information = MaybeUninit::<FileIdInfo>::uninit();
        let information_bytes = u32::try_from(size_of::<FileIdInfo>())
            .expect("FILE_ID_INFO has a fixed size representable by the Windows API");

        // SAFETY: `file` owns a valid handle for the duration of the call;
        // `information` points to writable, correctly aligned FILE_ID_INFO
        // storage; and the byte count is the exact size of that storage. The
        // value is assumed initialized only after Windows reports success.
        let succeeded = unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FILE_ID_INFO_CLASS,
                information.as_mut_ptr().cast(),
                information_bytes,
            )
        };
        if succeeded == 0 {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: a nonzero return from GetFileInformationByHandleEx promises
        // that the complete FILE_ID_INFO output buffer was initialized.
        let information = unsafe { information.assume_init() };
        Ok((
            information.volume_serial_number,
            information.file_id.identifier,
        ))
    }
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_identity_gate(path: &Path) -> io::Result<()> {
    internal_test_path_gate(
        path,
        "MCP_DOCTOR_INTERNAL_TEST_BOUND_FILE_PATH",
        "MCP_DOCTOR_INTERNAL_TEST_BOUND_FILE_GATE",
    )
}

#[cfg(feature = "internal-test-fixtures")]
pub(crate) fn internal_test_path_gate(
    path: &Path,
    selected_path_variable: &str,
    gate_variable: &str,
) -> io::Result<()> {
    use std::io::{Read as _, Write as _};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    if std::env::var_os("MCP_DOCTOR_TEST_MODE").as_deref() != Some(std::ffi::OsStr::new("1"))
        || std::env::var_os(selected_path_variable).as_deref() != Some(path.as_os_str())
    {
        return Ok(());
    }

    let address = std::env::var(gate_variable)
        .ok()
        .and_then(|value| value.parse::<SocketAddr>().ok())
        .filter(|address| address.ip().is_loopback())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid test gate"))?;
    let watchdog = Duration::from_secs(10);
    let mut stream = TcpStream::connect_timeout(&address, watchdog)?;
    stream.set_read_timeout(Some(watchdog))?;
    stream.set_write_timeout(Some(watchdog))?;
    stream.write_all(&[1])?;
    let mut acknowledgement = [0_u8; 1];
    stream.read_exact(&mut acknowledgement)?;
    if acknowledgement != [2] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid test acknowledgement",
        ));
    }
    Ok(())
}

#[cfg(not(feature = "internal-test-fixtures"))]
fn internal_test_identity_gate(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(not(feature = "internal-test-fixtures"))]
pub(crate) fn internal_test_path_gate(
    _path: &Path,
    _selected_path_variable: &str,
    _gate_variable: &str,
) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BoundFile, BoundFileErrorKind};
    use std::io::Read as _;
    use tempfile::TempDir;

    #[test]
    fn regular_file_reads_only_from_the_verified_handle() {
        let root = TempDir::new().expect("a disposable root should be created");
        let path = root.path().join("input.json");
        std::fs::write(&path, b"verified bytes").expect("the fixture should be written");

        let bound = BoundFile::open(&path).expect("the regular file should be bound");
        assert_eq!(bound.metadata().len(), 14);
        let mut bytes = Vec::new();
        bound
            .into_file()
            .read_to_end(&mut bytes)
            .expect("the bound handle should be readable");
        assert_eq!(bytes, b"verified bytes");
    }

    #[test]
    fn replacement_between_open_and_verification_fails_closed() {
        let root = TempDir::new().expect("a disposable root should be created");
        let path = root.path().join("input.json");
        let retained = root.path().join("retained.json");
        std::fs::write(&path, b"original").expect("the original fixture should be written");

        let error = BoundFile::open_with_hook(&path, || {
            std::fs::rename(&path, &retained)?;
            std::fs::write(&path, b"replacement")
        })
        .expect_err("a replaced path must fail identity verification");

        assert_eq!(error.kind(), BoundFileErrorKind::IdentityChanged);
    }

    #[test]
    fn disappearance_between_open_and_verification_fails_closed() {
        let root = TempDir::new().expect("a disposable root should be created");
        let path = root.path().join("input.json");
        std::fs::write(&path, b"original").expect("the original fixture should be written");

        let error = BoundFile::open_with_hook(&path, || std::fs::remove_file(&path))
            .expect_err("a disappeared path must fail identity verification");

        assert_eq!(error.kind(), BoundFileErrorKind::IdentityChanged);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn symlink_substitution_between_open_and_verification_fails_closed() {
        let root = TempDir::new().expect("a disposable root should be created");
        let path = root.path().join("input.json");
        let retained = root.path().join("retained.json");
        let target = root.path().join("target.json");
        std::fs::write(&path, b"original").expect("the original fixture should be written");
        std::fs::write(&target, b"replacement").expect("the replacement target should be written");

        let error = BoundFile::open_with_hook(&path, || {
            std::fs::rename(&path, &retained)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &path)?;
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&target, &path)?;
            Ok(())
        })
        .expect_err("a symlink substitution must fail identity verification");

        assert_eq!(error.kind(), BoundFileErrorKind::IdentityChanged);
    }

    #[test]
    fn parent_entry_replacement_between_open_and_verification_fails_closed() {
        let root = TempDir::new().expect("a disposable root should be created");
        let selected_parent = root.path().join("selected");
        let retained_parent = root.path().join("retained");
        let replacement_parent = root.path().join("replacement");
        std::fs::create_dir(&selected_parent).expect("the selected parent should be created");
        std::fs::create_dir(&replacement_parent).expect("the replacement parent should be created");
        let path = selected_parent.join("input.json");
        std::fs::write(&path, b"same bytes").expect("the selected fixture should be written");
        std::fs::write(replacement_parent.join("input.json"), b"same bytes")
            .expect("the replacement fixture should be written");

        let error = BoundFile::open_with_hook(&path, || {
            std::fs::rename(&selected_parent, &retained_parent)?;
            std::fs::rename(&replacement_parent, &selected_parent)
        })
        .expect_err("a replaced parent entry must fail identity verification");

        assert_eq!(error.kind(), BoundFileErrorKind::IdentityChanged);
    }

    #[test]
    fn native_identity_matches_hard_links_and_separates_files() {
        let root = TempDir::new().expect("a disposable root should be created");
        let original = root.path().join("original.json");
        let hard_link = root.path().join("hard-link.json");
        let distinct = root.path().join("distinct.json");
        std::fs::write(&original, b"same bytes").expect("the original should be written");
        std::fs::hard_link(&original, &hard_link).expect("the hard link should be created");
        std::fs::write(&distinct, b"same bytes").expect("the distinct file should be written");

        let original = BoundFile::open(&original).expect("the original should open");
        let hard_link = BoundFile::open(&hard_link).expect("the hard link should open");
        let distinct = BoundFile::open(&distinct).expect("the distinct file should open");

        assert_eq!(original.identity(), hard_link.identity());
        assert_ne!(original.identity(), distinct.identity());
    }

    #[cfg(unix)]
    #[test]
    fn final_symlink_is_rejected_without_following_it() {
        use std::os::unix::fs::symlink;

        let root = TempDir::new().expect("a disposable root should be created");
        let target = root.path().join("target.json");
        let link = root.path().join("link.json");
        std::fs::write(&target, b"target").expect("the target should be written");
        symlink(&target, &link).expect("the symlink should be created");

        let error = BoundFile::open(&link).expect_err("a symlink must be rejected");
        assert_eq!(error.kind(), BoundFileErrorKind::NotRegular);
    }
}
