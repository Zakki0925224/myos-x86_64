use alloc::string::String;

use super::FatType;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BootSectorFat32OtherField {
    fat_size32: [u8; 4],
    ext_flags: [u8; 2],
    fs_version: [u8; 2],
    root_cluster: [u8; 4],
    fs_info: [u8; 2],
    backup_boot_sector: [u8; 2],
    reserved0: [u8; 12],
    drive_num: u8,
    reserved1: u8,
    ext_boot_sign: u8,
    volume_id: [u8; 4],
    volume_label: [u8; 11],
    fs_type: [u8; 8],
    boot_code32: [u8; 420],
    boot_sign: [u8; 2],
}

impl BootSectorFat32OtherField {
    pub fn fs_info_sector_num(&self) -> usize {
        return u16::from_le_bytes(self.fs_info) as usize;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BootSectorFat16OtherField {
    drive_num: u8,
    reserved: u8,
    ext_boot_sign: u8,
    volume_id: [u8; 4],
    volume_label: [u8; 11],
    fs_type: [u8; 8],
    boot_code: [u8; 448],
    boot_sign: [u8; 2],
}

#[repr(C)]
pub union BootSectorOtherField {
    pub fat32: BootSectorFat32OtherField,
    pub fat16: BootSectorFat16OtherField,
}

#[repr(C)]
pub struct BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: [u8; 2],
    sectors_per_cluster: u8,
    reserved_sector_count: [u8; 2],
    num_fats: u8,
    root_entry_count: [u8; 2],
    total_sector16: [u8; 2],
    media: u8,
    fat_size16: [u8; 2],
    sectors_per_track: [u8; 2],
    num_heads: [u8; 2],
    hidden_sectors: [u8; 4],
    total_sector32: [u8; 4],
    pub other_field: BootSectorOtherField,
}

impl BootSector {
    pub fn fat_type(&self) -> FatType {
        return match self.num_data_clusters() {
            ..=4085 => FatType::Fat12,
            4086..=65525 => FatType::Fat16,
            _ => FatType::Fat32,
        };
    }

    pub fn oem_name(&self) -> String {
        let buf = String::from_utf8_lossy(&self.oem_name).into_owned();
        return buf;
    }

    pub fn num_data_clusters(&self) -> usize {
        return self.data_sectors() / self.sectors_per_cluster as usize;
    }

    pub fn bytes_per_sector(&self) -> usize {
        return u16::from_le_bytes(self.bytes_per_sector) as usize;
    }

    pub fn sectors_per_cluster(&self) -> usize {
        return self.sectors_per_cluster as usize;
    }

    pub fn fat_sectors(&self) -> usize {
        return self.fat_size16() * self.num_fats as usize;
    }

    pub fn total_sectors(&self) -> usize {
        let total_sector16 = u16::from_le_bytes(self.total_sector16);
        let total_sector32 = u32::from_le_bytes(self.total_sector32);

        if total_sector16 == 0 {
            return total_sector32 as usize;
        }

        return total_sector16 as usize;
    }

    pub fn reserved_sectors(&self) -> usize {
        return u16::from_le_bytes(self.reserved_sector_count) as usize;
    }

    pub fn data_start_sector(&self) -> usize {
        return self.root_dir_start_sector() + self.root_dir_sectors();
    }

    fn fat_size16(&self) -> usize {
        return u16::from_le_bytes(self.fat_size16) as usize;
    }

    fn root_entry_count(&self) -> usize {
        return u16::from_le_bytes(self.root_entry_count) as usize;
    }

    pub fn root_dir_start_sector(&self) -> usize {
        return (((self.reserved_sector_count[1] as u16) << 8)
            | self.reserved_sector_count[0] as u16) as usize
            + self.fat_sectors();
    }

    pub fn root_dir_sectors(&self) -> usize {
        let bytes_per_sector = self.bytes_per_sector();
        return (32 * self.root_entry_count() + bytes_per_sector - 1) / bytes_per_sector;
    }

    fn data_sectors(&self) -> usize {
        return self.total_sectors() - self.data_start_sector();
    }
}
