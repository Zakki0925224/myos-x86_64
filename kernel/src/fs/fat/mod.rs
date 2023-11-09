use self::{boot_sector::BootSector, dir_entry::DirectoryEntry, fs_info_sector::FsInfoSector};
use crate::{arch::addr::VirtualAddress, println};

pub mod boot_sector;
pub mod dir_entry;
pub mod fs_info_sector;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

pub struct FatVolume {
    volume_start_virt_addr: VirtualAddress,
}

impl FatVolume {
    pub fn new(volume_start_virt_addr: VirtualAddress) -> Self {
        Self {
            volume_start_virt_addr,
        }
    }

    pub fn read_boot_sector(&self) -> BootSector {
        self.volume_start_virt_addr.read_volatile()
    }

    pub fn read_fs_info_sector(&self) -> Option<FsInfoSector> {
        match self.fat_type() {
            FatType::Fat32 => {
                let boot_sector = self.read_boot_sector();
                let fat32_other_field = unsafe { boot_sector.other_field.fat32 };

                Some(
                    self.volume_start_virt_addr
                        .offset(
                            fat32_other_field.fs_info_sector_num() * boot_sector.bytes_per_sector(),
                        )
                        .read_volatile(),
                )
            }
            _ => None,
        }
    }

    pub fn fat_type(&self) -> FatType {
        let boot_sector = self.read_boot_sector();

        boot_sector.fat_type()
    }

    pub fn read_dir_entry(&self, cluster_num: usize) -> Option<DirectoryEntry> {
        let boot_sector = self.read_boot_sector();
        let data_start_sector = boot_sector.data_start_sector();
        let data_clusters = boot_sector.data_clusters();
        let bytes_per_sector = boot_sector.bytes_per_sector();
        let sectors_per_cluster = boot_sector.sectors_per_cluster();

        if cluster_num < 2 {
            return None;
        }

        if cluster_num > data_clusters - 2 {
            return None;
        }

        let start_sector = data_start_sector + (cluster_num - 2) * sectors_per_cluster;
        let offset = start_sector * bytes_per_sector;
        println!("offset: 0x{:x}", offset);

        Some(self.volume_start_virt_addr.offset(offset).read_volatile())
    }

    pub fn read_root_dir_entry(&self) -> DirectoryEntry {
        match self.fat_type() {
            FatType::Fat32 => self.read_dir_entry(0).unwrap(),
            _ => {
                let boot_sector = self.read_boot_sector();
                let root_dir_start_sector = boot_sector.root_dir_start_sector().unwrap();
                let offset = root_dir_start_sector * boot_sector.bytes_per_sector();
                self.volume_start_virt_addr.offset(offset).read_volatile()
            }
        }
    }

    pub fn debug(&self) {
        // for i in 2..10 {
        //     println!("{:?}", self.read_dir_entry(i));
        // }

        let boot_sector = self.read_boot_sector();
        //println!("total sectors: {}", boot_sector.total_sectors());
        //println!("data start sector: {}", boot_sector.data_start_sector());
        //println!("data sectors: {}", boot_sector.data_sectors());
        //println!("data clusters: {}", boot_sector.data_clusters());
        // fat type() is not working (stopped qemu) => NO
        println!(
            "root dir start sector: {:?}",
            boot_sector.root_dir_start_sector()
        );
        // println!("root dir sectors: {:?}", boot_sector.root_dir_sectors());
        // println!("fat start sector: {}", boot_sector.reserved_sectors());
        // println!("fat sectors: {}", boot_sector.fat_sectors());
    }
}
