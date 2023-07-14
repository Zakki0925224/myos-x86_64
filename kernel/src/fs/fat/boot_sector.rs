use alloc::{string::String, vec::Vec};

#[derive(Debug)]
#[repr(C)]
pub enum BootSectorOtherField {
    Fat32 {
        fat_size32: u32,
        ext_flags: u16,
        fs_version: [u8; 2],
        root_cluster: u32,
        fs_info: u16,
        backup_boot_sector: u16,
        reserved0: [u8; 12],
        drive_num: u8,
        reserved1: u8,
        ext_boot_sign: u8,
        volume_id: u32,
        volume_label: [u8; 11],
        fs_type: [u8; 8],
        boot_code32: [u8; 420],
        boot_sign: [u8; 2],
    },
    Fat16 {
        drive_num: u8,
        reserved: u8,
        ext_boot_sign: u8,
        volume_id: u32,
        volume_label: [u8; 11],
        fs_type: [u8; 8],
        boot_code: [u8; 448],
        boot_sign: [u8; 2],
    },
}

#[derive(Debug)]
#[repr(C)]
pub struct BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    num_fats: u8,
    root_entry_count: u16,
    total_sector16: u16,
    media: u8,
    fat_size16: u16,
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sector32: u32,
    other_field: BootSectorOtherField,
}

impl BootSector {
    pub fn oem_name(&self) -> String {
        let buf = String::from_utf8_lossy(&self.oem_name).into_owned();
        return buf;
    }
}
