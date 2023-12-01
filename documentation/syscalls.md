# MIPS Assembly Syscalls Guide

This guide provides an overview of how to use syscalls in MIPS assembly. Syscalls (system calls) are crucial for performing various operations in MIPS, such as input/output, process control, and more.

## Using a Syscall

To use a syscall in Saturn, or MIPS assembly in general, follow these steps:

1. Set the Syscall Number:
   - Load the number of your desired syscall into the `$v0` register.
   - Example: `li $v0, 1` loads the syscall number for 'integer print' into `$v0`.

2. Prepare Arguments:
   - If the syscall requires arguments, place them in the appropriate registers before making the syscall.
   - Example: `li $a0, 1` will prepare the integer that we will print, assuming this is used with integer print

3. Execute the Syscall:
   - call `syscall`

4. Handle the Return Value:
   - After execution, the return value is stored in `$v0`.

## Example: Printing an Integer

Here's an example of using a syscall to print an integer:

```assembly
.data
number: .word 100    # Example number to print

.text
main:
    lw $a0, number   # Load integer into $a0
    li $v0, 1        # Load wanted syscall
    syscall          # Execute syscall
```

# MIPS Syscalls List

| Syscall Number | Name                  | Description                                           |
|----------------|-----------------------|-------------------------------------------------------|
| 1              | Print integer         | Prints the integer in `$a0`                           |
| 2              | Print float           | Prints the float in `$f12`                            |
| 3              | Print double          | Prints the double in `$f12`                           |
| 4              | Print string          | Prints the null-terminated string at the address in `$a0` |
| 5              | Read integer          | Reads an integer from standard input into `$v0`       |
| 6              | Read float            | Reads a float from standard input into `$f0`          |
| 7              | Read double           | Reads a double from standard input into `$f0`         |
| 8              | Read string           | Reads a string into the buffer at the address in `$a0` with the length in `$a1` |
| 9              | Sbrk (allocate heap memory)                  | Allocates `$a0` bytes in the data segment and returns the address to `$v0` |
| 10             | Exit                  | Exits from the program                                |
| 11             | Print character       | Prints the character in `$a0`                         |
| 12             | Read character        | Reads a character from standard input into `$v0`      |
| 13             | Open file             | Opens a file (parameters in `$a0`, `$a1`, `$a2`) and returns the file descriptor in `$v0` |
| 14             | Read from file        | Reads from a file (parameters in `$a0`, `$a1`, `$a2`) and returns the number of characters read in `$v0` |
| 15             | Write to file         | Writes to a file (parameters in `$a0`, `$a1`, `$a2`) and returns the number of characters written in `$v0` (negative if there is an error) |
| 16             | Close file            | Closes a file (file descriptor in `$a0`)              |
| 17             | Exit2                 | Exits from the program with a return value            |


