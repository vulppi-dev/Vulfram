/// Result codes returned by engine functions
#[derive(Debug)]
#[repr(u32)]
pub enum EngineResult {
    Success = 0,
    UnknownError = 1,
    NotInitialized,
    AlreadyInitialized,
    WrongThread,
    BufferOverflow,
    CmdInvalidCborError,
}
