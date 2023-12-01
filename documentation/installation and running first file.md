# Installation
Head to the [releases](https://github.com/1whatleytay/saturn/releases) page to download the installation files.
If you are just getting started, we recommend installing the [latest version](https://github.com/1whatleytay/saturn/releases/latest).
For experimental use, install the [pre-release version](https://github.com/1whatleytay/saturn/releases/tag/app-v0.1.8).

For general use, download the file ending with `setup.exe`.

Follow the instructions after running the file you downloaded.
Once completed Saturn is fully installed in your system.
## Windows
Download the file ending with `.msi`.
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
	.data
value:	.word 12 		
Z:		.word 0 		
```

Now we can write the code section using the `.text` directive.
```
	.text 		
	li $t2, 25	
	lw $t3, value	
	add $t4, $t2, $t3	
	sub $t5, $t2, $t3	
	sw $t5, Z		
	li $v0, 10 
	syscall 
```

Run the program and click on the registers tab.

After scrolling down to the temporary registers, you should see 25 in $t2, 12 in $3, 37 in $t4, and 13 in $t5.

This program is based on this [example program](https://ecs-network.serv.pacific.edu/ecpe-170/tutorials/example1.asm/view).
