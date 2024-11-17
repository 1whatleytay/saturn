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

.text
main:
    li $v0, 10          # Load syscall number for exit (10) into $v0
    syscall             # Make the syscall to exit
    j main              # despite trying to go back to main, we already exited the program
```

# MIPS Syscalls List

| Syscall Number | Name                  | Description                                           |
|----------------|-----------------------|-------------------------------------------------------|
| 1              | Print integer         | Prints the integer in `$a0`.                           |
| 4              | Print string          | Prints the null-terminated string at the address in `$a0`. |
| 5              | Read integer          | Reads an integer from standard input into `$v0`.       |
| 8              | Read string           | Reads a string into the buffer at the address in `$a0` with the length in `$a1`.* |
| 9              | Sbrk (allocate heap memory)                  | Allocates `$a0` bytes in the data segment and returns the address to `$v0`. |
| 10             | Exit                  | Exits from the program.                                |
| 11             | Print character       | Prints the character in `$a0`.*                         |
| 12             | Read character        | Reads a character from standard input into `$v0`.      |
| 13             | Open file             | Opens a file (parameters in `$a0`, `$a1`, `$a2`) and returns the file descriptor in `$v0`.* |
| 14             | Read from file        | Reads from a file (parameters in `$a0`, `$a1`, `$a2`) and returns the number of characters read in `$v0`.* |
| 15             | Write to file         | Writes to a file (parameters in `$a0`, `$a1`, `$a2`) and returns the number of characters written in `$v0` (negative if there is an error).* |
| 16             | Close file            | Closes a file (file descriptor in `$a0`).              |
| 17             | Exit2                 | Exits from the program with a return value.*            |
| 30             | Time (System Time)         | Retrieves the system time, providing the low and high order 32 bits separately in `$a0` and `$a1`.*           |
| 31             | MIDI Out                   | Generates a MIDI tone with specified pitch, duration, instrument, and volume, and returns immediately.*       |
| 32             | Sleep                      | Pauses the execution of the program for a specified number of milliseconds, as given in `$a0`.               |
| 33             | MIDI Out Synchronous       | Similar to Syscall 31, but waits until the tone generation is complete before returning.*                     |
| 34             | Print Integer in Hexadecimal| Prints an integer in hexadecimal format (8 hexadecimal digits, left-padded if necessary), with the value provided in `$a0`.                                   |
| 35             | Print Integer in Binary    | Displays an integer in binary format (32 bits, left-padded if necessary), taking the integer from `$a0`.                                        |
| 36             | Print Integer as Unsigned  | Prints an integer as an unsigned decimal value, with the integer specified in `$a0`.                        |
| 40             | Set Seed                   | Sets the seed for a pseudorandom number generator, with the generator ID in `$a0` and the seed in `$a1`.*    |
| 41             | Random Integer             | Generates a pseudorandom integer using the generator specified in `$a0` and returns the value.*              |
| 42             | Random Integer Range       | Produces a pseudorandom integer within a specified range, using the generator in `$a0` and upper bound in `$a1`.* |

##
Service 8 - Follows semantics of UNIX 'fgets'. For specified length n, string can be no longer than n-1. If less than that, adds newline to end. In either case, then pads with null byte If n = 1, input is ignored and null byte placed at buffer address. If n < 1, input is ignored and nothing is written to the buffer.

Service 11 - Prints ASCII character corresponding to contents of low-order byte.

Service 13 - Saturn implements three flag values: 0 for read-only, 1 for write-only with create, and 9 for write-only with create and append. It ignores mode. The returned file descriptor will be negative if the operation failed. The underlying file I/O implementation uses Rust's read and write to inputstreams. 

Service 30 - System time is obtained using Rust's std::time::SystemTime and std::time::UNIX_EPOCH, which provide the number of milliseconds since 1 January 1970.

Services 31,33 - Simulate MIDI output through MIDI.js. Details below.

Services 40-42 use underlying Rust pseudorandom number generators, analogous to those provided by the rand crate. Each stream (identified by $a0 contents) is modeled by a different Random object. There are no default seed values, so use the Set Seed service (40) if replicated random sequences are desired.



## Using SYSCALL system services 31 and 33: MIDI output
These system services provide a means of producing sound. MIDI output is played back through MIDI.js.
Service 31 will generate the tone then immediately return. Service 33 will generate the tone then sleep for the tone's duration before returning. Thus it essentially combines services 31 and 32.

This service requires four parameters as follows:
pitch ($a0)
Accepts a positive byte value (0-127) that denotes a pitch as it would be represented in MIDI
Each number is one semitone / half-step in the chromatic scale.
0 represents a very low C and 127 represents a very high G (a standard 88 key piano begins at 9-A and ends at 108-C).
If the parameter value is outside this range, it applies a default value 60 which is the same as middle C on a piano.
From middle C, all other pitches in the octave are as follows:
61 = C# or Db
62 = D
63 = D# or Eb
64 = E or Fb
65 = E# or F
66 = F# or Gb
67 = G
68 = G# or Ab
69 = A
70 = A# or Bb
71 = B or Cb
72 = B# or C
To produce these pitches in other octaves, add or subtract multiples of 12.

duration in milliseconds ($a1)
Accepts a positive integer value that is the length of the tone in milliseconds.
If the parameter value is negative, it applies a default value of one second (1000 milliseconds).

instrument ($a2)
Accepts a positive byte value (0-127) that denotes the General MIDI "patch" used to play the tone.
If the parameter is outside this range, it applies a default value 0 which is an Acoustic Grand Piano.
General MIDI standardizes the number associated with each possible instrument (often referred to as program change numbers), however it does not determine how the tone will sound. This is determined by the synthesizer that is producing the sound. Thus a Tuba (patch 58) on one computer may sound different than that same patch on another computer.
The 128 available patches are divided into instrument families of 8:
0-7	Piano	64-71	Reed
8-15	Chromatic Percussion	72-79	Pipe
16-23	Organ	80-87	Synth Lead
24-31	Guitar	88-95	Synth Pad
32-39	Bass	96-103	Synth Effects
40-47	Strings	104-111	Ethnic
48-55	Ensemble	112-119	Percussion
56-63	Brass	120-127	Sound Effects
Note that General MIDI usually refers to patches 1-128. When referring to a list of General MIDI patches, 1 must be subtracted to play the correct patch. For a full list of General MIDI instruments, see https://fmslogo.sourceforge.io/manual/midi-instrument.html. The General MIDI channel 10 percussion key map is not relevant to the toneGenerator method because it always defaults to MIDI channel 1.

volume ($a3)
Accepts a positive byte value (0-127) where 127 is the loudest and 0 is silent. This value denotes MIDI velocity which refers to the initial attack of the tone.
If the parameter value is outside this range, it applies a default value 100.
MIDI velocity measures how hard a note on (or note off) message is played, perhaps on a MIDI controller like a keyboard. Most MIDI synthesizers will translate this into volume on a logarithmic scale in which the difference in amplitude decreases as the velocity value increases.
Note that velocity value on more sophisticated synthesizers can also affect the timbre of the tone (as most instruments sound different when they are played louder or softer).

Note: since the files for MIDI sounds are not included when Saturn is first installed, the first time you try to play a MIDI sound, Saturn will try to download these files (it will print an orange info message in the console when it tries to do this), this means you have to have an internet connection the first time you try to play a MIDI sound.

Saturn may occasionally have to reload the MIDI sound files into memory, so its encouraged for you to play a sound of duration 0 (totally silent) when your app starts to give the indication to Saturn that you're going to be playing MIDI files.