use std::{
    fs::{self, File, OpenOptions},
    path::Path,
};

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};

/// Open file and forbid "delete" on windows when log file is in use.
pub fn open_file(path: impl AsRef<Path>, truncate: bool) -> crate::Result<File> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(crate::Error::CreateDirectory)?;
        }
    }

    let mut open_options = OpenOptions::new();

    if truncate {
        open_options.write(true).truncate(true);
    } else {
        open_options.append(true);
    }

    #[cfg(windows)]
    {
        // Remove `FILE_SHARE_DELETE` to forbid delete "in use" log file
        // pub const FILE_SHARE_DELETE: DWORD = 0x4;
        // pub const FILE_SHARE_READ: DWORD = 0x1;
        // pub const FILE_SHARE_WRITE: DWORD = 0x2;
        open_options.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE);
    }

    let open_result = open_options.create(true).open(path);

    let f = match open_result {
        Ok(file) => file,
        Err(err) => {
            return Err(crate::Error::OpenFile(err));
        }
    };

    Ok(f)
}
