use crate::{
    error::{Error, Result},
    println,
};
use alloc::{collections::BTreeMap, vec::Vec};
use common::elf::Elf64;

// https://dwarfstd.org/doc/DWARF5.pdf
// https://qiita.com/mhiramat/items/8df17f5113434e93ff0c

// 7.5.1 Unit Headers
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
enum UnitType {
    Compile = 0x01,
    Type = 0x02,
    Partial = 0x03,
    Skeleton = 0x04,
    SplitCompile = 0x05,
    SplitType = 0x06,
    LowUser = 0x80,
    HighUser = 0xff,
}

impl TryFrom<u8> for UnitType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(Self::Compile),
            0x02 => Ok(Self::Type),
            0x03 => Ok(Self::Partial),
            0x04 => Ok(Self::Skeleton),
            0x05 => Ok(Self::SplitCompile),
            0x06 => Ok(Self::SplitType),
            0x80 => Ok(Self::LowUser),
            0xff => Ok(Self::HighUser),
            _ => Err(Error::Failed("Invalid UnitType")),
        }
    }
}

// 7.5.1.1 Full and Partial Compilation Unit Headers
#[derive(Debug, Clone)]
struct DebugInfo {
    unit_length: u32,
    version: u16,
    unit_type: UnitType,
    address_size: u8,
    debug_abbrev_offset: u32,
    dwo_id: Option<u64>,   // 7.5.1.2 Skeleton and Split Compilation Unit Headers
    type_sig: Option<u64>, // 7.5.1.3 Type Unit Headers
    type_offset: Option<u64>,
    data: Vec<u8>,
}

impl TryFrom<&[u8]> for DebugInfo {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 12 {
            return Err(Error::Failed("Invalid DebugInfo length"));
        }

        let unit_length = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);

        if unit_length == 0xffff_ffff {
            return Err(Error::Failed("64-bit DWARF format is not supported"));
        }

        let version = u16::from_le_bytes([value[4], value[5]]);
        let unit_type = UnitType::try_from(value[6])?;
        let address_size = value[7];
        let debug_abbrev_offset = u32::from_le_bytes([value[8], value[9], value[10], value[11]]);

        let dwo_id = match unit_type {
            UnitType::Skeleton | UnitType::SplitCompile => {
                if value.len() < 20 {
                    return Err(Error::Failed("Invalid DebugInfo length for dwo_id"));
                }
                Some(u64::from_le_bytes([
                    value[12], value[13], value[14], value[15], value[16], value[17], value[18],
                    value[19],
                ]))
            }
            _ => None,
        };

        let type_sig = match unit_type {
            UnitType::Type => {
                if value.len() < 20 {
                    return Err(Error::Failed("Invalid DebugInfo length for type_sig"));
                }
                Some(u64::from_le_bytes([
                    value[12], value[13], value[14], value[15], value[16], value[17], value[18],
                    value[19],
                ]))
            }
            _ => None,
        };

        let type_offset = match unit_type {
            UnitType::Type => {
                if value.len() < 28 {
                    return Err(Error::Failed("Invalid DebugInfo length for type_offset"));
                }
                Some(u64::from_le_bytes([
                    value[20], value[21], value[22], value[23], value[24], value[25], value[26],
                    value[27],
                ]))
            }
            _ => None,
        };

        let mut data_offset = 12;
        match unit_type {
            UnitType::Skeleton | UnitType::SplitCompile => {
                data_offset += 8; // dwo_id
            }
            UnitType::Type => {
                data_offset += 16; // type_sig + type_offset
            }
            _ => (),
        }

        if value.len() < data_offset + unit_length as usize {
            return Err(Error::Failed("DebugInfo data section out of bounds"));
        }

        let data = value[data_offset..(data_offset + (unit_length as usize))].to_vec();

        Ok(DebugInfo {
            unit_length,
            version,
            unit_type,
            address_size,
            debug_abbrev_offset,
            dwo_id,
            type_sig,
            type_offset,
            data,
        })
    }
}

impl DebugInfo {
    fn size(&self) -> usize {
        let mut size = 12; // unit_length + version + unit_type + address_size + debug_abbrev_offset

        match self.unit_type {
            UnitType::Skeleton | UnitType::SplitCompile => {
                size += 8; // dwo_id
            }
            UnitType::Type => {
                size += 16; // type_sig + type_offset
            }
            _ => (),
        }

        size += self.unit_length as usize;
        size
    }
}

#[derive(Debug, Clone)]
struct DebugAbbrev {
    code: u64,
    tag: u64,
    has_children: bool,
    attributes: Vec<(u64, u64)>, // (Attr, Form)
}

fn read_uleb128(slice: &[u8], offset: &mut usize) -> u64 {
    let mut res = 0;
    let mut shift = 0;

    while *offset < slice.len() {
        let byte = slice[*offset];
        *offset += 1;
        res |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    res
}

fn parse_debug_abbrev(
    debug_abbrev_slice: &[u8],
    offset: usize,
) -> Result<BTreeMap<u64, DebugAbbrev>> {
    let mut offset = offset;
    let mut debug_abbrevs = BTreeMap::new();

    while offset < debug_abbrev_slice.len() {
        let code = read_uleb128(debug_abbrev_slice, &mut offset);
        if code == 0 {
            break; // null entry
        }

        let tag = read_uleb128(debug_abbrev_slice, &mut offset);
        let has_children = match debug_abbrev_slice[offset] {
            0 => false,
            1 => true,
            _ => return Err(Error::Failed("Invalid has_children value")),
        };
        offset += 1;

        let mut attributes = Vec::new();
        loop {
            let name = read_uleb128(debug_abbrev_slice, &mut offset);
            let form = read_uleb128(debug_abbrev_slice, &mut offset);

            if name == 0 && form == 0 {
                break; // null entry
            }
            attributes.push((name, form));
        }

        debug_abbrevs.insert(
            code,
            DebugAbbrev {
                code,
                tag,
                has_children,
                attributes,
            },
        );
    }

    Ok(debug_abbrevs)
}

fn parse_debug_info(debug_info_slice: &[u8]) -> Result<Vec<DebugInfo>> {
    let mut debug_infos = Vec::new();
    let mut offset = 0;

    while offset < debug_info_slice.len() {
        let debug_info = DebugInfo::try_from(&debug_info_slice[offset..])?;
        offset += debug_info.size();
        debug_infos.push(debug_info);
        break; // TODO
    }

    Ok(debug_infos)
}

fn parse_die(debug_abbrev_slice: &[u8], debug_info: &DebugInfo) -> Result<()> {
    let debug_abbrev_offset = debug_info.debug_abbrev_offset as usize;
    let debug_abbrevs = parse_debug_abbrev(debug_abbrev_slice, debug_abbrev_offset)?;
    println!("DebugInfo: {:?}", debug_info);
    println!("DebugAbbrev: {:?}", debug_abbrevs);

    let die_data: &[u8] = &debug_info.data;
    let mut offset = 0;
    while offset < die_data.len() {
        let code = read_uleb128(die_data, &mut offset);
        if code == 0 {
            break; // null entry
        }

        let abbrev = debug_abbrevs
            .get(&code)
            .ok_or(Error::Failed("Failed to find abbrev"))?;
        println!("DIE: {:?}", abbrev);
    }

    Ok(())
}

pub fn parse(elf64: &Elf64) -> Result<()> {
    let debug_info_sh = elf64
        .section_header_by_name(".debug_info")
        .ok_or(Error::Failed("Failed to find .debug_info section"))?;

    let debug_info_slice = elf64
        .data_by_section_header(debug_info_sh)
        .ok_or(Error::Failed("Failed to get .debug_info section data"))?;

    let debug_abbrev_sh = elf64
        .section_header_by_name(".debug_abbrev")
        .ok_or(Error::Failed("Failed to find .debug_abbrev section"))?;

    let debug_abbrev_slice = elf64
        .data_by_section_header(debug_abbrev_sh)
        .ok_or(Error::Failed("Failed to get .debug_abbrev section data"))?;

    // parse DIE syntax tree
    let debug_infos = parse_debug_info(debug_info_slice)?;

    for debug_info in &debug_infos {
        parse_die(debug_abbrev_slice, debug_info)?;
    }

    Ok(())
}
