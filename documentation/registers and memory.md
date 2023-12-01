# Understanding the Registers and Memory Tabs in MIPS Simulator

When working with a MIPS simulator, two of the essential interfaces you'll interact with are the Registers and Memory tabs. Here's a guide to understanding and using these tabs effectively.

## Registers Tab

The Registers tab displays the current values in the CPU's registers. These are categorized into several sections:

- **System**: Special registers like the program counter (`pc`) and the `hi` and `lo` registers for multiplication and division operations.
- **Values**: General-purpose registers like `$v0` and `$a0-$a3`, which are often used to hold syscall numbers and arguments.
- **Temporary**: Registers `$t0-$t9` used for temporary storage.
- **Saved**: Registers `$s0-$s7` used for saving important values across function calls.

### Features:
- **Reading Register Values**: You can see the current value of each register in either decimal (`Dec`) or hexadecimal (`Hex`) format.
- **Modifying Register Values**: Some simulators allow you to modify the values of registers during debugging.

### Usage Notes:
- **Switching Between Dec/Hex Modes**: A toggle usually allows you to switch between viewing register values in decimal or hexadecimal.
- **When Values are Available**: Register values are available and updated while the program is running. It's important to have the simulator in 'Running' mode to see the current register values.

## Memory Tab

The Memory tab provides a view into the simulated machine's memory, showing contents at different addresses.

### Features:
- **Reading Memory**: The tab displays memory contents, allowing you to monitor the values stored in different memory locations.
- **Navigating Memory**: You can jump to interesting regions, such as the `.data` section where initialized data is stored or the `.text` section which contains the compiled machine code.

### Usage Notes:
- **Understanding When Memory is Populated**: Similar to the Registers tab, the Memory tab will display information when the program is in a running state. If the program is not running or is paused, the memory contents may not reflect the current state of program execution.

By familiarizing yourself with the Registers and Memory tabs, you can more effectively debug and understand the behavior of your MIPS assembly programs.
