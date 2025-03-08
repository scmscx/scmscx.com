use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use lazy_static::lazy_static;
use scopeguard::defer;
use std::ffi::c_void;
use std::ffi::CString;
use std::fs::remove_file;
use std::fs::File;
use std::io::Write;
use std::mem::size_of;
use std::path::Path;
use std::sync::Mutex;
use stormlib_bindings::SFileCloseArchive;
use stormlib_bindings::SFileCloseFile;
use stormlib_bindings::SFileGetFileInfo;
use stormlib_bindings::SFileGetFileSize;
use stormlib_bindings::SFileOpenFileEx;
use stormlib_bindings::SFileReadFile;
use stormlib_bindings::SFileSetLocale;
use stormlib_bindings::_SFileInfoClass_SFileInfoLocale;
use stormlib_bindings::ERROR_HANDLE_EOF;
use stormlib_bindings::SFILE_INVALID_SIZE;
use stormlib_bindings::STREAM_FLAG_READ_ONLY;
use stormlib_bindings::{GetLastError, SFileOpenArchive, HANDLE};
use tracing::info;
use tracing::{error, instrument};
use uuid::Uuid;

#[instrument(level = "trace", skip_all)]
pub fn get_chk_from_mpq_filename<T: AsRef<Path>>(filename: T) -> Result<Vec<u8>> {
    info!(
        "Extracting scenario.chk. filename: {}",
        filename.as_ref().to_string_lossy()
    );

    lazy_static! {
        // This is really not the rust way to do things but stormlib_bindings is internally not threadsafe so what we can do.
        static ref LOCK: Mutex<()> = Mutex::new(());
    }

    let cstr = CString::new(
        filename
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("Could not convert filename to str"))?,
    )?;

    let _lock = LOCK.lock().unwrap();
    unsafe {
        let mut mpq_handle = 0 as HANDLE;
        if !SFileOpenArchive(
            cstr.as_ptr(),
            0,
            STREAM_FLAG_READ_ONLY,
            &mut mpq_handle as *mut _,
        ) {
            bail!(
                "SFileOpenArchive. GetLastError: {}, filename: {}",
                GetLastError(),
                filename.as_ref().to_string_lossy()
            );
        }

        defer! {
            if !SFileCloseArchive(mpq_handle) {
                error!(
                    "{:?}",
                    anyhow!(
                        "SFileCloseArchive. GetLastError: {}, filename: {}",
                        GetLastError(),
                        filename.as_ref().to_string_lossy()
                    )
                );
            }
        };

        let try_map_with_locale = |filename: &str, locale| {
            let cstr = CString::new(filename)?;

            SFileSetLocale(locale);
            let mut archive_file_handle = 0 as HANDLE;
            if !SFileOpenFileEx(
                mpq_handle,
                cstr.as_ptr(),
                0,
                &mut archive_file_handle as *mut _,
            ) {
                bail!(
                    "SFileOpenFileEx. GetLastError: {}, filename: {filename}, locale: {locale}",
                    GetLastError()
                );
            }

            defer! {
                if !SFileCloseFile(archive_file_handle) {
                    error!(
                        "{:?}",
                        anyhow!(
                            "SFileCloseFile. GetLastError: {}, filename: {filename}, locale: {locale}",
                            GetLastError()
                        )
                    );
                }
            };

            let mut gotten_locale = 0u32;
            if !SFileGetFileInfo(
                archive_file_handle,
                _SFileInfoClass_SFileInfoLocale,
                &mut gotten_locale as *mut _ as *mut c_void,
                size_of::<u32>() as u32,
                0 as *mut _,
            ) {
                bail!(
                    "SFileGetFileInfo. GetLastError: {}, filename: {filename}, locale: {locale}",
                    GetLastError()
                );
            }

            if gotten_locale != locale {
                bail!("not found");
            }

            let file_size_low;
            let mut file_size_high: u32 = 0;

            file_size_low = SFileGetFileSize(archive_file_handle, &mut file_size_high as *mut _);

            if file_size_low == SFILE_INVALID_SIZE {
                bail!(
                    "SFileGetFileSize. GetLastError: {}, filename: {filename}, locale: {locale}",
                    GetLastError()
                );
            }

            if file_size_high != 0 {
                bail!(
                    "SFileGetFileSize. File size too big. file_size_high: {file_size_high}, file_size_low: {file_size_low}",
                );
            }

            let mut chk_data: Vec<u8> = vec![0; file_size_low as usize];

            let mut size: u32 = 0;
            if !SFileReadFile(
                archive_file_handle,
                chk_data.as_mut_ptr() as *mut _,
                chk_data.len() as u32,
                &mut size as *mut _,
                0 as *mut _,
            ) {
                let last_error = GetLastError();
                if last_error != ERROR_HANDLE_EOF || size == chk_data.len() as u32 {
                    bail!(
                        "SFileReadFile. GetLastError: {}, filename: {filename}, locale: {locale}",
                        last_error,
                    );
                }
            }

            chk_data.resize(size as usize, 0);

            Ok(chk_data)
        };

        let locales = [
            0x404, 0x405, 0x407, 0x409, 0x40a, 0x40c, 0x410, 0x411, 0x412, 0x415, 0x416, 0x419,
            0x809, 0,
        ];

        // PROTECTION: Some maps put fake scenario.chk files at different locales. Try to find the real one by trying a lot of them.
        // TODO: Although this algorithm works for the existing test cases it does not feel correct. I suspect that when SC opens a file it just takes the first one it finds.
        // So, in stormlib that would be the one with the lowest index. I won't implement that until doing some more research and confirming that is the case.
        for locale in locales {
            if let Ok(x) = try_map_with_locale("staredit\\scenario.chk", locale) {
                return Ok(x);
            }
        }

        bail!(
            "Couldn't find scenario.chk the legit way: {}, file: {}",
            GetLastError(),
            filename.as_ref().to_string_lossy(),
        );
    }
}

#[instrument(level = "trace", skip_all)]
pub fn get_chk_from_mpq_in_memory(mpq: &[u8]) -> Result<Vec<u8>> {
    // For stormlib to use the right hacks and fixes, it needs to see a file that ends in .scm or .scx
    let path = format!("/tmp/{}.scx", Uuid::new_v4().as_simple().to_string());

    let mut file = File::create(&path)?;

    defer! {
        if let Err(err) = remove_file(&path) {
            error!("{:?}", err);
        }
    }

    file.write_all(mpq)?;

    file.flush()?;

    get_chk_from_mpq_filename(&path)
}
