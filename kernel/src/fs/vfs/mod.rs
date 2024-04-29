use crate::{arch::addr::VirtualAddress, error::Result, println};
use alloc::{string::String, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

pub mod file_desc;

const FILE_ID_STDIN: FileId = FileId::new_val(0);
const FILE_ID_STDOUT: FileId = FileId::new_val(1);
const FILE_ID_STDERR: FileId = FileId::new_val(2);

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, PartialEq, Eq)]
enum FileType {
    File,
    Directory,
    Special(SpecialFile),
}

#[derive(Debug)]
pub struct FileInfo {
    pub id: FileId,
    pub ty: FileType,
    pub name: String,
    pub virt_addr: Option<VirtualAddress>,
    pub len: Option<usize>,
    pub parent: Option<FileId>,
    pub child: Option<FileId>,
    pub next: Option<FileId>,
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
        let test_dir1_id = FileId::new();
        let test_file1_id = FileId::new();
        let test_file2_id = FileId::new();
        let test_file3_id = FileId::new();

        let root_fs = FileInfo {
            id: rootfs_id,
            ty: FileType::Directory,
            name: String::from("/"),
            virt_addr: None,
            len: None,
            parent: Some(rootfs_id),
            child: Some(test_dir1_id),
            next: None,
        };
        files.push(root_fs);

        let test_dir1 = FileInfo {
            id: test_dir1_id,
            ty: FileType::Directory,
            name: String::from("test dir1"),
            virt_addr: None,
            len: None,
            parent: Some(rootfs_id),
            child: Some(test_file2_id),
            next: Some(test_file1_id),
        };
        files.push(test_dir1);

        let test_file1 = FileInfo {
            id: test_file1_id,
            ty: FileType::File,
            name: String::from("test file1"),
            virt_addr: None,
            len: None,
            parent: Some(rootfs_id),
            child: None,
            next: None,
        };
        files.push(test_file1);

        let test_file2 = FileInfo {
            id: test_file2_id,
            ty: FileType::File,
            name: String::from("test file2"),
            virt_addr: None,
            len: None,
            parent: Some(test_dir1_id),
            child: None,
            next: Some(test_file3_id),
        };
        files.push(test_file2);

        let test_file3 = FileInfo {
            id: test_file3_id,
            ty: FileType::File,
            name: String::from("test file3"),
            virt_addr: None,
            len: None,
            parent: Some(test_dir1_id),
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

    pub fn add_file(
        &mut self,
        file_type: FileType,
        file_path: String,
        virt_addr: Option<VirtualAddress>,
        len: Option<usize>,
    ) -> Result<()> {
        Ok(())
    }

    fn find_file(&self, id: FileId) -> Option<&FileInfo> {
        self.files.iter().find(|f| f.id.get() == id.get())
    }

    pub fn find_file_by_path(&self, path: &str) -> Option<&FileInfo> {
        let mut file_ref = self.find_file(self.cwd_id)?;

        if path.starts_with("/") {
            file_ref = self.find_file(self.root_id)?;
        }

        for name in path.split("/") {
            println!("name: {:?}, ref: {:?}", name, file_ref);

            match name {
                "" | "." => continue,
                ".." => {
                    if file_ref.ty == FileType::File {
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
                let next_file_ref = self.find_file(next_file_id)?;
                file_ref = next_file_ref;

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
}
