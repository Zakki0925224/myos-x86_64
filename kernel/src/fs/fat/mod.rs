use self::{
    boot_sector::BootSector, dir_entry::DirectoryEntry, file_allocation_table::ClusterType,
    fs_info_sector::FsInfoSector,
};
use crate::arch::addr::VirtualAddress;
use alloc::vec::Vec;
use core::mem::size_of;

pub mod boot_sector;
pub mod dir_entry;
pub mod file_allocation_table;
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

    pub fn boot_sector(&self) -> &BootSector {
        unsafe { &*(self.volume_start_virt_addr.as_ptr() as *const BootSector) }
    }

    pub fn fs_info_sector(&self) -> Option<&FsInfoSector> {
        match self.fat_type() {
            FatType::Fat32 => {
                let boot_sector = self.boot_sector();
                let fat32_other_field = boot_sector.fat32_other_field().unwrap();
                let fs_info_sector = unsafe {
                    &*(self
                        .volume_start_virt_addr
                        .offset(
                            fat32_other_field.fs_info_sector_num() * boot_sector.bytes_per_sector(),
                        )
                        .as_ptr() as *const FsInfoSector)
                };

                Some(fs_info_sector)
            }
            _ => None,
        }
    }

    pub fn fat_type(&self) -> FatType {
        let boot_sector = self.boot_sector();
        boot_sector.fat_type()
    }

    pub fn root_cluster_num(&self) -> usize {
        match self.fat_type() {
            FatType::Fat12 => unimplemented!(),
            FatType::Fat16 => unimplemented!(),
            FatType::Fat32 => (),
        }

        let boot_sector = self.boot_sector();
        let fat32_other_field = boot_sector.fat32_other_field().unwrap();
        fat32_other_field.root_cluster_num()
    }

    pub fn read_chained_dir_entries(&self, start_cluster_num: usize) -> Vec<DirectoryEntry> {
        let mut entries = Vec::new();
        let mut current_cluster_num = start_cluster_num;
        let mut next_cluster_num = self.next_cluster_num(current_cluster_num);

        loop {
            entries.extend(self.dir_entries(current_cluster_num));

            match next_cluster_num {
                Some(cluster_type) => match &cluster_type {
                    ClusterType::Data(next_cluster_num) => current_cluster_num = *next_cluster_num,
                    _ => break,
                },
                None => break,
            }
            next_cluster_num = self.next_cluster_num(current_cluster_num);
        }

        entries
    }

    fn dir_entries(&self, cluster_num: usize) -> Vec<&DirectoryEntry> {
        let boot_sector = self.boot_sector();
        let mut entries = Vec::with_capacity(self.dir_entries_per_cluster());

        if cluster_num < 2 || cluster_num >= self.clusters_cnt() {
            return entries;
        }

        match self.fat_type() {
            FatType::Fat12 => unimplemented!(),
            FatType::Fat16 => unimplemented!(),
            FatType::Fat32 => (),
        }

        for i in 0..entries.capacity() {
            let offset = boot_sector.data_start_sector32().unwrap()
                * boot_sector.bytes_per_sector()
                + boot_sector.bytes_per_sector()
                    * boot_sector.sectors_per_cluster()
                    * (cluster_num - 2)
                + size_of::<DirectoryEntry>() * i;
            let entry = unsafe {
                &*(self.volume_start_virt_addr.offset(offset).as_ptr() as *const DirectoryEntry)
            };
            entries.push(entry);
        }

        entries
    }

    // read file allocation table
    fn next_cluster_num(&self, cluster_num: usize) -> Option<ClusterType> {
        let boot_sector = self.boot_sector();
        match self.fat_type() {
            FatType::Fat12 => unimplemented!(),
            FatType::Fat16 => unimplemented!(),
            FatType::Fat32 => (),
        }

        let offset = boot_sector.reserved_sectors() * boot_sector.bytes_per_sector()
            + size_of::<u32>() * cluster_num;
        let ref_value =
            unsafe { &*(self.volume_start_virt_addr.offset(offset).as_ptr() as *const _) };
        let value = u32::from_le_bytes(*ref_value) as usize;

        match value {
            0xffffff8.. => Some(ClusterType::EndOfChain),
            0xffffff7.. => Some(ClusterType::Bad(value)),
            0xffffff0.. => Some(ClusterType::Reserved),
            0x2.. => Some(ClusterType::Data(value)),
            0x1 => Some(ClusterType::Reserved),
            0x0 => Some(ClusterType::Free),
            _ => None,
        }
    }

    fn max_dir_entry_num(&self) -> usize {
        let boot_sector = self.boot_sector();
        let data_sectors = match self.fat_type() {
            FatType::Fat12 | FatType::Fat16 => boot_sector.data_sectors16(),
            FatType::Fat32 => boot_sector.data_sectors32().unwrap(),
        };

        data_sectors * boot_sector.bytes_per_sector() / size_of::<DirectoryEntry>()
    }

    fn dir_entries_per_cluster(&self) -> usize {
        let boot_sector = self.boot_sector();
        let cluster_size_bytes = boot_sector.bytes_per_sector() * boot_sector.sectors_per_cluster();
        cluster_size_bytes / size_of::<DirectoryEntry>()
    }

    fn clusters_cnt(&self) -> usize {
        let boot_sector = self.boot_sector();
        boot_sector.data_clusters()
    }

    // pub fn debug(&self) {
    //     let boot_sector = self.read_boot_sector();
    //     println!("{:?}", boot_sector);
    //     println!("fat type: {:?}", boot_sector.fat_type());
    //     println!("oem name: {:?}", boot_sector.oem_name());
    //     println!("data clusters: {}", boot_sector.data_clusters());
    //     println!("bytes per sector: {}", boot_sector.bytes_per_sector());
    //     println!("sectors per cluster: {}", boot_sector.sectors_per_cluster());
    //     println!("fat sectors16: {}", boot_sector.fat_sectors16());
    //     println!("fat sectors32: {:?}", boot_sector.fat_sectors32());
    //     println!("total sectors: {}", boot_sector.total_sectors());
    //     println!("reserved sectors: {}", boot_sector.reserved_sectors());
    //     println!(
    //         "root dir start sector16: {}",
    //         boot_sector.root_dir_start_sector16()
    //     );
    //     println!("root dir sectors16: {}", boot_sector.root_dir_sectors16());
    //     println!("data start sector16: {}", boot_sector.data_start_sector16());
    //     println!("data sectors16: {}", boot_sector.data_sectors16());
    //     println!(
    //         "data start sector32: {:?}",
    //         boot_sector.data_start_sector32()
    //     );
    //     println!("data sectors32: {:?}", boot_sector.data_sectors32());
    //     println!("max dir entry num: {}", self.max_dir_entry_num());

    //     println!("root cluster num: {}", self.root_cluster_num());

    //     for i in 2..self.clusters_cnt() {
    //         let next_cluster_num = self.next_cluster_num(i);
    //         let dir_entries = self.read_dir_entries(i);
    //         println!(
    //             "cluster num: {}, next cluster num: {:?}",
    //             i, next_cluster_num
    //         );

    //         for j in 0..dir_entries.len() {
    //             let dir_entry = dir_entries[j];
    //             println!(
    //                 "\t{}: sfn: {:?}, lfn: {:?} (index: {:?}), attr: {:?}, type: {:?}, fcn: {}, file size: {}",
    //                 j,
    //                 dir_entry.sf_name(),
    //                 dir_entry.lf_name(),
    //                 dir_entry.lfn_entry_index(),
    //                 dir_entry.attr(),
    //                 dir_entry.entry_type(),
    //                 dir_entry.first_cluster_num(),
    //                 dir_entry.file_size()
    //             );
    //         }
    //     }
    // }
}
