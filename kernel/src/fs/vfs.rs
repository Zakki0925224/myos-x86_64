use super::{fat::Fat, path::Path};
use crate::{
    error::{Error, Result},
    fs::fat::dir_entry::Attribute,
    util::mutex::Mutex,
};
use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

static mut VFS: Mutex<VirtualFileSystem> = Mutex::new(VirtualFileSystem::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FileId(usize);

impl FileId {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileDescriptorNumber(u64);

impl FileDescriptorNumber {
    pub const STDIN: Self = Self(0);
    pub const STDOUT: Self = Self(1);
    pub const STDERR: Self = Self(2);

    pub fn new() -> Self {
        static NEXT_NUM: AtomicU64 = AtomicU64::new(3);
        Self(NEXT_NUM.fetch_add(1, Ordering::Relaxed))
    }

    pub fn new_val(value: i64) -> Result<Self> {
        if value < 0 {
            return Err(Error::Failed("Invalid file descriptor number"));
        }

        Ok(Self(value as u64))
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileDescriptorStatus {
    Open,
    Close,
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    num: FileDescriptorNumber,
    status: FileDescriptorStatus,
    file_id: FileId,
}

#[derive(Debug, PartialEq, Eq)]
enum SpecialFile {
    Device, // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    File,
    Directory,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileSystem {
    Fat(Fat),
}

#[derive(Debug)]
struct FileInfo {
    ty: FileType,
    name: String,
    fs: Option<FileSystem>,
    parent: FileId,
    children: Vec<FileId>,
}

impl FileInfo {
    fn new_directory(name: String, parent: FileId) -> Self {
        Self {
            ty: FileType::Directory,
            name,
            fs: None,
            parent,
            children: Vec::new(),
        }
    }

    fn check_integrity(&self) -> Result<()> {
        if self.ty != FileType::Directory && !self.children.is_empty() {
            return Err(Error::Failed("File must be a directory"));
        }

        if self.fs.is_some() && self.ty != FileType::Directory {
            return Err(Error::Failed("File system mountpoint must be a directory"));
        }

        if self.name.is_empty() {
            return Err(Error::Failed("File name must not be empty"));
        }

        if ["", Path::CURRENT_DIR, Path::PARENT_DIR].contains(&self.name.as_str()) {
            return Err(Error::Failed("File name is invalid"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VirtualFileSystemError {
    NotInitialized,
    NoSuchFileOrDirectoryError(Option<Path>),
    InvalidFileTypeError((FileType, Option<Path>)),
    BlockingFileResourceError(FileDescriptorNumber),
    ReleasedFileResourceError(FileDescriptorNumber),
}

#[derive(Debug)]
struct VirtualFileSystem {
    cwd_id: Option<FileId>,
    root_id: Option<FileId>,
    files: BTreeMap<FileId, FileInfo>,
    fds: Vec<FileDescriptor>,
}

impl VirtualFileSystem {
    const fn new() -> Self {
        Self {
            cwd_id: None,
            root_id: None,
            files: BTreeMap::new(),
            fds: Vec::new(),
        }
    }

    fn insert_file(&mut self, id: FileId, info: FileInfo) -> Result<()> {
        info.check_integrity()?;

        // root
        if id == info.parent {
            self.root_id = Some(id);
            self.cwd_id = Some(id);
        }

        self.files.insert(id, info);

        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        let root_dir_path = Path::root();
        let mnt_dir_path = root_dir_path.join("mnt");
        let dev_dir_path = root_dir_path.join("dev");
        let initramfs_dir_path = mnt_dir_path.join("initramfs");

        let root_id = FileId::new();
        let mnt_id = FileId::new();
        let dev_id = FileId::new();
        let initramfs_id = FileId::new();

        let mut root_dir = FileInfo::new_directory(root_dir_path.name(), root_id);
        let mut mnt_dir = FileInfo::new_directory(mnt_dir_path.name(), root_id);
        let dev_dir = FileInfo::new_directory(dev_dir_path.name(), root_id);
        let initramfs_dir = FileInfo::new_directory(initramfs_dir_path.name(), mnt_id);

        root_dir.children.push(mnt_id);
        root_dir.children.push(dev_id);
        mnt_dir.children.push(initramfs_id);

        self.insert_file(root_id, root_dir)?;
        self.insert_file(mnt_id, mnt_dir)?;
        self.insert_file(dev_id, dev_dir)?;
        self.insert_file(initramfs_id, initramfs_dir)?;

        Ok(())
    }

    fn find_file(&self, id: &FileId) -> Option<&FileInfo> {
        self.files.get(id)
    }

    fn find_file_mut(&mut self, id: &FileId) -> Option<&mut FileInfo> {
        self.files.get_mut(id)
    }

    fn find_file_by_path(&self, path: &Path) -> Option<(FileId, &FileInfo)> {
        let normalized_path = path.normalize();
        let mut file_id = if normalized_path.is_abs() {
            self.root_id
        } else {
            self.cwd_id
        }?;
        let mut file_ref = self.find_file(&file_id)?;

        for name in normalized_path.names() {
            match name {
                Path::CURRENT_DIR => continue,
                Path::PARENT_DIR => {
                    let parent_id = file_ref.parent;
                    file_ref.check_integrity().ok()?;
                    file_ref = self.find_file(&parent_id)?;
                    file_id = parent_id;
                    continue;
                }
                _ => (),
            }

            let mut found = false;
            for child_id in &file_ref.children {
                let child_ref = self.find_file(child_id)?;
                if child_ref.name == name {
                    file_ref = child_ref;
                    file_id = *child_id;
                    found = true;
                    break;
                }
            }

            if !found {
                return None;
            }
        }

        Some((file_id, file_ref))
    }

    fn find_file_by_path_mut(&mut self, path: &Path) -> Option<(FileId, &mut FileInfo)> {
        let (file_id, _) = self.find_file_by_path(path)?;
        let file_ref_mut = self.find_file_mut(&file_id)?;
        Some((file_id, file_ref_mut))
    }

    fn cwd_files(&mut self) -> Vec<&FileInfo> {
        let mut files = Vec::new();
        let cwd_id = if let Some(id) = self.cwd_id {
            id
        } else {
            return files;
        };
        let file_ref = if let Some(file_ref) = self.find_file(&cwd_id) {
            file_ref
        } else {
            return files;
        };

        for child_id in &file_ref.children {
            if let Some(child_ref) = self.find_file(child_id) {
                files.push(child_ref);
            }
        }

        files
    }

    fn chdir(&mut self, path: &Path) -> Result<()> {
        let (file_id, file_ref) = self.find_file_by_path(path).ok_or(
            VirtualFileSystemError::NoSuchFileOrDirectoryError(Some(path.clone())),
        )?;
        if file_ref.ty != FileType::Directory {
            return Err(VirtualFileSystemError::InvalidFileTypeError((
                file_ref.ty,
                Some(path.clone()),
            ))
            .into());
        }

        self.cwd_id = Some(file_id);

        Ok(())
    }

    fn mount_fs(&mut self, path: &Path, fs: FileSystem) -> Result<()> {
        let (mp_file_id, mp_file_ref) = self.find_file_by_path_mut(path).ok_or(
            VirtualFileSystemError::NoSuchFileOrDirectoryError(Some(path.clone())),
        )?;

        if mp_file_ref.ty != FileType::Directory {
            return Err(VirtualFileSystemError::InvalidFileTypeError((
                mp_file_ref.ty,
                Some(path.clone()),
            ))
            .into());
        }

        // mount point's children are removed
        mp_file_ref.children.clear();

        // cache fs
        let cached_files: Vec<(FileId, FileInfo)> = match &fs {
            FileSystem::Fat(fat) => {
                fn cache_recursively(
                    fat: &Fat,
                    dir_cluster_num: Option<usize>,
                    parent_file: (&FileId, &mut FileInfo),
                ) -> Vec<(FileId, FileInfo)> {
                    let (p_file_id, p_file_ref) = parent_file;

                    let mut files = Vec::new();
                    let metadata = fat.scan_dir(dir_cluster_num);
                    for meta in metadata {
                        match meta.name.trim() {
                            "." | ".." => continue,
                            _ => (),
                        }

                        let file_id = FileId::new();
                        let mut file_info = FileInfo {
                            ty: match meta.attr {
                                Attribute::Directory => FileType::Directory,
                                _ => FileType::File,
                            },
                            name: meta.name,
                            fs: None,
                            parent: *p_file_id,
                            children: Vec::new(),
                        };

                        if file_info.ty == FileType::Directory {
                            let child_files = cache_recursively(
                                fat,
                                Some(meta.target_cluster_num),
                                (&file_id, &mut file_info),
                            );
                            files.extend(child_files);
                        }

                        files.push((file_id, file_info));
                        p_file_ref.children.push(file_id);
                    }

                    files
                }

                let files = cache_recursively(fat, None, (&mp_file_id, mp_file_ref));
                files
            }
        };

        mp_file_ref.fs = Some(fs);
        mp_file_ref.check_integrity()?;

        for (id, info) in cached_files {
            self.insert_file(id, info)?;
        }

        Ok(())
    }

    fn find_fs<'a>(&'a self, file_ref: &'a FileInfo) -> Option<(&'a FileSystem, Path)> {
        if let Some(fs) = &file_ref.fs {
            return Some((fs, self.abs_path_by_file(file_ref)?));
        }

        let mut parent_id = file_ref.parent;
        loop {
            let parent_ref = self.find_file(&parent_id)?;
            if let Some(fs) = &parent_ref.fs {
                return Some((fs, self.abs_path_by_file(parent_ref)?));
            }

            parent_id = parent_ref.parent;

            if parent_id == self.root_id? {
                break;
            }
        }

        None
    }

    fn abs_path_by_file(&self, file_ref: &FileInfo) -> Option<Path> {
        let mut s = file_ref.name.clone();

        let mut parent_id = file_ref.parent;
        loop {
            if parent_id == self.root_id? {
                break;
            }

            let parent_ref = self.find_file(&parent_id)?;
            s = format!("{}/{}", parent_ref.name, s);
            parent_id = parent_ref.parent;
        }

        s = format!("{}{}", Path::ROOT, s);
        Some(Path::new(s).normalize())
    }

    fn open_file(&mut self, path: &Path) -> Result<FileDescriptor> {
        let (file_id, file_ref) = self.find_file_by_path(path).ok_or(
            VirtualFileSystemError::NoSuchFileOrDirectoryError(Some(path.clone())),
        )?;
        if file_ref.ty != FileType::File {
            return Err(VirtualFileSystemError::InvalidFileTypeError((
                file_ref.ty,
                Some(path.clone()),
            ))
            .into());
        }

        if let Some(fd) = self.fds.iter().find(|fd| fd.file_id == file_id) {
            return Err(VirtualFileSystemError::BlockingFileResourceError(fd.num).into());
        }

        let fd_num = FileDescriptorNumber::new();
        let fd = FileDescriptor {
            num: fd_num,
            status: FileDescriptorStatus::Open,
            file_id,
        };
        self.fds.push(fd.clone());

        Ok(fd)
    }

    fn close_file(&mut self, fd_num: FileDescriptorNumber) -> Result<()> {
        if let Some(index) = self.fds.iter().position(|f| f.num == fd_num) {
            self.fds.remove(index);
        } else {
            return Err(VirtualFileSystemError::ReleasedFileResourceError(fd_num).into());
        }

        Ok(())
    }

    fn read_file(&self, fd_num: FileDescriptorNumber) -> Result<Vec<u8>> {
        let fd = if let Some(fd) = self
            .fds
            .iter()
            .find(|f| f.num == fd_num && f.status == FileDescriptorStatus::Open)
        {
            fd
        } else {
            return Err(VirtualFileSystemError::ReleasedFileResourceError(fd_num).into());
        };

        let file_ref = self
            .find_file(&fd.file_id)
            .ok_or(VirtualFileSystemError::NoSuchFileOrDirectoryError(None))?;
        let file_path = self
            .abs_path_by_file(file_ref)
            .ok_or(VirtualFileSystemError::NoSuchFileOrDirectoryError(None))?;

        if file_ref.ty != FileType::File {
            return Err(VirtualFileSystemError::InvalidFileTypeError((
                file_ref.ty,
                Some(file_path),
            ))
            .into());
        }

        if let Some((fs, path)) = self.find_fs(file_ref) {
            let diffed_path = path.diff(&file_path);

            match fs {
                FileSystem::Fat(fat) => {
                    let (_, bytes) = fat.get_file_by_abs_path(&diffed_path)?;
                    return Ok(bytes);
                }
            }
        }

        unimplemented!()
    }
}

pub fn init() -> Result<()> {
    let mut vfs = unsafe { VFS.try_lock() }?;
    vfs.init()
}

pub fn chdir(path: &Path) -> Result<()> {
    let mut vfs = unsafe { VFS.try_lock() }?;
    vfs.chdir(path)
}

pub fn mount_fs(path: &Path, fs: FileSystem) -> Result<()> {
    let mut vfs = unsafe { VFS.try_lock() }?;
    vfs.mount_fs(path, fs)
}

pub fn cwd_entry_names() -> Result<Vec<String>> {
    let mut vfs = unsafe { VFS.try_lock() }?;
    let names = vfs.cwd_files().iter().map(|f| f.name.clone()).collect();
    Ok(names)
}

pub fn cwd_path() -> Result<Path> {
    let vfs = unsafe { VFS.try_lock() }?;
    let cwd_id = vfs.cwd_id.ok_or(VirtualFileSystemError::NotInitialized)?;
    let file_ref = vfs
        .find_file(&cwd_id)
        .ok_or(VirtualFileSystemError::NotInitialized)?;
    let path = vfs
        .abs_path_by_file(file_ref)
        .ok_or(VirtualFileSystemError::NotInitialized)?;

    Ok(path)
}

pub fn open_file(path: &Path) -> Result<FileDescriptorNumber> {
    let mut vfs = unsafe { VFS.try_lock() }?;
    let fd = vfs.open_file(path)?;
    Ok(fd.num)
}

// TODO
pub fn close_file(fd_num: &FileDescriptorNumber) -> Result<()> {
    let mut vfs = unsafe { VFS.try_lock() }?;
    vfs.close_file(*fd_num)
}

// TODO
pub fn read_file(fd_num: &FileDescriptorNumber) -> Result<Vec<u8>> {
    let vfs = unsafe { VFS.try_lock() }?;
    vfs.read_file(*fd_num)
}
