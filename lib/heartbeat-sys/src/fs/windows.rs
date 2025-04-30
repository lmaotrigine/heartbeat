use std::{
    ffi::{c_void, OsStr, OsString},
    fs::{File, OpenOptions},
    io,
    mem::{self, MaybeUninit},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::{MetadataExt, OpenOptionsExt},
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Component, Path, PathBuf},
    ptr,
};

use windows_sys::{
    Wdk::{
        Foundation::OBJECT_ATTRIBUTES,
        Storage::FileSystem::{
            NtCreateFile, RtlInitUnicodeStringEx, FILE_OPEN, FILE_OPEN_REPARSE_POINT, FILE_SYNCHRONOUS_IO_NONALERT,
        },
    },
    Win32::{
        Foundation::{
            DuplicateHandle, RtlNtStatusToDosError, DUPLICATE_SAME_ACCESS, ERROR_INVALID_FUNCTION,
            ERROR_INVALID_PARAMETER, ERROR_NOT_SUPPORTED, ERROR_NO_MORE_FILES, HANDLE, NTSTATUS,
            STATUS_INVALID_PARAMETER, STATUS_SUCCESS, UNICODE_STRING,
        },
        Storage::FileSystem::{
            FileBasicInfo, FileDispositionInfo, FileDispositionInfoEx, FileIdBothDirectoryInfo,
            FileIdBothDirectoryRestartInfo, GetFileInformationByHandleEx, GetFullPathNameW, LockFileEx,
            SetFileInformationByHandle, DELETE, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_READONLY, FILE_BASIC_INFO,
            FILE_DISPOSITION_FLAG_DELETE, FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE,
            FILE_DISPOSITION_FLAG_POSIX_SEMANTICS, FILE_DISPOSITION_INFO, FILE_DISPOSITION_INFO_EX,
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_BOTH_DIR_INFO, FILE_INFO_BY_HANDLE_CLASS,
            FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, SYNCHRONIZE,
        },
        System::{
            SystemServices::UNICODE_STRING_MAX_CHARS,
            Threading::GetCurrentProcess,
            WindowsProgramming::{FILE_CREATED, FILE_OPENED, FILE_OVERWRITTEN},
        },
    },
};

pub(super) fn flock(file: &File) -> io::Result<()> {
    unsafe {
        let mut overlapped = mem::zeroed();
        let ret = LockFileEx(
            file.as_raw_handle() as isize,
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            !0,
            !0,
            &mut overlapped,
        );
        if ret == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

const PATH_SEP: u16 = b'\\' as _;

fn convert_separators(path: &Path) -> io::Result<(Vec<u16>, PathBuf)> {
    let mut wide_path = crate::utils::raw::windows::to_u16s(path)?;
    for ch in &mut wide_path {
        if ch == &b'/'.into() {
            *ch = PATH_SEP;
        }
    }
    let path = OsString::from_wide(&wide_path).into();
    Ok((wide_path, path))
}

fn normalize_verbatim(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().encode_wide().collect::<Vec<_>>();
    for ch in &mut path[..4] {
        if ch == &b'/'.into() {
            *ch = PATH_SEP;
        }
    }
    PathBuf::from(OsString::from_wide(&path))
}

fn normalize_virtually(initial_path: &Path) -> io::Result<PathBuf> {
    let (wide_path, path) = convert_separators(initial_path)?;
    match path.components().next() {
        Some(Component::Prefix(prefix)) if prefix.kind().is_verbatim() => {
            return Ok(normalize_verbatim(initial_path));
        }
        Some(Component::RootDir) if wide_path[1] == PATH_SEP => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "partial UNC prefixes are invalid",
            ));
        }
        _ => {}
    }
    let mut buf = Vec::new();
    let mut capacity = 0;
    loop {
        capacity = unsafe { GetFullPathNameW(wide_path.as_ptr(), capacity, buf.as_mut_ptr(), ptr::null_mut()) };
        if capacity == 0 {
            break Err(io::Error::last_os_error());
        }
        let length = capacity as usize;
        if let Some(mut additional) = length.checked_sub(buf.capacity()) {
            assert_ne!(additional, 0);
            capacity = capacity
                .checked_add(2)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "required path length is too large for WinAPI"))?;
            additional += 2;
            buf.reserve(additional);
            continue;
        }
        unsafe {
            buf.set_len(length);
        }
        break Ok(PathBuf::from(OsString::from_wide(&buf)));
    }
}

pub(super) fn normalize(path: &Path) -> io::Result<PathBuf> {
    path.metadata().and_then(|_| normalize_virtually(path))
}

struct UnicodeString {
    _content: Vec<u16>,
    inner: UNICODE_STRING,
}

#[allow(clippy::cast_possible_wrap)]
fn make_rtl(mut v: Vec<u16>) -> io::Result<UnicodeString> {
    unsafe fn inner(src: &mut [u16], dest: *mut UNICODE_STRING) -> NTSTATUS {
        if src.len() > UNICODE_STRING_MAX_CHARS as usize {
            return STATUS_INVALID_PARAMETER;
        }
        RtlInitUnicodeStringEx(dest, src.as_mut_ptr());
        STATUS_SUCCESS
    }
    let mut unistr = MaybeUninit::uninit();
    let status = unsafe { inner(&mut v, unistr.as_mut_ptr()) };
    if status < 0 {
        return Err(io::Error::from_raw_os_error(unsafe {
            RtlNtStatusToDosError(status) as i32
        }));
    }
    let winapi_str = unsafe { unistr.assume_init() };
    Ok(UnicodeString {
        _content: v,
        inner: winapi_str,
    })
}

#[allow(clippy::cast_possible_truncation)]
pub(super) fn open_path_at(f: &File, p: &Path) -> io::Result<File> {
    let desired_access = SYNCHRONIZE | DELETE | FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES;
    let create_disposition = FILE_OPEN;
    let create_options = FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT;
    let mut handle = MaybeUninit::uninit();
    let mut object_attributes = unsafe { mem::zeroed::<OBJECT_ATTRIBUTES>() };
    object_attributes.Length = mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
    object_attributes.RootDirectory = f.as_raw_handle() as isize;
    let u16_path = crate::utils::raw::windows::to_u16s(p)?;
    let mut rtl_string = make_rtl(u16_path)?;
    object_attributes.ObjectName = &mut rtl_string.inner;
    let mut status_block = MaybeUninit::uninit();
    let file_attributes = FILE_ATTRIBUTE_NORMAL;
    let share_access = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    let status_block = unsafe {
        let status = NtCreateFile(
            handle.as_mut_ptr(),
            desired_access,
            &object_attributes,
            status_block.as_mut_ptr(),
            ptr::null_mut(),
            file_attributes,
            share_access,
            create_disposition,
            create_options,
            ptr::null_mut(),
            0,
        );
        if status < 0 {
            #[allow(clippy::cast_possible_wrap)]
            let err = io::Error::from_raw_os_error(RtlNtStatusToDosError(status) as i32);
            return Err(err);
        }
        status_block.assume_init()
    };
    let information = u32::try_from(status_block.Information).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    match information {
        FILE_CREATED | FILE_OPENED | FILE_OVERWRITTEN => {
            let handle = unsafe { handle.assume_init() };
            Ok(unsafe { File::from_raw_handle(handle as *mut c_void) })
        }
        _ => {
            unimplemented!(
                "expected FILE_CREATED|FILE_OPENED|FILE_OVERWRITTEN, got {}",
                status_block.Information
            );
        }
    }
}

pub(super) fn duplicate_fd(f: &File) -> io::Result<File> {
    let mut new_handle = MaybeUninit::uninit();
    let result = unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            f.as_raw_handle() as HANDLE,
            GetCurrentProcess(),
            new_handle.as_mut_ptr(),
            0,
            i32::from(false),
            DUPLICATE_SAME_ACCESS,
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    let new_handle = unsafe { new_handle.assume_init() };
    Ok(unsafe { File::from_raw_handle(new_handle as *mut c_void) })
}

pub(super) fn open_dir(p: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
    let maybe_dir = options.open(p)?;
    if maybe_dir.metadata()?.is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "path is a directory link, not directory",
        ));
    }
    Ok(maybe_dir)
}

pub(super) struct ReadDir<'a> {
    buffer: Option<Vec<u8>>,
    d: &'a mut File,
    offset: usize,
}

impl<'a> ReadDir<'a> {
    pub(super) fn new(d: &'a mut File) -> io::Result<Self> {
        let mut result = Self {
            buffer: Some(vec![0; 4096]),
            d,
            offset: 0,
        };
        result.fill_buffer(FileIdBothDirectoryRestartInfo)?;
        Ok(result)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn fill_buffer(&mut self, class: FILE_INFO_BY_HANDLE_CLASS) -> io::Result<bool> {
        let buffer = self
            .buffer
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "attempt to fill buffer after end of dir"))?;
        match unsafe {
            GetFileInformationByHandleEx(
                self.d.as_raw_handle() as HANDLE,
                class,
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
            )
        } {
            0 => {
                let err = io::Error::last_os_error();
                if err.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
                    Ok(true)
                } else {
                    Err(err)
                }
            }
            _ => Ok(false),
        }
    }
}

impl Iterator for ReadDir<'_> {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.buffer.as_ref()?.len() {
            match self.fill_buffer(FileIdBothDirectoryInfo) {
                Ok(false) => {
                    self.offset = 0;
                }
                Ok(true) => {
                    self.buffer = None;
                    return None;
                }
                Err(e) => return Some(Err(e)),
            }
        }
        let mem = &self.buffer.as_ref()?[self.offset..];
        #[allow(clippy::cast_ptr_alignment)] // windows is pain, i hope this is alright...
        let info = unsafe { &*mem.as_ptr().cast::<FILE_ID_BOTH_DIR_INFO>() };
        self.offset = if info.NextEntryOffset == 0 {
            self.buffer.as_ref()?.len()
        } else {
            info.NextEntryOffset as usize + self.offset
        };
        let name = OsString::from_wide(unsafe {
            core::slice::from_raw_parts(
                info.FileName.as_ptr(),
                info.FileNameLength as usize / mem::size_of::<u16>(),
            )
        });
        Some(Ok(DirEntry { name }))
    }
}

pub(super) struct DirEntry {
    name: OsString,
}

impl DirEntry {
    pub(super) fn name(&self) -> &OsStr {
        &self.name
    }
}

#[allow(clippy::cast_possible_truncation)]
fn delete_with_posix(f: File) -> Result<File, (File, io::Error)> {
    let mut delete_disposition = FILE_DISPOSITION_INFO_EX {
        Flags: FILE_DISPOSITION_FLAG_DELETE
            | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS
            | FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE,
    };
    match unsafe {
        SetFileInformationByHandle(
            f.as_raw_handle() as HANDLE,
            FileDispositionInfoEx,
            std::ptr::addr_of_mut!(delete_disposition) as *const c_void,
            mem::size_of::<FILE_DISPOSITION_INFO_EX>() as u32,
        )
    } {
        0 => Err((f, io::Error::last_os_error())),
        _ => Ok(f),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn delete_with_win7(f: File) -> Result<File, (File, io::Error)> {
    let mut delete_disposition = FILE_DISPOSITION_INFO { DeleteFile: 1 };
    match unsafe {
        SetFileInformationByHandle(
            f.as_raw_handle() as HANDLE,
            FileDispositionInfo,
            std::ptr::addr_of_mut!(delete_disposition) as *const c_void,
            mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    } {
        0 => Err((f, io::Error::last_os_error())),
        _ => Ok(f),
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn delete_with_win7_readonly(f: File, e: io::Error) -> Result<File, (File, io::Error)> {
    let m = match f.metadata() {
        Ok(m) => m,
        Err(e) => return Err((f, e)),
    };
    if !m.permissions().readonly() {
        return Err((f, e));
    }
    let mut info = FILE_BASIC_INFO {
        FileAttributes: m.file_attributes() & !FILE_ATTRIBUTE_READONLY,
        CreationTime: m.creation_time() as _,
        LastAccessTime: m.last_access_time() as _,
        LastWriteTime: m.last_write_time() as _,
        ChangeTime: 0,
    };
    if unsafe {
        SetFileInformationByHandle(
            f.as_raw_handle() as HANDLE,
            FileBasicInfo,
            std::ptr::addr_of_mut!(info).cast(),
            mem::size_of::<FILE_BASIC_INFO>() as u32,
        )
    } == 0
    {
        return Err((f, io::Error::last_os_error()));
    };
    let f = delete_with_win7(f)?;
    info.FileAttributes |= FILE_ATTRIBUTE_READONLY;
    match unsafe {
        SetFileInformationByHandle(
            f.as_raw_handle() as HANDLE,
            FileBasicInfo,
            std::ptr::addr_of_mut!(info).cast(),
            mem::size_of::<FILE_BASIC_INFO>() as u32,
        )
    } {
        0 => Err((f, io::Error::last_os_error())),
        _ => Ok(f),
    }
}

#[allow(clippy::cast_sign_loss)]
pub(super) fn delete_by_handle(f: File) -> Result<(), (File, io::Error)> {
    match delete_with_posix(f)
        .or_else(|(f, e)| match e.raw_os_error().map(|i| i as u32) {
            Some(ERROR_NOT_SUPPORTED | ERROR_INVALID_PARAMETER | ERROR_INVALID_FUNCTION) => delete_with_win7(f),
            _ => Err((f, e)),
        })
        .or_else(|(f, e)| match e.kind() {
            io::ErrorKind::PermissionDenied => delete_with_win7_readonly(f, e),
            _ => Err((f, e)),
        }) {
        Ok(f) => {
            mem::drop(f);
            Ok(())
        }
        Err((f, e)) => Err((f, e)),
    }
}
