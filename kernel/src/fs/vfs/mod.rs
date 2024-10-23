use self::file_desc::{FileDescriptor, FileDescriptorNumber};
use super::initramfs::Initramfs;
use crate::{error::Result, fs::fat::dir_entry::Attribute, util::mutex::Mutex};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::sync::atomic::{AtomicUsize, Ordering};

pub mod file_desc;

const PATH_SEPARATOR: char = '/';

static mut VFS: Mutex<Option<VirtualFileSystem>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileId(usize);

impl FileId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn new_val(value: usize) -> Self {
        Self(value)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SpecialFile {
    StdIn,
    StdOut,
    StdErr,
    DeviceZero,
}

#[derive(Debug, PartialEq, Eq)]
enum FileType {
    File,
    Directory,
    Special(SpecialFile),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileSystem {
    Initramfs(Initramfs),
}

#[derive(Debug)]
pub struct FileInfo {
    pub id: FileId,
    pub ty: FileType,
    pub fs: Option<FileSystem>,
    pub name: String,
    pub parent: Option<FileId>,
    pub child: Option<FileId>,
    pub next: Option<FileId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtualFileSystemError {
    NotInitialized,
    NoSuchFileOrDirectoryError,
    NotDirectoryError,
    NotFileError,
    BlockingFileResourceError(FileDescriptorNumber),
    ReleasedFileResourceError(FileDescriptorNumber),
}

struct VirtualFileSystem {
    cwd_id: FileId,
    root_id: FileId,
    files: Vec<FileInfo>,
    file_descs: Vec<FileDescriptor>,
}

impl VirtualFileSystem {
    fn new() -> Self {
        let mut files = Vec::new();

        let rootfs_id = FileId::new();
        let mnt_dir_id = FileId::new();

        let root_fs = FileInfo {
            id: rootfs_id,
            ty: FileType::Directory,
            fs: None,
            name: String::from("/"),
            parent: Some(rootfs_id),
            child: Some(mnt_dir_id),
            next: None,
        };
        files.push(root_fs);

        let mnt_dir = FileInfo {
            id: mnt_dir_id,
            ty: FileType::Directory,
            fs: None,
            name: String::from("mnt"),
            parent: Some(rootfs_id),
            child: None,
            next: None,
        };
        files.push(mnt_dir);

        Self {
            cwd_id: rootfs_id,
            root_id: rootfs_id,
            files,
            file_descs: Vec::new(),
        }
    }

    fn find_file(&self, id: &FileId) -> Option<&FileInfo> {
        self.files.iter().find(|f| f.id.get() == id.get())
    }

    fn find_file_mut(&mut self, id: &FileId) -> Option<&mut FileInfo> {
        self.files.iter_mut().find(|f| f.id.get() == id.get())
    }

    fn find_file_by_path(&self, path: &str) -> Option<&FileInfo> {
        let mut file_ref = self.find_file(&self.cwd_id)?;

        if path.starts_with(PATH_SEPARATOR) {
            file_ref = self.find_file(&self.root_id)?;
        }

        for name in path.split(PATH_SEPARATOR) {
            match name {
                "" | "." => continue,
                ".." => {
                    if !self.is_directory(file_ref) {
                        return None;
                    }

                    let parent_file_id = file_ref.parent?;
                    file_ref = self.find_file(&parent_file_id)?;
                    continue;
                }
                _ => (),
            }

            if let Some(child_file_id) = file_ref.child {
                file_ref = self.find_file(&child_file_id)?;
            }

            if file_ref.name == name {
                continue;
            }

            let mut found = false;
            while let Some(next_file_id) = file_ref.next {
                file_ref = self.find_file(&next_file_id)?;

                if file_ref.name == name {
                    found = true;
                    break;
                }
            }

            if !found {
                return None;
            }
        }

        Some(file_ref)
    }

    fn find_file_by_path_mut(&mut self, path: &str) -> Option<&mut FileInfo> {
        let file_ref = self.find_file_by_path(path)?;
        let file_ref_id = file_ref.id;
        self.files
            .iter_mut()
            .find(|f| f.id.get() == file_ref_id.get())
    }

    fn chroot(&mut self, path: &str) -> Result<()> {
        let file_ref = match self.find_file_by_path(path) {
            Some(f) => f,
            None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
        };
        if !self.is_directory(file_ref) {
            return Err(VirtualFileSystemError::NotDirectoryError.into());
        }

        self.root_id = file_ref.id;
        Ok(())
    }

    fn chdir(&mut self, path: &str) -> Result<()> {
        let file_ref = match self.find_file_by_path(path) {
            Some(f) => f,
            None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
        };
        if !self.is_directory(file_ref) {
            return Err(VirtualFileSystemError::NotDirectoryError.into());
        }

        self.cwd_id = file_ref.id;
        Ok(())
    }

    fn cwd_files(&mut self) -> Vec<&FileInfo> {
        let mut files = Vec::new();
        let cwd_ref = match self.find_file(&self.cwd_id) {
            Some(f) => f,
            None => return files,
        };

        if let Some(child_file_id) = cwd_ref.child {
            let mut file_ref = match self.find_file(&child_file_id) {
                Some(f) => f,
                None => return files,
            };
            files.push(file_ref);

            while let Some(next_file_id) = file_ref.next {
                file_ref = match self.find_file(&next_file_id) {
                    Some(f) => f,
                    None => return files,
                };
                files.push(file_ref)
            }
        }

        files
    }

    fn cwd_path(&self) -> Result<String> {
        let mut path = String::new();
        let mut file_ref = match self.find_file(&self.cwd_id) {
            Some(f) => f,
            None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
        };
        path = format!("{}", file_ref.name);

        if file_ref.id == self.root_id {
            return Ok(path);
        }

        while let Some(parent_file_id) = file_ref.parent {
            file_ref = match self.find_file(&parent_file_id) {
                Some(f) => f,
                None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
            };

            if file_ref.id == self.root_id {
                path = format!("/{}", path);
                break;
            }

            path = format!("{}{}{}", file_ref.name, PATH_SEPARATOR, path);
        }

        Ok(path)
    }

    fn mount(&mut self, path: &str, fs: FileSystem) -> Result<()> {
        fn map_initramfs(mount_fs: &mut FileInfo) -> Vec<FileInfo> {
            let initramfs_ref = match &mut mount_fs.fs {
                Some(FileSystem::Initramfs(r)) => r,
                _ => unreachable!(),
            };

            let mut files = Vec::new();
            fn scan_recursively(
                initramfs_ref: &mut Initramfs,
                parent_file_id: FileId,
            ) -> Vec<FileInfo> {
                let mut files: Vec<FileInfo> = Vec::new();

                for metadata in initramfs_ref.scan_current_dir() {
                    match metadata.name.trim() {
                        "." | ".." => continue,
                        _ => (),
                    }

                    let mut file_ref = FileInfo {
                        id: FileId::new(),
                        ty: match metadata.attr {
                            Attribute::Directory => FileType::Directory,
                            _ => FileType::File,
                        },
                        fs: None,
                        name: metadata.name,
                        parent: Some(parent_file_id),
                        child: None,
                        next: None,
                    };

                    if let Some(prev_file_ref) = files
                        .iter_mut()
                        .filter(|f| f.parent == Some(parent_file_id) && f.next.is_none())
                        .last()
                    {
                        prev_file_ref.next = Some(file_ref.id);
                    }

                    if file_ref.ty == FileType::Directory {
                        initramfs_ref.cd(&file_ref.name).unwrap();
                        let dir_files = scan_recursively(initramfs_ref, file_ref.id);
                        initramfs_ref.cd("..").unwrap();
                        if let Some(child_file_ref) = dir_files
                            .iter()
                            .filter(|f| f.parent == Some(file_ref.id))
                            .next()
                        {
                            file_ref.child = Some(child_file_ref.id);
                        }

                        files.extend(dir_files);
                    }

                    files.push(file_ref);
                }
                files
            }
            initramfs_ref.reset_cwd();
            files.extend(scan_recursively(initramfs_ref, mount_fs.id));
            initramfs_ref.reset_cwd();

            if let Some(child_file_ref) = files
                .iter()
                .filter(|f| f.parent == Some(mount_fs.id))
                .next()
            {
                mount_fs.child = Some(child_file_ref.id);
            }

            files
        }

        let path_parts: Vec<&str> = path.split(PATH_SEPARATOR).collect();
        let mount_name = match path_parts.last() {
            Some(s) => *s,
            None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
        };

        let mut mount_fs = FileInfo {
            id: FileId::new(),
            ty: FileType::Directory,
            fs: Some(fs),
            name: mount_name.to_string(),
            parent: None,
            child: None,
            next: None,
        };

        let mapped_files = match fs {
            FileSystem::Initramfs(_) => map_initramfs(&mut mount_fs),
        };
        self.files.extend(mapped_files);

        let parent_dir_path = &path[0..path.len() - (mount_name.len() + 1)];
        self.add_file_into_directory(parent_dir_path, &mut mount_fs)?;
        self.files.push(mount_fs);

        Ok(())
    }

    fn open_file(&mut self, path: &str) -> Result<FileDescriptorNumber> {
        let file_ref = match self.find_file_by_path(path) {
            Some(f) => f,
            None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
        };

        if let Some(fd) = self.find_fd_by_file_id(&file_ref.id) {
            return Err(VirtualFileSystemError::BlockingFileResourceError(fd.num).into());
        }

        if file_ref.ty != FileType::File {
            return Err(VirtualFileSystemError::NotFileError.into());
        }

        let fd_num = FileDescriptorNumber::new();
        let fd = FileDescriptor {
            num: fd_num,
            status: file_desc::Status::Open,
            file_id: file_ref.id,
        };
        self.file_descs.push(fd);

        Ok(fd_num)
    }

    fn close_file(&mut self, fd_num: &FileDescriptorNumber) -> Result<()> {
        if self.find_fd(fd_num).is_none() {
            return Err(VirtualFileSystemError::ReleasedFileResourceError(fd_num.clone()).into());
        }

        // remove file descriptor in self.file_descs
        self.file_descs.retain(|fd| fd.num != *fd_num);

        Ok(())
    }

    fn read_file(&mut self, fd_num: &FileDescriptorNumber) -> Result<Vec<u8>> {
        let fd = match self.find_fd(fd_num) {
            Some(f) => f,
            None => {
                return Err(
                    VirtualFileSystemError::ReleasedFileResourceError(fd_num.clone()).into(),
                )
            }
        };

        let file_ref = match self.find_file_mut(&fd.file_id.clone()) {
            Some(f) => f,
            None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
        };

        if file_ref.ty != FileType::File {
            return Err(VirtualFileSystemError::NotFileError.into());
        }

        let mut file_names_to_root_file = Vec::new();
        let mut fs_root_file_ref = file_ref;
        // TODO: if fs not found, loop infinity
        while let Some(parent_file_id) = fs_root_file_ref.parent {
            file_names_to_root_file.push(fs_root_file_ref.name.clone());
            fs_root_file_ref = match self.find_file_mut(&parent_file_id) {
                Some(f) => f,
                None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
            };

            if fs_root_file_ref.fs.is_some() {
                break;
            }
        }
        file_names_to_root_file.reverse();

        match &mut fs_root_file_ref.fs {
            Some(FileSystem::Initramfs(initramfs)) => {
                initramfs.reset_cwd();
                if file_names_to_root_file.len() > 1 {
                    for i in 0..file_names_to_root_file.len() - 1 {
                        initramfs.cd(file_names_to_root_file[i].as_str())?;
                    }
                }

                let (_, bytes) = initramfs.get_file(file_names_to_root_file.last().unwrap())?;
                initramfs.reset_cwd();
                return Ok(bytes);
            }
            None => unreachable!(),
        }
    }

    fn find_fd(&self, num: &FileDescriptorNumber) -> Option<&FileDescriptor> {
        self.file_descs.iter().find(|fd| fd.num == *num)
    }

    fn find_fd_mut(&mut self, num: &FileDescriptorNumber) -> Option<&mut FileDescriptor> {
        self.file_descs.iter_mut().find(|fd| fd.num == *num)
    }

    fn find_fd_by_file_id(&self, file_id: &FileId) -> Option<&FileDescriptor> {
        self.file_descs.iter().find(|fd| fd.file_id == *file_id)
    }

    fn is_directory(&self, file_ref: &FileInfo) -> bool {
        file_ref.ty == FileType::Directory || file_ref.fs.is_some()
    }

    fn add_file_into_directory(
        &mut self,
        parent_dir_path: &str,
        target_file_ref: &mut FileInfo,
    ) -> Result<()> {
        let parent_dir_file_ref = match self.find_file_by_path_mut(parent_dir_path) {
            Some(f) => f,
            None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
        };
        if parent_dir_file_ref.ty != FileType::Directory {
            return Err(VirtualFileSystemError::NotDirectoryError.into());
        }
        target_file_ref.parent = Some(parent_dir_file_ref.id);

        if let Some(child_file_id) = parent_dir_file_ref.child {
            let mut file_ref = match self.find_file_mut(&child_file_id) {
                Some(f) => f,
                None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
            };

            while let Some(next_file_id) = file_ref.next {
                file_ref = match self.find_file_mut(&next_file_id) {
                    Some(f) => f,
                    None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
                };
            }

            file_ref.next = Some(target_file_ref.id);
        } else {
            parent_dir_file_ref.child = Some(target_file_ref.id);
        }

        Ok(())
    }
}

pub fn init() -> Result<()> {
    *unsafe { VFS.try_lock() }? = Some(VirtualFileSystem::new());
    Ok(())
}

pub fn chroot(path: &str) -> Result<()> {
    unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .chroot(path)
}

pub fn chdir(path: &str) -> Result<()> {
    unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .chdir(path)
}

pub fn mount(path: &str, fs: FileSystem) -> Result<()> {
    unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .mount(path, fs)
}

pub fn cwd_entry_names() -> Result<Vec<String>> {
    Ok(unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .cwd_files()
        .iter()
        .map(|f| f.name.clone())
        .collect())
}

pub fn cwd_path() -> Result<String> {
    unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .cwd_path()
}

pub fn open_file(path: &str) -> Result<FileDescriptorNumber> {
    let fd_num = unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .open_file(path)?;
    Ok(fd_num)
}

pub fn close_file(fd_num: &FileDescriptorNumber) -> Result<()> {
    unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .close_file(fd_num)?;
    Ok(())
}

pub fn read_file(fd_num: &FileDescriptorNumber) -> Result<Vec<u8>> {
    unsafe { VFS.try_lock() }?
        .as_mut()
        .ok_or(VirtualFileSystemError::NotInitialized)?
        .read_file(fd_num)
}
