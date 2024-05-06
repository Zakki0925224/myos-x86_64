use super::initramfs::Initramfs;
use crate::{error::Result, fs::fat::dir_entry::Attribute};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::sync::atomic::{AtomicU64, Ordering};

pub mod file_desc;

const PATH_SEPARATOR: char = '/';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileId(u64);

impl FileId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(3);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn new_val(value: u64) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u64 {
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
    NoSuchFileOrDirectoryError,
    NotDirectoryError,
    NotFileError,
}

pub struct VirtualFileSystem {
    cwd_id: FileId,
    root_id: FileId,
    files: Vec<FileInfo>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        let mut files = Vec::new();

        let rootfs_id = FileId::new();
        let mnt_dir_id = FileId::new();
        let test_file1_id = FileId::new();
        let test_file2_id = FileId::new();
        let test_file3_id = FileId::new();

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
            child: Some(test_file2_id),
            next: Some(test_file1_id),
        };
        files.push(mnt_dir);

        let test_file1 = FileInfo {
            id: test_file1_id,
            ty: FileType::File,
            fs: None,
            name: String::from("test file1"),
            parent: Some(rootfs_id),
            child: None,
            next: None,
        };
        files.push(test_file1);

        let test_file2 = FileInfo {
            id: test_file2_id,
            ty: FileType::File,
            fs: None,
            name: String::from("test file2"),
            parent: Some(mnt_dir_id),
            child: None,
            next: Some(test_file3_id),
        };
        files.push(test_file2);

        let test_file3 = FileInfo {
            id: test_file3_id,
            ty: FileType::File,
            fs: None,
            name: String::from("test file3"),
            parent: Some(mnt_dir_id),
            child: None,
            next: None,
        };
        files.push(test_file3);

        Self {
            cwd_id: rootfs_id,
            root_id: rootfs_id,
            files,
        }
    }

    fn find_file(&self, id: FileId) -> Option<&FileInfo> {
        self.files.iter().find(|f| f.id.get() == id.get())
    }

    fn find_file_mut(&mut self, id: FileId) -> Option<&mut FileInfo> {
        self.files.iter_mut().find(|f| f.id.get() == id.get())
    }

    pub fn find_file_by_path(&self, path: &str) -> Option<&FileInfo> {
        let mut file_ref = self.find_file(self.cwd_id)?;

        if path.starts_with(PATH_SEPARATOR) {
            file_ref = self.find_file(self.root_id)?;
        }

        for name in path.split(PATH_SEPARATOR) {
            match name {
                "" | "." => continue,
                ".." => {
                    if !self.is_directory(file_ref) {
                        return None;
                    }

                    let parent_file_id = file_ref.parent?;
                    file_ref = self.find_file(parent_file_id)?;
                    continue;
                }
                _ => (),
            }

            if let Some(child_file_id) = file_ref.child {
                file_ref = self.find_file(child_file_id)?;
            }

            if file_ref.name == name {
                continue;
            }

            let mut found = false;
            while let Some(next_file_id) = file_ref.next {
                file_ref = self.find_file(next_file_id)?;

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

    pub fn find_file_by_path_mut(&mut self, path: &str) -> Option<&mut FileInfo> {
        let file_ref = self.find_file_by_path(path)?;
        let file_ref_id = file_ref.id;
        self.files
            .iter_mut()
            .find(|f| f.id.get() == file_ref_id.get())
    }

    pub fn chroot(&mut self, path: &str) -> Result<()> {
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

    pub fn chdir(&mut self, path: &str) -> Result<()> {
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

    pub fn cwd_files(&mut self) -> Vec<&FileInfo> {
        let mut files = Vec::new();
        let cwd_ref = match self.find_file(self.cwd_id) {
            Some(f) => f,
            None => return files,
        };

        if let Some(child_file_id) = cwd_ref.child {
            let mut file_ref = match self.find_file(child_file_id) {
                Some(f) => f,
                None => return files,
            };
            files.push(file_ref);

            while let Some(next_file_id) = file_ref.next {
                file_ref = match self.find_file(next_file_id) {
                    Some(f) => f,
                    None => return files,
                };
                files.push(file_ref)
            }
        }

        files
    }

    pub fn mount(&mut self, path: &str, fs: FileSystem) -> Result<()> {
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

    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        let file_ref = match self.find_file_by_path_mut(path) {
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
            fs_root_file_ref = match self.find_file_mut(parent_file_id) {
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
            let mut file_ref = match self.find_file_mut(child_file_id) {
                Some(f) => f,
                None => return Err(VirtualFileSystemError::NoSuchFileOrDirectoryError.into()),
            };

            while let Some(next_file_id) = file_ref.next {
                file_ref = match self.find_file_mut(next_file_id) {
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

pub trait Api {
    fn mount(&mut self, mount_name: &str) -> (FileInfo, Vec<FileInfo>);
}
