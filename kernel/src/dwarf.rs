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
enum UnitType {
    Compile,
    Type,
    Partial,
    Skeleton,
    SplitCompile,
    SplitType,
    LowUser,
    HighUser,
    Invalid(u8),
}

impl From<u8> for UnitType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Compile,
            0x02 => Self::Type,
            0x03 => Self::Partial,
            0x04 => Self::Skeleton,
            0x05 => Self::SplitCompile,
            0x06 => Self::SplitType,
            0x80 => Self::LowUser,
            0xff => Self::HighUser,
            _ => Self::Invalid(value),
        }
    }
}

// 7.5.1.1 Full and Partial Compilation Unit Headers
#[derive(Clone)]
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

impl core::fmt::Debug for DebugInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DebugInfo")
            .field("unit_length", &self.unit_length)
            .field("version", &self.version)
            .field("unit_type", &self.unit_type)
            .field("address_size", &self.address_size)
            .field("debug_abbrev_offset", &self.debug_abbrev_offset)
            .field("dwo_id", &self.dwo_id)
            .field("type_sig", &self.type_sig)
            .field("type_offset", &self.type_offset)
            .finish()
    }
}

impl TryFrom<&[u8]> for DebugInfo {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 4 {
            // Need at least unit_length
            return Err(Error::Failed("Invalid DebugInfo length (unit_length)"));
        }

        let unit_length = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);

        if unit_length == 0xffff_ffff {
            return Err(Error::Failed("64-bit DWARF format is not supported"));
        }

        let total_unit_size = 4 + unit_length as usize;
        if value.len() < total_unit_size {
            return Err(Error::Failed(
                "DebugInfo data section out of bounds (unit_length mismatch)",
            ));
        }

        let minimum_header_size = 12;
        if value.len() < minimum_header_size {
            return Err(Error::Failed("Invalid DebugInfo length (header minimum)"));
        }

        let version = u16::from_le_bytes([value[4], value[5]]);
        let unit_type = UnitType::from(value[6]);
        let address_size = value[7];
        let debug_abbrev_offset = u32::from_le_bytes([value[8], value[9], value[10], value[11]]);

        let mut header_size_after_length = 8;

        let dwo_id = match unit_type {
            UnitType::Skeleton | UnitType::SplitCompile => {
                if value.len() < 4 + header_size_after_length + 8 {
                    return Err(Error::Failed("Invalid DebugInfo length for dwo_id"));
                }
                let dwo_offset = 4 + header_size_after_length;
                header_size_after_length += 8;
                Some(u64::from_le_bytes([
                    value[dwo_offset],
                    value[dwo_offset + 1],
                    value[dwo_offset + 2],
                    value[dwo_offset + 3],
                    value[dwo_offset + 4],
                    value[dwo_offset + 5],
                    value[dwo_offset + 6],
                    value[dwo_offset + 7],
                ]))
            }
            _ => None,
        };

        let type_sig = match unit_type {
            UnitType::Type => {
                if value.len() < 4 + header_size_after_length + 8 {
                    return Err(Error::Failed("Invalid DebugInfo length for type_sig"));
                }
                let sig_offset = 4 + header_size_after_length;
                header_size_after_length += 8;
                Some(u64::from_le_bytes([
                    value[sig_offset],
                    value[sig_offset + 1],
                    value[sig_offset + 2],
                    value[sig_offset + 3],
                    value[sig_offset + 4],
                    value[sig_offset + 5],
                    value[sig_offset + 6],
                    value[sig_offset + 7],
                ]))
            }
            _ => None,
        };

        let type_offset = match unit_type {
            UnitType::Type => {
                if value.len() < 4 + header_size_after_length + 8 {
                    return Err(Error::Failed("Invalid DebugInfo length for type_offset"));
                }
                let offset_offset = 4 + header_size_after_length;
                header_size_after_length += 8;
                Some(u64::from_le_bytes([
                    value[offset_offset],
                    value[offset_offset + 1],
                    value[offset_offset + 2],
                    value[offset_offset + 3],
                    value[offset_offset + 4],
                    value[offset_offset + 5],
                    value[offset_offset + 6],
                    value[offset_offset + 7],
                ]))
            }
            _ => None,
        };

        let data_offset = 4 + header_size_after_length;

        if total_unit_size < data_offset {
            return Err(Error::Failed(
                "Calculated data offset exceeds total unit size",
            ));
        }
        let data = value[data_offset..total_unit_size].to_vec();

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
        4 + self.unit_length as usize
    }
}

// 7.5.3 Abbreviations Tables
#[derive(Debug, Clone, PartialEq, Eq)]
enum AbbrevTag {
    ArrayType,
    ClassType,
    EntryPoint,
    EnumerationType,
    FormalParameter,
    ImportedDeclaration,
    Label,
    LexicalBlock,
    Member,
    PointerType,
    ReferenceType,
    CompileUnit,
    StringType,
    StructureType,
    SubroutineType,
    Typedef,
    UnionType,
    UnspecifiedParameters,
    Variant,
    CommonBlock,
    CommonInclusion,
    Inheritance,
    InlinedSubroutine,
    Module,
    PtrToMemberType,
    SetType,
    SubrangeType,
    WithStmt,
    AccessDeclaration,
    BaseType,
    CatchBlock,
    ConstType,
    Constant,
    Enumerator,
    FileType,
    Friend,
    Namelist,
    NamelistItem,
    PackedType,
    Subprogram,
    TemplateTypeParameter,
    TemplateValueParameter,
    ThrownType,
    TryBlock,
    VariantPart,
    Variable,
    VolatileType,
    DwarfProcedure,
    RestrictType,
    InterfaceType,
    Namespace,
    ImportedModule,
    UnspecifiedType,
    PartialUnit,
    ImportedUnit,
    Condition,
    SharedType,
    TypeUnit,
    RvalueReferenceType,
    TemplateAlias,
    CoarrayType,
    GenericSubrange,
    DynamicType,
    AtomicType,
    CallSite,
    CallSiteParameter,
    SkeletonUnit,
    ImmutableType,
    LoUser,
    HiUser,
    Invalid(u64),
}

impl From<u64> for AbbrevTag {
    fn from(value: u64) -> Self {
        match value {
            0x01 => Self::ArrayType,
            0x02 => Self::ClassType,
            0x03 => Self::EntryPoint,
            0x04 => Self::EnumerationType,
            0x05 => Self::FormalParameter,
            0x08 => Self::ImportedDeclaration,
            0x0a => Self::Label,
            0x0b => Self::LexicalBlock,
            0x0d => Self::Member,
            0x0f => Self::PointerType,
            0x10 => Self::ReferenceType,
            0x11 => Self::CompileUnit,
            0x12 => Self::StringType,
            0x13 => Self::StructureType,

            0x15 => Self::SubroutineType,
            0x16 => Self::Typedef,
            0x17 => Self::UnionType,
            0x18 => Self::UnspecifiedParameters,
            0x19 => Self::Variant,
            0x1a => Self::CommonBlock,
            0x1b => Self::CommonInclusion,
            0x1c => Self::Inheritance,
            0x1d => Self::InlinedSubroutine,
            0x1e => Self::Module,
            0x1f => Self::PtrToMemberType,
            0x20 => Self::SetType,
            0x21 => Self::SubrangeType,
            0x22 => Self::WithStmt,
            0x23 => Self::AccessDeclaration,
            0x24 => Self::BaseType,
            0x25 => Self::CatchBlock,
            0x26 => Self::ConstType,
            0x27 => Self::Constant,
            0x28 => Self::Enumerator,
            0x29 => Self::FileType,
            0x2a => Self::Friend,
            0x2b => Self::Namelist,
            0x2c => Self::NamelistItem,
            0x2d => Self::PackedType,
            0x2e => Self::Subprogram,
            0x2f => Self::TemplateTypeParameter,
            0x30 => Self::TemplateValueParameter,
            0x31 => Self::ThrownType,
            0x32 => Self::TryBlock,
            0x33 => Self::VariantPart,
            0x34 => Self::Variable,
            0x35 => Self::VolatileType,
            0x36 => Self::DwarfProcedure,
            0x37 => Self::RestrictType,
            0x38 => Self::InterfaceType,
            0x39 => Self::Namespace,
            0x3a => Self::ImportedModule,
            0x3b => Self::UnspecifiedType,
            0x3c => Self::PartialUnit,
            0x3d => Self::ImportedUnit,
            0x3f => Self::Condition,
            0x40 => Self::SharedType,
            0x41 => Self::TypeUnit,
            0x42 => Self::RvalueReferenceType,
            0x43 => Self::TemplateAlias,
            0x44 => Self::CoarrayType,
            0x45 => Self::GenericSubrange,
            0x46 => Self::DynamicType,
            0x47 => Self::AtomicType,
            0x48 => Self::CallSite,
            0x49 => Self::CallSiteParameter,
            0x4a => Self::SkeletonUnit,
            0x4b => Self::ImmutableType,
            0x4090 => Self::LoUser,
            0xffff => Self::HiUser,
            _ => Self::Invalid(value),
        }
    }
}

// 7.5.4 Attribute Encodings
#[derive(Debug, Clone, PartialEq, Eq)]
enum AbbrevAttribute {
    Sibling,
    Location,
    Name,
    Ordering,
    ByteSize,
    BitSize,
    StmtList,
    LowPc,
    HighPc,
    Language,
    Discr,
    DiscrValue,
    Visibility,
    Import,
    StringLength,
    CommonReference,
    CompDir,
    ConstValue,
    ContainingType,
    DefaultValue,
    Inline,
    IsOptional,
    LowerBound,
    Producer,
    Prototyped,
    ReturnAddr,
    StartScope,
    BitStride,
    UpperBound,
    AbstractOrigin,
    Accessibility,
    AddressClass,
    Artificial,
    BaseTypes,
    CallingConvention,
    Count,
    DataMemberLocation,
    DeclColumn,
    DeclFile,
    DeclLine,
    Declaration,
    DiscrList,
    Encoding,
    External,
    FrameBase,
    Friend,
    IdentifierCase,
    NameListItem,
    Priority,
    Segment,
    Specification,
    StaticLink,
    Type,
    UseLocation,
    VariableParameter,
    Virtuality,
    VtableElemLocation,
    Allocated,
    Associated,
    DataLocation,
    ByteStride,
    EntryPc,
    UseUtf8,
    Extension,
    Ranges,
    Trampoline,
    CallColumn,
    CallFile,
    CallLine,
    Description,
    BinaryScale,
    DecimalScale,
    Small,
    DecimalSign,
    DigitCount,
    PictureString,
    Mutable,
    ThreadsScaled,
    Explicit,
    ObjectPointer,
    Endianity,
    Elemental,
    Pure,
    Recursive,
    Signature,
    MainSubprogram,
    DataBitOffset,
    ConstExpr,
    EnumClass,
    LinkageName,
    StringLengthBitSize,
    StringLengthByteSize,
    Rank,
    StrOffsetsBase,
    AddrBase,
    RnglistsBase,
    DwoName,
    Reference,
    RvalueReference,
    Macros,
    CallAllCalls,
    CallAllSourceCalls,
    CallAllTailCalls,
    CallReturnPc,
    CallValue,
    CallOrigin,
    CallParameter,
    CallPc,
    CallTailCall,
    CallTarget,
    CallTargetClobbered,
    CallDataLocation,
    CallDataValue,
    Noreturn,
    Alignment,
    ExportSymbols,
    Deleted,
    Defaulted,
    LoclistsBase,
    LoUser,
    HiUser,
    Invalid(u64),
}

impl From<u64> for AbbrevAttribute {
    fn from(value: u64) -> Self {
        match value {
            0x01 => Self::Sibling,
            0x02 => Self::Location,
            0x03 => Self::Name,
            0x09 => Self::Ordering,
            0x0b => Self::ByteSize,
            0x0d => Self::BitSize,
            0x10 => Self::StmtList,
            0x11 => Self::LowPc,
            0x12 => Self::HighPc,
            0x13 => Self::Language,
            0x15 => Self::Discr,
            0x16 => Self::DiscrValue,
            0x17 => Self::Visibility,
            0x18 => Self::Import,
            0x19 => Self::StringLength,
            0x1a => Self::CommonReference,
            0x1b => Self::CompDir,
            0x1c => Self::ConstValue,
            0x1d => Self::ContainingType,
            0x1e => Self::DefaultValue,
            0x20 => Self::Inline,
            0x21 => Self::IsOptional,
            0x22 => Self::LowerBound,
            0x25 => Self::Producer,
            0x27 => Self::Prototyped,
            0x2a => Self::ReturnAddr,
            0x2c => Self::StartScope,
            0x2e => Self::BitStride,
            0x2f => Self::UpperBound,
            0x31 => Self::AbstractOrigin,
            0x32 => Self::Accessibility,
            0x33 => Self::AddressClass,
            0x34 => Self::Artificial,
            0x35 => Self::BaseTypes,
            0x36 => Self::CallingConvention,
            0x37 => Self::Count,
            0x38 => Self::DataMemberLocation,
            0x39 => Self::DeclColumn,
            0x3a => Self::DeclFile,
            0x3b => Self::DeclLine,
            0x3c => Self::Declaration,
            0x3d => Self::DiscrList,
            0x3e => Self::Encoding,
            0x3f => Self::External,
            0x40 => Self::FrameBase,
            0x41 => Self::Friend,
            0x42 => Self::IdentifierCase,
            0x44 => Self::NameListItem,
            0x45 => Self::Priority,
            0x46 => Self::Segment,
            0x47 => Self::Specification,
            0x48 => Self::StaticLink,
            0x49 => Self::Type,
            0x4a => Self::UseLocation,
            0x4b => Self::VariableParameter,
            0x4c => Self::Virtuality,
            0x4d => Self::VtableElemLocation,
            0x4e => Self::Allocated,
            0x4f => Self::Associated,
            0x50 => Self::DataLocation,
            0x51 => Self::ByteStride,
            0x52 => Self::EntryPc,
            0x53 => Self::UseUtf8,
            0x54 => Self::Extension,
            0x55 => Self::Ranges,
            0x56 => Self::Trampoline,
            0x57 => Self::CallColumn,
            0x58 => Self::CallFile,
            0x59 => Self::CallLine,
            0x5a => Self::Description,
            0x5b => Self::BinaryScale,
            0x5c => Self::DecimalScale,
            0x5d => Self::Small,
            0x5e => Self::DecimalSign,
            0x5f => Self::DigitCount,
            0x60 => Self::PictureString,
            0x61 => Self::Mutable,
            0x62 => Self::ThreadsScaled,
            0x63 => Self::Explicit,
            0x64 => Self::ObjectPointer,
            0x65 => Self::Endianity,
            0x66 => Self::Elemental,
            0x67 => Self::Pure,
            0x68 => Self::Recursive,
            0x69 => Self::Signature,
            0x6a => Self::MainSubprogram,
            0x6b => Self::DataBitOffset,
            0x6c => Self::ConstExpr,
            0x6d => Self::EnumClass,
            0x6e => Self::LinkageName,
            0x6f => Self::StringLengthBitSize,
            0x70 => Self::StringLengthByteSize,
            0x71 => Self::Rank,
            0x72 => Self::StrOffsetsBase,
            0x73 => Self::AddrBase,
            0x74 => Self::RnglistsBase,
            0x76 => Self::DwoName,
            0x77 => Self::Reference,
            0x78 => Self::RvalueReference,
            0x79 => Self::Macros,
            0x7a => Self::CallAllCalls,
            0x7b => Self::CallAllSourceCalls,
            0x7c => Self::CallAllTailCalls,
            0x7d => Self::CallReturnPc,
            0x7e => Self::CallValue,
            0x7f => Self::CallOrigin,
            0x80 => Self::CallParameter,
            0x81 => Self::CallPc,
            0x82 => Self::CallTailCall,
            0x83 => Self::CallTarget,
            0x84 => Self::CallTargetClobbered,
            0x85 => Self::CallDataLocation,
            0x86 => Self::CallDataValue,
            0x87 => Self::Noreturn,
            0x88 => Self::Alignment,
            0x89 => Self::ExportSymbols,
            0x8a => Self::Deleted,
            0x8b => Self::Defaulted,
            0x8c => Self::LoclistsBase,
            0x2000 => Self::LoUser,
            0x3fff => Self::HiUser,
            _ => Self::Invalid(value),
        }
    }
}

// 7.5.6 Form Encodings
#[derive(Debug, Clone, PartialEq, Eq)]
enum AbbrevForm {
    Addr,
    Block2,
    Block4,
    Data2,
    Data4,
    Data8,
    String,
    Block,
    Block1,
    Data1,
    Flag,
    Sdata,
    Strp,
    Udata,
    RefAddr,
    Ref1,
    Ref2,
    Ref4,
    Ref8,
    RefUdata,
    Indirect,
    SecOffset,
    Exprloc,
    FlagPresent,
    Strx,
    Addrx,
    RefSup4,
    StrpSup,
    Data16,
    LineStrp,
    RefSig8,
    ImplicitConst,
    Loclistx,
    Rnglistx,
    RefSup8,
    Strx1,
    Strx2,
    Strx3,
    Strx4,
    Addrx1,
    Addrx2,
    Addrx3,
    Addrx4,
    Invalid(u64),
}

impl From<u64> for AbbrevForm {
    fn from(value: u64) -> Self {
        match value {
            0x01 => Self::Addr,
            0x03 => Self::Block2,
            0x04 => Self::Block4,
            0x05 => Self::Data2,
            0x06 => Self::Data4,
            0x07 => Self::Data8,
            0x08 => Self::String,
            0x09 => Self::Block,
            0x0a => Self::Block1,
            0x0b => Self::Data1,
            0x0c => Self::Flag,
            0x0d => Self::Sdata,
            0x0e => Self::Strp,
            0x0f => Self::Udata,
            0x10 => Self::RefAddr,
            0x11 => Self::Ref1,
            0x12 => Self::Ref2,
            0x13 => Self::Ref4,
            0x14 => Self::Ref8,
            0x15 => Self::RefUdata,
            0x16 => Self::Indirect,
            0x17 => Self::SecOffset,
            0x18 => Self::Exprloc,
            0x19 => Self::FlagPresent,
            0x1a => Self::Strx,
            0x1b => Self::Addrx,
            0x1c => Self::RefSup4,
            0x1d => Self::StrpSup,
            0x1e => Self::Data16,
            0x1f => Self::LineStrp,
            0x20 => Self::RefSig8,
            0x21 => Self::ImplicitConst,
            0x22 => Self::Loclistx,
            0x23 => Self::Rnglistx,
            0x24 => Self::RefSup8,
            0x25 => Self::Strx1,
            0x26 => Self::Strx2,
            0x27 => Self::Strx3,
            0x28 => Self::Strx4,
            0x29 => Self::Addrx1,
            0x2a => Self::Addrx2,
            0x2b => Self::Addrx3,
            0x2c => Self::Addrx4,
            _ => Self::Invalid(value),
        }
    }
}

#[derive(Debug, Clone)]
struct DebugAbbrev {
    code: u64,
    tag: AbbrevTag,
    has_children: bool,
    attributes: Vec<(AbbrevAttribute, AbbrevForm)>,
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

        let tag = read_uleb128(debug_abbrev_slice, &mut offset).into();
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

            attributes.push((name.into(), form.into()));
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
        let debug_info = match DebugInfo::try_from(&debug_info_slice[offset..]) {
            Ok(debug_info) => debug_info,
            Err(_) => break,
        };
        offset += debug_info.size();
        debug_infos.push(debug_info);
    }

    Ok(debug_infos)
}

fn parse_die(debug_abbrev_slice: &[u8], debug_info: &DebugInfo) -> Result<()> {
    println!("DebugInfo: {:?}", debug_info);

    let debug_abbrev_offset = debug_info.debug_abbrev_offset as usize;
    let debug_abbrevs = parse_debug_abbrev(debug_abbrev_slice, debug_abbrev_offset)?;

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
        println!("Abbrev: {} - {:?}", abbrev.code, abbrev.tag);

        for (attr, form) in &abbrev.attributes {
            println!("  Attribute: {:?}, Form: {:?}", attr, form);
        }
        break; // TODO
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
