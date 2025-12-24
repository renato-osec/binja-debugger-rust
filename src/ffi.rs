// FFI bindings for Binary Ninja Debugger
// Auto-generated from api/ffi.h

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::c_char;
use std::ffi::c_void;

// opaque types from binaryninjacore
pub type BNBinaryView = c_void;
pub type BNFileMetadata = c_void;
pub type BNArchitecture = c_void;
pub type BNDataBuffer = c_void;
pub type BNMetadata = c_void;
pub type BNSettings = c_void;
pub type BNLowLevelILFunction = c_void;
pub type BNMediumLevelILFunction = c_void;
pub type BNHighLevelILFunction = c_void;
pub type BNVariable = c_void;

// opaque debugger types
pub type BNDebuggerController = c_void;
pub type BNDebugAdapterType = c_void;
pub type BNDebugAdapter = c_void;
pub type BNDebuggerState = c_void;

// function graph type enum (from binaryninjacore.h)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNFunctionGraphType {
    InvalidILViewType = -1,
    NormalFunctionGraph = 0,
    LowLevelILFunctionGraph = 1,
    LiftedILFunctionGraph = 2,
    LowLevelILSSAFormFunctionGraph = 3,
    MediumLevelILFunctionGraph = 4,
    MediumLevelILSSAFormFunctionGraph = 5,
    MappedMediumLevelILFunctionGraph = 6,
    MappedMediumLevelILSSAFormFunctionGraph = 7,
    HighLevelILFunctionGraph = 8,
    HighLevelILSSAFormFunctionGraph = 9,
    HighLevelLanguageRepresentationFunctionGraph = 10,
}

// Debug process
#[repr(C)]
pub struct BNDebugProcess {
    pub m_pid: u32,
    pub m_processName: *mut c_char,
}

// Debug thread
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BNDebugThread {
    pub m_tid: u32,
    pub m_rip: u64,
    pub m_isFrozen: bool,
}

// Debug frame
#[repr(C)]
pub struct BNDebugFrame {
    pub m_index: usize,
    pub m_pc: u64,
    pub m_sp: u64,
    pub m_fp: u64,
    pub m_functionName: *mut c_char,
    pub m_functionStart: u64,
    pub m_module: *mut c_char,
}

// Debug module
#[repr(C)]
pub struct BNDebugModule {
    pub m_name: *mut c_char,
    pub m_short_name: *mut c_char,
    pub m_address: u64,
    pub m_size: usize,
    pub m_loaded: bool,
}

// Debug register
#[repr(C)]
pub struct BNDebugRegister {
    pub m_name: *mut c_char,
    pub m_value: [u8; 64],
    pub m_width: usize,
    pub m_registerIndex: usize,
    pub m_hint: *mut c_char,
}

// Debug breakpoint
#[repr(C)]
pub struct BNDebugBreakpoint {
    pub module: *mut c_char,
    pub offset: u64,
    pub address: u64,
    pub enabled: bool,
}

// Module name and offset
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BNModuleNameAndOffset {
    pub module: *mut c_char,
    pub offset: u64,
}

// Stop reason
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNDebugStopReason {
    UnknownReason = 0,
    InitialBreakpoint,
    ProcessExited,
    AccessViolation,
    SingleStep,
    Calculation,
    Breakpoint,
    IllegalInstruction,
    SignalHup,
    SignalInt,
    SignalQuit,
    SignalIll,
    SignalAbrt,
    SignalEmt,
    SignalFpe,
    SignalKill,
    SignalBus,
    SignalSegv,
    SignalSys,
    SignalPipe,
    SignalAlrm,
    SignalTerm,
    SignalUrg,
    SignalStop,
    SignalTstp,
    SignalCont,
    SignalChld,
    SignalTtin,
    SignalTtou,
    SignalIo,
    SignalXcpu,
    SignalXfsz,
    SignalVtalrm,
    SignalProf,
    SignalWinch,
    SignalInfo,
    SignalUsr1,
    SignalUsr2,
    SignalStkflt,
    SignalBux,
    SignalPoll,
    ExcEmulation,
    ExcSoftware,
    ExcSyscall,
    ExcMachSyscall,
    ExcRpcAlert,
    ExcCrash,
    InternalError,
    InvalidStatusOrOperation,
    UserRequestedBreak,
    OperationNotSupported,
}

// Connection status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNDebugAdapterConnectionStatus {
    DebugAdapterNotConnectedStatus = 0,
    DebugAdapterConnectingStatus,
    DebugAdapterConnectedStatus,
}

// Target status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNDebugAdapterTargetStatus {
    DebugAdapterInvalidStatus = 0,
    DebugAdapterRunningStatus,
    DebugAdapterPausedStatus,
}

// Event type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNDebuggerEventType {
    LaunchEventType = 0,
    ResumeEventType,
    StepIntoEventType,
    StepOverEventType,
    StepReturnEventType,
    StepToEventType,
    RestartEventType,
    AttachEventType,
    DetachEventType,
    ConnectEventType,
    AdapterStoppedEventType,
    AdapterTargetExitedEventType,
    InvalidOperationEventType,
    InternalErrorEventType,
    TargetStoppedEventType,
    ErrorEventType,
    GeneralEventType,
    LaunchFailureEventType,
    StdoutMessageEventType,
    BackendMessageEventType,
    TargetExitedEventType,
    DetachedEventType,
    AbsoluteBreakpointAddedEvent,
    RelativeBreakpointAddedEvent,
    AbsoluteBreakpointRemovedEvent,
    RelativeBreakpointRemovedEvent,
    AbsoluteBreakpointEnabledEvent,
    RelativeBreakpointEnabledEvent,
    AbsoluteBreakpointDisabledEvent,
    RelativeBreakpointDisabledEvent,
    ActiveThreadChangedEvent,
    DebuggerAdapterChangedEvent,
    RegisterChangedEvent,
    ThreadStateChangedEvent,
    ForceMemoryCacheUpdateEvent,
}

// Event data structs
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BNTargetStoppedEventData {
    pub reason: BNDebugStopReason,
    pub lastActiveThread: u32,
    pub exitCode: usize,
    pub data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BNErrorEventData {
    pub error: *mut c_char,
    pub shortError: *mut c_char,
    pub data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BNTargetExitedEventData {
    pub exitCode: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BNStdoutMessageEventData {
    pub message: *mut c_char,
}

// Event data union (represented as struct with all fields)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BNDebuggerEventData {
    pub targetStoppedData: BNTargetStoppedEventData,
    pub errorData: BNErrorEventData,
    pub absoluteAddress: u64,
    pub relativeAddress: BNModuleNameAndOffset,
    pub exitData: BNTargetExitedEventData,
    pub messageData: BNStdoutMessageEventData,
}

// Main event struct
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BNDebuggerEvent {
    pub r#type: BNDebuggerEventType,
    pub data: BNDebuggerEventData,
}

// Adapter operation enum
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BNDebuggerAdapterOperation {
    DebugAdapterLaunch = 0,
    DebugAdapterAttach,
    DebugAdapterConnect,
    DebugAdapterGo,
    DebugAdapterStepInto,
    DebugAdapterStepOver,
    DebugAdapterStepReturn,
    DebugAdapterPause,
    DebugAdapterQuit,
    DebugAdapterDetach,
    DebugAdapterStepIntoReverse,
    DebugAdapterStepOverReverse,
    DebugAdapterGoReverse,
    DebugAdapterStepReturnReverse,
}

// FFI function declarations
extern "C" {
    // String allocation
    pub fn BNDebuggerAllocString(string: *const c_char) -> *mut c_char;
    pub fn BNDebuggerFreeString(string: *mut c_char);
    pub fn BNDebuggerFreeStringList(stringList: *mut *mut c_char, count: usize);

    // Controller lifecycle
    pub fn BNGetDebuggerController(data: *mut BNBinaryView) -> *mut BNDebuggerController;
    pub fn BNDebuggerDestroyController(controller: *mut BNDebuggerController);
    pub fn BNDebuggerControllerExists(data: *mut BNBinaryView) -> bool;
    pub fn BNGetDebuggerControllerFromFile(file: *mut BNFileMetadata) -> *mut BNDebuggerController;
    pub fn BNDebuggerControllerExistsFromFile(file: *mut BNFileMetadata) -> bool;
    pub fn BNDebuggerNewControllerReference(
        controller: *mut BNDebuggerController,
    ) -> *mut BNDebuggerController;
    pub fn BNDebuggerFreeController(controller: *mut BNDebuggerController);

    // Data access
    pub fn BNDebuggerGetData(controller: *mut BNDebuggerController) -> *mut BNBinaryView;
    pub fn BNDebuggerSetData(controller: *mut BNDebuggerController, data: *mut BNBinaryView);
    pub fn BNDebuggerGetRemoteArchitecture(
        controller: *mut BNDebuggerController,
    ) -> *mut BNArchitecture;

    // Status queries
    pub fn BNDebuggerIsConnected(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerIsConnectedToDebugServer(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerIsRunning(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerGetConnectionStatus(
        controller: *mut BNDebuggerController,
    ) -> BNDebugAdapterConnectionStatus;
    pub fn BNDebuggerGetTargetStatus(
        controller: *mut BNDebuggerController,
    ) -> BNDebugAdapterTargetStatus;

    // Registers and memory
    pub fn BNDebuggerGetStackPointer(controller: *mut BNDebuggerController) -> u64;
    pub fn BNDebuggerReadMemory(
        controller: *mut BNDebuggerController,
        address: u64,
        size: usize,
    ) -> *mut BNDataBuffer;
    pub fn BNDebuggerWriteMemory(
        controller: *mut BNDebuggerController,
        address: u64,
        buffer: *mut BNDataBuffer,
    ) -> bool;
    pub fn BNDebuggerGetRegisters(
        controller: *mut BNDebuggerController,
        count: *mut usize,
    ) -> *mut BNDebugRegister;
    pub fn BNDebuggerFreeRegisters(registers: *mut BNDebugRegister, count: usize);
    pub fn BNDebuggerSetRegisterValue(
        controller: *mut BNDebuggerController,
        name: *const c_char,
        value: *const u8,
    ) -> bool;
    pub fn BNDebuggerGetRegisterValue(
        controller: *mut BNDebuggerController,
        name: *const c_char,
        buffer: *mut u8,
    );

    // Process list
    pub fn BNDebuggerGetProcessList(
        controller: *mut BNDebuggerController,
        count: *mut usize,
    ) -> *mut BNDebugProcess;
    pub fn BNDebuggerFreeProcessList(processes: *mut BNDebugProcess, count: usize);
    pub fn BNDebuggerGetActivePID(controller: *mut BNDebuggerController) -> u32;

    // Threads
    pub fn BNDebuggerGetThreads(
        controller: *mut BNDebuggerController,
        count: *mut usize,
    ) -> *mut BNDebugThread;
    pub fn BNDebuggerFreeThreads(threads: *mut BNDebugThread, count: usize);
    pub fn BNDebuggerGetActiveThread(controller: *mut BNDebuggerController) -> BNDebugThread;
    pub fn BNDebuggerSetActiveThread(controller: *mut BNDebuggerController, thread: BNDebugThread);
    pub fn BNDebuggerSuspendThread(controller: *mut BNDebuggerController, tid: u32) -> bool;
    pub fn BNDebuggerResumeThread(controller: *mut BNDebuggerController, tid: u32) -> bool;

    // Frames
    pub fn BNDebuggerGetFramesOfThread(
        controller: *mut BNDebuggerController,
        tid: u32,
        count: *mut usize,
    ) -> *mut BNDebugFrame;
    pub fn BNDebuggerFreeFrames(frames: *mut BNDebugFrame, count: usize);

    // Modules
    pub fn BNDebuggerGetModules(
        controller: *mut BNDebuggerController,
        count: *mut usize,
    ) -> *mut BNDebugModule;
    pub fn BNDebuggerFreeModules(modules: *mut BNDebugModule, count: usize);

    // Execution control - non-blocking
    pub fn BNDebuggerLaunch(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerExecute(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerRestart(controller: *mut BNDebuggerController);
    pub fn BNDebuggerQuit(controller: *mut BNDebuggerController);
    pub fn BNDebuggerConnect(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerConnectToDebugServer(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerDisconnectDebugServer(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerDetach(controller: *mut BNDebuggerController);
    pub fn BNDebuggerLaunchOrConnect(controller: *mut BNDebuggerController);
    pub fn BNDebuggerAttach(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerGo(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerGoReverse(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerStepInto(controller: *mut BNDebuggerController, il: BNFunctionGraphType)
        -> bool;
    pub fn BNDebuggerStepIntoReverse(
        controller: *mut BNDebuggerController,
        il: BNFunctionGraphType,
    ) -> bool;
    pub fn BNDebuggerStepOver(controller: *mut BNDebuggerController, il: BNFunctionGraphType)
        -> bool;
    pub fn BNDebuggerStepOverReverse(
        controller: *mut BNDebuggerController,
        il: BNFunctionGraphType,
    ) -> bool;
    pub fn BNDebuggerStepReturn(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerStepReturnReverse(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerRunTo(
        controller: *mut BNDebuggerController,
        remoteAddresses: *const u64,
        count: usize,
    ) -> bool;
    pub fn BNDebuggerRunToReverse(
        controller: *mut BNDebuggerController,
        remoteAddresses: *const u64,
        count: usize,
    ) -> bool;
    pub fn BNDebuggerPause(controller: *mut BNDebuggerController);

    // Execution control - blocking (AndWait)
    pub fn BNDebuggerLaunchAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;
    pub fn BNDebuggerConnectAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;
    pub fn BNDebuggerAttachAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;
    pub fn BNDebuggerGoAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;
    pub fn BNDebuggerGoReverseAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;
    pub fn BNDebuggerStepIntoAndWait(
        controller: *mut BNDebuggerController,
        il: BNFunctionGraphType,
    ) -> BNDebugStopReason;
    pub fn BNDebuggerStepIntoReverseAndWait(
        controller: *mut BNDebuggerController,
        il: BNFunctionGraphType,
    ) -> BNDebugStopReason;
    pub fn BNDebuggerStepOverAndWait(
        controller: *mut BNDebuggerController,
        il: BNFunctionGraphType,
    ) -> BNDebugStopReason;
    pub fn BNDebuggerStepOverReverseAndWait(
        controller: *mut BNDebuggerController,
        il: BNFunctionGraphType,
    ) -> BNDebugStopReason;
    pub fn BNDebuggerStepReturnAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;
    pub fn BNDebuggerStepReturnReverseAndWait(
        controller: *mut BNDebuggerController,
    ) -> BNDebugStopReason;
    pub fn BNDebuggerRunToAndWait(
        controller: *mut BNDebuggerController,
        remoteAddresses: *const u64,
        count: usize,
    ) -> BNDebugStopReason;
    pub fn BNDebuggerRunToReverseAndWait(
        controller: *mut BNDebuggerController,
        remoteAddresses: *const u64,
        count: usize,
    ) -> BNDebugStopReason;
    pub fn BNDebuggerPauseAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;
    pub fn BNDebuggerQuitAndWait(controller: *mut BNDebuggerController);
    pub fn BNDebuggerRestartAndWait(controller: *mut BNDebuggerController) -> BNDebugStopReason;

    // Adapter type and configuration
    pub fn BNDebuggerGetAdapterType(controller: *mut BNDebuggerController) -> *mut c_char;
    pub fn BNDebuggerSetAdapterType(controller: *mut BNDebuggerController, adapter: *const c_char);
    pub fn BNDebuggerGetRemoteHost(controller: *mut BNDebuggerController) -> *mut c_char;
    pub fn BNDebuggerGetRemotePort(controller: *mut BNDebuggerController) -> u32;
    pub fn BNDebuggerGetPIDAttach(controller: *mut BNDebuggerController) -> i32;
    pub fn BNDebuggerGetInputFile(controller: *mut BNDebuggerController) -> *mut c_char;
    pub fn BNDebuggerGetExecutablePath(controller: *mut BNDebuggerController) -> *mut c_char;
    pub fn BNDebuggerGetWorkingDirectory(controller: *mut BNDebuggerController) -> *mut c_char;
    pub fn BNDebuggerGetRequestTerminalEmulator(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerGetCommandLineArguments(controller: *mut BNDebuggerController) -> *mut c_char;
    pub fn BNDebuggerSetRemoteHost(controller: *mut BNDebuggerController, host: *const c_char);
    pub fn BNDebuggerSetRemotePort(controller: *mut BNDebuggerController, port: u32);
    pub fn BNDebuggerSetPIDAttach(controller: *mut BNDebuggerController, pid: i32);
    pub fn BNDebuggerSetInputFile(controller: *mut BNDebuggerController, path: *const c_char);
    pub fn BNDebuggerSetExecutablePath(controller: *mut BNDebuggerController, path: *const c_char);
    pub fn BNDebuggerSetWorkingDirectory(controller: *mut BNDebuggerController, path: *const c_char);
    pub fn BNDebuggerSetRequestTerminalEmulator(
        controller: *mut BNDebuggerController,
        requestEmulator: bool,
    );
    pub fn BNDebuggerSetCommandLineArguments(
        controller: *mut BNDebuggerController,
        args: *const c_char,
    );

    // Breakpoints
    pub fn BNDebuggerGetBreakpoints(
        controller: *mut BNDebuggerController,
        count: *mut usize,
    ) -> *mut BNDebugBreakpoint;
    pub fn BNDebuggerFreeBreakpoints(breakpoints: *mut BNDebugBreakpoint, count: usize);
    pub fn BNDebuggerDeleteAbsoluteBreakpoint(controller: *mut BNDebuggerController, address: u64);
    pub fn BNDebuggerDeleteRelativeBreakpoint(
        controller: *mut BNDebuggerController,
        module: *const c_char,
        offset: u64,
    );
    pub fn BNDebuggerAddAbsoluteBreakpoint(controller: *mut BNDebuggerController, address: u64);
    pub fn BNDebuggerAddRelativeBreakpoint(
        controller: *mut BNDebuggerController,
        module: *const c_char,
        offset: u64,
    );
    pub fn BNDebuggerEnableAbsoluteBreakpoint(controller: *mut BNDebuggerController, address: u64);
    pub fn BNDebuggerEnableRelativeBreakpoint(
        controller: *mut BNDebuggerController,
        module: *const c_char,
        offset: u64,
    );
    pub fn BNDebuggerDisableAbsoluteBreakpoint(controller: *mut BNDebuggerController, address: u64);
    pub fn BNDebuggerDisableRelativeBreakpoint(
        controller: *mut BNDebuggerController,
        module: *const c_char,
        offset: u64,
    );
    pub fn BNDebuggerContainsAbsoluteBreakpoint(
        controller: *mut BNDebuggerController,
        address: u64,
    ) -> bool;
    pub fn BNDebuggerContainsRelativeBreakpoint(
        controller: *mut BNDebuggerController,
        module: *const c_char,
        offset: u64,
    ) -> bool;

    // IP
    pub fn BNDebuggerGetIP(controller: *mut BNDebuggerController) -> u64;
    pub fn BNDebuggerGetLastIP(controller: *mut BNDebuggerController) -> u64;
    pub fn BNDebuggerSetIP(controller: *mut BNDebuggerController, address: u64) -> bool;

    // Address conversion
    pub fn BNDebuggerRelativeAddressToAbsolute(
        controller: *mut BNDebuggerController,
        module: *const c_char,
        offset: u64,
    ) -> u64;
    pub fn BNDebuggerAbsoluteAddressToRelative(
        controller: *mut BNDebuggerController,
        address: u64,
    ) -> BNModuleNameAndOffset;

    // Exit code and stop reason
    pub fn BNDebuggerGetExitCode(controller: *mut BNDebuggerController) -> u32;
    pub fn BNDebuggerGetStopReasonString(reason: BNDebugStopReason) -> *mut c_char;
    pub fn BNDebuggerGetStopReason(controller: *mut BNDebuggerController) -> BNDebugStopReason;

    // Stdin
    pub fn BNDebuggerWriteStdin(
        controller: *mut BNDebuggerController,
        data: *const c_char,
        len: usize,
    );

    // Backend command
    pub fn BNDebuggerInvokeBackendCommand(
        controller: *mut BNDebuggerController,
        cmd: *const c_char,
    ) -> *mut c_char;

    // Adapter activation
    pub fn BNDebuggerActivateDebugAdapter(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerIsFirstLaunch(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerIsFirstConnect(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerIsFirstConnectToDebugServer(controller: *mut BNDebuggerController) -> bool;
    pub fn BNDebuggerIsFirstAttach(controller: *mut BNDebuggerController) -> bool;

    // Debug adapter types
    pub fn BNGetDebugAdapterTypeByName(name: *const c_char) -> *mut BNDebugAdapterType;
    pub fn BNDebugAdapterTypeCanExecute(
        adapter: *mut BNDebugAdapterType,
        data: *mut BNBinaryView,
    ) -> bool;
    pub fn BNDebugAdapterTypeCanConnect(
        adapter: *mut BNDebugAdapterType,
        data: *mut BNBinaryView,
    ) -> bool;
    pub fn BNGetAvailableDebugAdapterTypes(
        data: *mut BNBinaryView,
        count: *mut usize,
    ) -> *mut *mut c_char;

    // Module comparison
    pub fn BNDebuggerIsSameBaseModule(module1: *const c_char, module2: *const c_char) -> bool;

    // Event callbacks
    pub fn BNDebuggerRegisterEventCallback(
        controller: *mut BNDebuggerController,
        callback: Option<unsafe extern "C" fn(ctx: *mut c_void, event: *mut BNDebuggerEvent)>,
        name: *const c_char,
        ctx: *mut c_void,
    ) -> usize;
    pub fn BNDebuggerRemoveEventCallback(controller: *mut BNDebuggerController, index: usize);

    // Post event
    pub fn BNDebuggerPostDebuggerEvent(
        controller: *mut BNDebuggerController,
        event: *mut BNDebuggerEvent,
    );
}
