use core::mem::transmute;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AsciiCode {
    Null,
    StartOfHeading,
    StartOfText,
    EndOfText,
    EndOfTransmission,
    Enquiry,
    Acknowledge,
    Bell,
    Backspace,
    HorizontalTab,
    NewLine,
    VerticalTab,
    NewPage,
    CarriageReturn,
    ShiftOut,
    ShiftIn,
    DataLinkEscape,
    DeviceControl1,
    DeviceControl2,
    DeviceControl3,
    DeviceControl4,
    NegativeAcknowledge,
    SynchronousIdle,
    EndOfTransBlock,
    Cancel,
    EndOfMedium,
    Substitute,
    Escape,
    FileSeparator,
    GroupSeparator,
    RecordSeparator,
    UnitSeparator,
    Space,
    Exclamation,  // !
    Quotation,    // "
    Hash,         // #
    Doll,         // $
    Percent,      // %
    Ampersand,    // &
    Apostrophe,   // '
    LParenthesis, // (
    RParenthesis, // )
    Asterisk,     // *
    Plus,         // +
    Comma,        // ,
    Hyphen,       // -
    FullStop,     // .
    Solidius,     // /
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Colon,       // :
    Semiclon,    // ;
    LessThan,    // <
    Equal,       // =
    GreaterThan, // >
    Question,    // ?
    At,          // @
    LargeA,
    LargeB,
    LargeC,
    LargeD,
    LargeE,
    LargeF,
    LargeG,
    LargeH,
    LargeI,
    LargeJ,
    LargeK,
    LargeL,
    LargeM,
    LargeN,
    LargeO,
    LargeP,
    LargeQ,
    LargeR,
    LargeS,
    LargeT,
    LargeU,
    LargeV,
    LargeW,
    LargeX,
    LargeY,
    LargeZ,
    LSquareBracket,   // [
    ReverseSolidus,   // \
    RSquareBracket,   // ]
    CircumflexAccent, // ^
    LowLine,          // _
    GraveAccent,      // `
    SmallA,
    SmallB,
    SmallC,
    SmallD,
    SmallE,
    SmallF,
    SmallG,
    SmallH,
    SmallI,
    SmallJ,
    SmallK,
    SmallL,
    SmallM,
    SmallN,
    SmallO,
    SmallP,
    SmallQ,
    SmallR,
    SmallS,
    SmallT,
    SmallU,
    SmallV,
    SmallW,
    SmallX,
    SmallY,
    SmallZ,
    LCurlyBracket, // {
    VerticalLine,  // |
    RCurlyBracket, // }
    Tilde,         // ~
    Delete,
}

impl From<u8> for AsciiCode {
    fn from(value: u8) -> Self {
        if value <= Self::Delete as u8 {
            return unsafe { transmute(value) };
        }

        panic!("Invalid value for AsciiCode");
    }
}
