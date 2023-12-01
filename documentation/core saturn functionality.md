# Saturn Core Functionality

## Introduction

This document provides an overview of several different important buttons in Saturn.

## Functionality

### Play

The Play functionality allows you to run a program or execute a specific code snippet. By clicking the Play button, you can start the execution of your code and see the output or behavior of your program.

### Pause

The Pause functionality allows you to temporarily halt the execution of a running program. It is useful when you want to pause the program's execution to inspect variables or debug an issue.

### Stop

The Stop functionality allows you to terminate the execution of a running program. Clicking the Stop button will immediately halt the program's execution and return you to the code editor.

### Step

The Step functionality allows you to execute your code one line at a time. By clicking the Step button, you can move forward through your code, observing the changes and behavior of your program at each step.

### Backstep

The Backstep functionality allows you to reverse the execution of your code one line at a time. By clicking the Backstep button, you can move backward through your code, undoing the changes and reverting the behavior of your program at each step.

### Breakpoint

The Breakpoint functionality allows you to set a marker in your code for debugging purposes. By setting a breakpoint, you can pause the execution of your program at a specific line, allowing you to inspect variables, evaluate expressions, and analyze the state of your program at that point.

## The Console Tab

In MIPS Assembly, you can print values using the following methods:

1. **Print Integer**: To print an integer value, you can use the `li` (load immediate) instruction to load the value into a register, and then use the `syscall` instruction with the appropriate system call number to print the value.

2. **Print String**: To print a string, you can load the address of the string into a register using the `la` (load address) instruction, and then use the `syscall` instruction with the appropriate system call number to print the string.

3. **Print Character**: To print a single character, you can load the ASCII value of the character into a register, and then use the `syscall` instruction with the appropriate system call number to print the character.

4. **Print Hexadecimal**: To print a value in hexadecimal format, you can convert the value to hexadecimal representation and then print it using the methods mentioned above.

To find the aforementioned system call numbers, you can refer to the documentation at 