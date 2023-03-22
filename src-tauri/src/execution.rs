use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use titan::debug::Debugger;
use crate::state::{DebuggerBody, MemoryType};
use crate::syscall::{SyscallDelegate, SyscallResult, SyscallState};
use serde::Serialize;
use titan::cpu::state::Registers;
use titan::debug::debugger::{DebugFrame, DebuggerMode};
use crate::display::{FlushDisplayBody, read_display};

#[derive(Serialize)]
#[serde(tag="type")]
pub enum ResumeMode {
    Running,
    Invalid { message: String },
    Paused,
    Breakpoint,
    Finished { pc: u32, code: Option<u32> },
}

impl From<DebuggerMode> for ResumeMode {
    fn from(value: DebuggerMode) -> Self {
        match value {
            DebuggerMode::Running => ResumeMode::Running,
            DebuggerMode::Recovered => ResumeMode::Breakpoint,
            DebuggerMode::Invalid(error) => ResumeMode::Invalid {
                message: format!("{}", error)
            },
            DebuggerMode::Paused => ResumeMode::Paused,
            DebuggerMode::Breakpoint => ResumeMode::Breakpoint
        }
    }
}

#[derive(Serialize)]
pub struct RegistersResult {
    pc: u32,
    line: [u32; 32],
    lo: u32,
    hi: u32,
}

impl From<Registers> for RegistersResult {
    fn from(value: Registers) -> Self {
        RegistersResult {
            pc: value.pc,
            line: value.line,
            lo: value.lo,
            hi: value.hi
        }
    }
}

#[derive(Serialize)]
pub struct ResumeResult {
    mode: ResumeMode,
    registers: RegistersResult
}

impl ResumeResult {
    fn from_frame(frame: DebugFrame, finished_pcs: &Vec<u32>, result: Option<SyscallResult>) -> ResumeResult {
        let mode = match result {
            Some(SyscallResult::Failure(message)) => ResumeMode::Invalid { message },
            Some(SyscallResult::Terminated(code)) => ResumeMode::Finished {
                pc: frame.registers.pc, code: Some(code)
            },
            Some(SyscallResult::Aborted) => ResumeMode::Paused,
            Some(SyscallResult::Exception(error)) => ResumeMode::Invalid {
                message: error.to_string()
            },
            Some(SyscallResult::Unimplemented(code)) => ResumeMode::Invalid {
                message: format!("Unimplemented syscall {}, file a bug or make a contribution at https://github.com/1whatleytay/saturn.", code)
            },
            Some(SyscallResult::Unknown(code)) => ResumeMode::Invalid {
                message: format!("Unrecognized syscall {}.", code)
            },
            _ => {
                // This is probably okay...
                if finished_pcs.contains(&frame.registers.pc) {
                    ResumeMode::Finished { pc: frame.registers.pc, code: None }
                } else {
                    frame.mode.into()
                }
            }
        };

        ResumeResult {
            mode,
            registers: frame.registers.into()
        }
    }
}

type CloneResult = (Arc<Mutex<Debugger<MemoryType>>>, Arc<Mutex<SyscallState>>, Vec<u32>);

fn lock_and_clone(state: tauri::State<'_, DebuggerBody>) -> Option<CloneResult> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    Some((pointer.debugger.clone(), pointer.delegate.clone(), pointer.finished_pcs.clone()))
}

fn flush_display(memory: &mut MemoryType, state: tauri::State<'_, FlushDisplayBody>) {
    let mut display = state.lock().unwrap();

    display.data = read_display(display.address, display.width, display.height, memory);
}

#[tauri::command]
pub async fn resume(
    count: Option<u32>,
    breakpoints: Option<Vec<u32>>,
    state: tauri::State<'_, DebuggerBody>,
    display: tauri::State<'_, FlushDisplayBody>
) -> Result<ResumeResult, ()> {
    let (
        debugger, state, finished_pcs
    ) = lock_and_clone(state).ok_or(())?;

    let debugger_clone = debugger.clone();

    if let Some(breakpoints) = breakpoints {
        let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

        debugger.lock().unwrap().set_breakpoints(breakpoints_set);
    }

    // Ensure the cancel token hasn't been set previously.
    state.lock().unwrap().renew();

    let delegate = SyscallDelegate::new(state);

    let (frame, result) = {
        if let Some(count) = count {
            for _ in 0 .. count - 1 {
                delegate.cycle(&debugger).await;
            }

            if count > 0 {
                delegate.cycle(&debugger).await
            } else {
                return Err(())
            }
        } else {
            delegate.run(&debugger).await
        }
    };

    flush_display(debugger_clone.lock().unwrap().memory(), display);

    Ok(ResumeResult::from_frame(frame, &finished_pcs, result))
}

#[tauri::command]
pub fn pause(
    state: tauri::State<'_, DebuggerBody>,
    display: tauri::State<'_, FlushDisplayBody>
) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.debugger.lock().unwrap().pause();
    pointer.delegate.lock().unwrap().cancel();

    flush_display(pointer.debugger.lock().unwrap().memory(), display);
}

#[tauri::command]
pub fn stop(
    state: tauri::State<'_, DebuggerBody>,
    display: tauri::State<'_, FlushDisplayBody>
) {
    let debugger = &mut *state.lock().unwrap();

    if let Some(pointer) = debugger {
        pointer.debugger.lock().unwrap().pause();
        pointer.delegate.lock().unwrap().cancel();

        flush_display(pointer.debugger.lock().unwrap().memory(), display);
    }

    *debugger = None;
}
