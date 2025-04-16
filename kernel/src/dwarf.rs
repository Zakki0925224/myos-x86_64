use crate::{
    error::{Error, Result},
    println, util,
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
    User(u8),
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
            0x80..=0xff => Ok(Self::User(value)),
            _ => Err(Error::Failed("Invalid UnitType value")),
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
        let unit_type = value[6].try_into()?;
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
    User(u64),
}

impl TryFrom<u64> for AbbrevTag {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        match value {
            0x01 => Ok(Self::ArrayType),
            0x02 => Ok(Self::ClassType),
            0x03 => Ok(Self::EntryPoint),
            0x04 => Ok(Self::EnumerationType),
            0x05 => Ok(Self::FormalParameter),
            0x08 => Ok(Self::ImportedDeclaration),
            0x0a => Ok(Self::Label),
            0x0b => Ok(Self::LexicalBlock),
            0x0d => Ok(Self::Member),
            0x0f => Ok(Self::PointerType),
            0x10 => Ok(Self::ReferenceType),
            0x11 => Ok(Self::CompileUnit),
            0x12 => Ok(Self::StringType),
            0x13 => Ok(Self::StructureType),
            0x15 => Ok(Self::SubroutineType),
            0x16 => Ok(Self::Typedef),
            0x17 => Ok(Self::UnionType),
            0x18 => Ok(Self::UnspecifiedParameters),
            0x19 => Ok(Self::Variant),
            0x1a => Ok(Self::CommonBlock),
            0x1b => Ok(Self::CommonInclusion),
            0x1c => Ok(Self::Inheritance),
            0x1d => Ok(Self::InlinedSubroutine),
            0x1e => Ok(Self::Module),
            0x1f => Ok(Self::PtrToMemberType),
            0x20 => Ok(Self::SetType),
            0x21 => Ok(Self::SubrangeType),
            0x22 => Ok(Self::WithStmt),
            0x23 => Ok(Self::AccessDeclaration),
            0x24 => Ok(Self::BaseType),
            0x25 => Ok(Self::CatchBlock),
            0x26 => Ok(Self::ConstType),
            0x27 => Ok(Self::Constant),
            0x28 => Ok(Self::Enumerator),
            0x29 => Ok(Self::FileType),
            0x2a => Ok(Self::Friend),
            0x2b => Ok(Self::Namelist),
            0x2c => Ok(Self::NamelistItem),
            0x2d => Ok(Self::PackedType),
            0x2e => Ok(Self::Subprogram),
            0x2f => Ok(Self::TemplateTypeParameter),
            0x30 => Ok(Self::TemplateValueParameter),
            0x31 => Ok(Self::ThrownType),
            0x32 => Ok(Self::TryBlock),
            0x33 => Ok(Self::VariantPart),
            0x34 => Ok(Self::Variable),
            0x35 => Ok(Self::VolatileType),
            0x36 => Ok(Self::DwarfProcedure),
            0x37 => Ok(Self::RestrictType),
            0x38 => Ok(Self::InterfaceType),
            0x39 => Ok(Self::Namespace),
            0x3a => Ok(Self::ImportedModule),
            0x3b => Ok(Self::UnspecifiedType),
            0x3c => Ok(Self::PartialUnit),
            0x3d => Ok(Self::ImportedUnit),
            0x3f => Ok(Self::Condition),
            0x40 => Ok(Self::SharedType),
            0x41 => Ok(Self::TypeUnit),
            0x42 => Ok(Self::RvalueReferenceType),
            0x43 => Ok(Self::TemplateAlias),
            0x44 => Ok(Self::CoarrayType),
            0x45 => Ok(Self::GenericSubrange),
            0x46 => Ok(Self::DynamicType),
            0x47 => Ok(Self::AtomicType),
            0x48 => Ok(Self::CallSite),
            0x49 => Ok(Self::CallSiteParameter),
            0x4a => Ok(Self::SkeletonUnit),
            0x4b => Ok(Self::ImmutableType),
            0x4090..=0xffff => Ok(Self::User(value)),
            _ => Err(Error::Failed("Invalid AbbrevTag value")),
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
    User(u64),
}

impl TryFrom<u64> for AbbrevAttribute {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        match value {
            0x01 => Ok(Self::Sibling),
            0x02 => Ok(Self::Location),
            0x03 => Ok(Self::Name),
            0x09 => Ok(Self::Ordering),
            0x0b => Ok(Self::ByteSize),
            0x0d => Ok(Self::BitSize),
            0x10 => Ok(Self::StmtList),
            0x11 => Ok(Self::LowPc),
            0x12 => Ok(Self::HighPc),
            0x13 => Ok(Self::Language),
            0x15 => Ok(Self::Discr),
            0x16 => Ok(Self::DiscrValue),
            0x17 => Ok(Self::Visibility),
            0x18 => Ok(Self::Import),
            0x19 => Ok(Self::StringLength),
            0x1a => Ok(Self::CommonReference),
            0x1b => Ok(Self::CompDir),
            0x1c => Ok(Self::ConstValue),
            0x1d => Ok(Self::ContainingType),
            0x1e => Ok(Self::DefaultValue),
            0x20 => Ok(Self::Inline),
            0x21 => Ok(Self::IsOptional),
            0x22 => Ok(Self::LowerBound),
            0x25 => Ok(Self::Producer),
            0x27 => Ok(Self::Prototyped),
            0x2a => Ok(Self::ReturnAddr),
            0x2c => Ok(Self::StartScope),
            0x2e => Ok(Self::BitStride),
            0x2f => Ok(Self::UpperBound),
            0x31 => Ok(Self::AbstractOrigin),
            0x32 => Ok(Self::Accessibility),
            0x33 => Ok(Self::AddressClass),
            0x34 => Ok(Self::Artificial),
            0x35 => Ok(Self::BaseTypes),
            0x36 => Ok(Self::CallingConvention),
            0x37 => Ok(Self::Count),
            0x38 => Ok(Self::DataMemberLocation),
            0x39 => Ok(Self::DeclColumn),
            0x3a => Ok(Self::DeclFile),
            0x3b => Ok(Self::DeclLine),
            0x3c => Ok(Self::Declaration),
            0x3d => Ok(Self::DiscrList),
            0x3e => Ok(Self::Encoding),
            0x3f => Ok(Self::External),
            0x40 => Ok(Self::FrameBase),
            0x41 => Ok(Self::Friend),
            0x42 => Ok(Self::IdentifierCase),
            0x44 => Ok(Self::NameListItem),
            0x45 => Ok(Self::Priority),
            0x46 => Ok(Self::Segment),
            0x47 => Ok(Self::Specification),
            0x48 => Ok(Self::StaticLink),
            0x49 => Ok(Self::Type),
            0x4a => Ok(Self::UseLocation),
            0x4b => Ok(Self::VariableParameter),
            0x4c => Ok(Self::Virtuality),
            0x4d => Ok(Self::VtableElemLocation),
            0x4e => Ok(Self::Allocated),
            0x4f => Ok(Self::Associated),
            0x50 => Ok(Self::DataLocation),
            0x51 => Ok(Self::ByteStride),
            0x52 => Ok(Self::EntryPc),
            0x53 => Ok(Self::UseUtf8),
            0x54 => Ok(Self::Extension),
            0x55 => Ok(Self::Ranges),
            0x56 => Ok(Self::Trampoline),
            0x57 => Ok(Self::CallColumn),
            0x58 => Ok(Self::CallFile),
            0x59 => Ok(Self::CallLine),
            0x5a => Ok(Self::Description),
            0x5b => Ok(Self::BinaryScale),
            0x5c => Ok(Self::DecimalScale),
            0x5d => Ok(Self::Small),
            0x5e => Ok(Self::DecimalSign),
            0x5f => Ok(Self::DigitCount),
            0x60 => Ok(Self::PictureString),
            0x61 => Ok(Self::Mutable),
            0x62 => Ok(Self::ThreadsScaled),
            0x63 => Ok(Self::Explicit),
            0x64 => Ok(Self::ObjectPointer),
            0x65 => Ok(Self::Endianity),
            0x66 => Ok(Self::Elemental),
            0x67 => Ok(Self::Pure),
            0x68 => Ok(Self::Recursive),
            0x69 => Ok(Self::Signature),
            0x6a => Ok(Self::MainSubprogram),
            0x6b => Ok(Self::DataBitOffset),
            0x6c => Ok(Self::ConstExpr),
            0x6d => Ok(Self::EnumClass),
            0x6e => Ok(Self::LinkageName),
            0x6f => Ok(Self::StringLengthBitSize),
            0x70 => Ok(Self::StringLengthByteSize),
            0x71 => Ok(Self::Rank),
            0x72 => Ok(Self::StrOffsetsBase),
            0x73 => Ok(Self::AddrBase),
            0x74 => Ok(Self::RnglistsBase),
            0x76 => Ok(Self::DwoName),
            0x77 => Ok(Self::Reference),
            0x78 => Ok(Self::RvalueReference),
            0x79 => Ok(Self::Macros),
            0x7a => Ok(Self::CallAllCalls),
            0x7b => Ok(Self::CallAllSourceCalls),
            0x7c => Ok(Self::CallAllTailCalls),
            0x7d => Ok(Self::CallReturnPc),
            0x7e => Ok(Self::CallValue),
            0x7f => Ok(Self::CallOrigin),
            0x80 => Ok(Self::CallParameter),
            0x81 => Ok(Self::CallPc),
            0x82 => Ok(Self::CallTailCall),
            0x83 => Ok(Self::CallTarget),
            0x84 => Ok(Self::CallTargetClobbered),
            0x85 => Ok(Self::CallDataLocation),
            0x86 => Ok(Self::CallDataValue),
            0x87 => Ok(Self::Noreturn),
            0x88 => Ok(Self::Alignment),
            0x89 => Ok(Self::ExportSymbols),
            0x8a => Ok(Self::Deleted),
            0x8b => Ok(Self::Defaulted),
            0x8c => Ok(Self::LoclistsBase),
            0x2000..=0x3fff => Ok(Self::User(value)),
            _ => Err(Error::Failed("Invalid AbbrevAttribute value")),
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
    Indirect(u64),
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
    ImplicitConst(i64),
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
}

impl TryFrom<u64> for AbbrevForm {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        match value {
            0x01 => Ok(Self::Addr),
            0x03 => Ok(Self::Block2),
            0x04 => Ok(Self::Block4),
            0x05 => Ok(Self::Data2),
            0x06 => Ok(Self::Data4),
            0x07 => Ok(Self::Data8),
            0x08 => Ok(Self::String),
            0x09 => Ok(Self::Block),
            0x0a => Ok(Self::Block1),
            0x0b => Ok(Self::Data1),
            0x0c => Ok(Self::Flag),
            0x0d => Ok(Self::Sdata),
            0x0e => Ok(Self::Strp),
            0x0f => Ok(Self::Udata),
            0x10 => Ok(Self::RefAddr),
            0x11 => Ok(Self::Ref1),
            0x12 => Ok(Self::Ref2),
            0x13 => Ok(Self::Ref4),
            0x14 => Ok(Self::Ref8),
            0x15 => Ok(Self::RefUdata),
            0x16 => Ok(Self::Indirect(0)),
            0x17 => Ok(Self::SecOffset),
            0x18 => Ok(Self::Exprloc),
            0x19 => Ok(Self::FlagPresent),
            0x1a => Ok(Self::Strx),
            0x1b => Ok(Self::Addrx),
            0x1c => Ok(Self::RefSup4),
            0x1d => Ok(Self::StrpSup),
            0x1e => Ok(Self::Data16),
            0x1f => Ok(Self::LineStrp),
            0x20 => Ok(Self::RefSig8),
            0x21 => Ok(Self::ImplicitConst(0)),
            0x22 => Ok(Self::Loclistx),
            0x23 => Ok(Self::Rnglistx),
            0x24 => Ok(Self::RefSup8),
            0x25 => Ok(Self::Strx1),
            0x26 => Ok(Self::Strx2),
            0x27 => Ok(Self::Strx3),
            0x28 => Ok(Self::Strx4),
            0x29 => Ok(Self::Addrx1),
            0x2a => Ok(Self::Addrx2),
            0x2b => Ok(Self::Addrx3),
            0x2c => Ok(Self::Addrx4),
            _ => Err(Error::Failed("Invalid AbbrevForm value")),
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

fn read_leb128(slice: &[u8], offset: &mut usize) -> i64 {
    let mut res = 0;
    let mut shift = 0;
    let mut byte = 0;

    while *offset < slice.len() {
        byte = slice[*offset];
        *offset += 1;
        res |= ((byte & 0x7f) as i64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    // sign extend
    if (shift < 64) && (byte & 0x40 != 0) {
        res |= !0 << shift;
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

        let tag = read_uleb128(debug_abbrev_slice, &mut offset).try_into()?;
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

            let attr = name.try_into()?;
            let mut form = form.try_into()?;

            match form {
                AbbrevForm::Indirect(ref mut v) => {
                    *v = read_uleb128(debug_abbrev_slice, &mut offset);
                }
                AbbrevForm::ImplicitConst(ref mut v) => {
                    *v = read_leb128(debug_abbrev_slice, &mut offset);
                }
                _ => (),
            }

            attributes.push((attr, form));
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

fn parse_die(
    debug_abbrev_slice: &[u8],
    debug_str_slice: &[u8],
    debug_line_str_slice: &[u8],
    debug_info: &DebugInfo,
) -> Result<()> {
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
        println!("DIE code: 0x{:x}, tag: {:?}", code, abbrev.tag);

        for (attr, form) in &abbrev.attributes {
            println!("  Attribute: {:?}, Form: {:?}", attr, form);

            match form {
                AbbrevForm::Strp => {
                    let str_offset = u32::from_le_bytes([
                        die_data[offset],
                        die_data[offset + 1],
                        die_data[offset + 2],
                        die_data[offset + 3],
                    ]) as usize;
                    offset += 4;
                    let s = util::cstring::from_slice(&debug_str_slice[str_offset..]);
                    println!("    Strp: {:?}", s);
                }
                AbbrevForm::LineStrp => {
                    let str_offset = u32::from_le_bytes([
                        die_data[offset],
                        die_data[offset + 1],
                        die_data[offset + 2],
                        die_data[offset + 3],
                    ]) as usize;
                    offset += 4;
                    let s = util::cstring::from_slice(&debug_line_str_slice[str_offset..]);
                    println!("    LineStrp: {:?}", s);
                }
                AbbrevForm::Data1 => {
                    let data = die_data[offset];
                    offset += 1;
                    println!("    Data1: 0x{:x}", data);
                }
                AbbrevForm::Data2 => {
                    let data = u16::from_le_bytes([die_data[offset], die_data[offset + 1]]);
                    offset += 2;
                    println!("    Data2: 0x{:x}", data);
                }
                AbbrevForm::Data4 => {
                    let data = u32::from_le_bytes([
                        die_data[offset],
                        die_data[offset + 1],
                        die_data[offset + 2],
                        die_data[offset + 3],
                    ]);
                    offset += 4;
                    println!("    Data4: 0x{:x}", data);
                }
                AbbrevForm::Data8 => {
                    let data = u64::from_le_bytes([
                        die_data[offset],
                        die_data[offset + 1],
                        die_data[offset + 2],
                        die_data[offset + 3],
                        die_data[offset + 4],
                        die_data[offset + 5],
                        die_data[offset + 6],
                        die_data[offset + 7],
                    ]);
                    offset += 8;
                    println!("    Data8: 0x{:x}", data);
                }
                _ => {
                    break; // TODO
                }
            }
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

    let debug_str_sh = elf64
        .section_header_by_name(".debug_str")
        .ok_or(Error::Failed("Failed to find .debug_str section"))?;

    let debug_str_slice = elf64
        .data_by_section_header(debug_str_sh)
        .ok_or(Error::Failed("Failed to get .debug_str section data"))?;

    let debug_line_str_sh = elf64
        .section_header_by_name(".debug_line_str")
        .ok_or(Error::Failed("Failed to find .debug_line_str section"))?;

    let debug_line_str_slice = elf64
        .data_by_section_header(debug_line_str_sh)
        .ok_or(Error::Failed("Failed to get .debug_line_str section data"))?;

    // parse DIE syntax tree
    let debug_infos = parse_debug_info(debug_info_slice)?;

    for debug_info in &debug_infos {
        parse_die(
            debug_abbrev_slice,
            debug_str_slice,
            debug_line_str_slice,
            debug_info,
        )?;
    }

    Ok(())
}
