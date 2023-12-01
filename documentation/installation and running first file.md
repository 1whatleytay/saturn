# Installation
Head to the [releases](https://github.com/1whatleytay/saturn/releases) page to download the installation files.
If you are just getting started, we recommend installing the [latest version](https://github.com/1whatleytay/saturn/releases/latest).
For experimental use, install the [pre-release version](https://github.com/1whatleytay/saturn/releases/tag/app-v0.1.8).

Follow the instructions after running the file you downloaded.
Once completed Saturn is fully installed in your system.
## Windows
Download the file ending with `.msi` or `.exe`.
## macOS
Download the file ending with `.dmg`.
## Linux
Download the file ending with `.deb` or `.AppImage`.

# Getting Started
Once you have installed Saturn, you are ready to write your first program!

After opening Saturn, there is an **Untitled** file to write in.

Let's create a basic program to add and subtract 2 numbers.

Every assembly program consists of 2 parts: the data and the code.

Let's first create our data section with 2 variables.
```	
	# All memory structures are placed after the
	# .data assembler directive
	.data
	# The .word assembler directive reserves space
	# in memory for a single 4-byte word with an initial value
value:	.word 12 		# This holes the number 12
Z:	.word 0 		# This holds the number 0
```

Now we can write the code section using the `.text` directive.
```
	# All program code is placed after the
	# .text assembler directive
	.text 		
	li $t2, 25		# Load immediate value (25) 
	lw $t3, value		# Load the word stored in value (see above)
	add $t4, $t2, $t3	# Add t4 = t2 + t3 (25 + 12) = 37 
	sub $t5, $t2, $t3	# Subtract t5 = t2 - t3 (25 - 12) = 13
	sw $t5, Z		#Store the answer in Z (declared above)  	
	li $v0, 10 		# Sets $v0 to "10" to select exit syscall
	syscall 		# Exit
```

Run the program and click on the registers tab.

After scrolling down to the temporary registers, you should see 25 in $t2, 12 in $3, 37 in $t4, and 13 in $t5.

This program is based on this [example program](https://ecs-network.serv.pacific.edu/ecpe-170/tutorials/example1.asm/view).
