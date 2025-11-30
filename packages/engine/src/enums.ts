export enum VulframResult {
  Success = 0,
  UnknownError = 1,
  NotInitialized,
  AlreadyInitialized,
  WrongThread,
  BufferOverflow,
  // Reserved error codes for Winit 1000-1999
  WinitError = 1000,
  // Reserved error codes for WGPU 2000-2999
  WgpuInstanceError = 2000,
  // Reserved error codes for Command Processing 3000-3999
  CmdInvalidCborError = 3000,
}
