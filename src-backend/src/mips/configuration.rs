use titan::cpu::Memory;
use titan::cpu::memory::{Mountable, Region};
use titan::elf::Elf;
use titan::mips::assembler::registers::RegisterSlot::StackPointer;
use titan::mips::cpu::{Registers, State};
use titan::mips::cpu::registers::WhichRegister::Pc;

pub fn create_elf_state<Mem: Memory + Mountable, Reg: Registers>(
    elf: &Elf,
    heap_size: u32,
    mut memory: Mem,
    mut registers: Reg,
) -> State<Mem, Reg> {
    for header in &elf.program_headers {
        let region = Region {
            start: header.virtual_address,
            data: header.data.clone(),
        };

        memory.mount(region)
    }

    let heap_end = 0x7FFFFFFCu32;

    let heap = Region {
        start: heap_end - heap_size,
        data: vec![0; heap_size as usize],
    };

    memory.mount(heap);

    registers.set_l(StackPointer, heap_end);
    registers.set(Pc, elf.header.program_entry);

    State::new(registers, memory)
}
