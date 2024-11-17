# Bitmap Display
The bitmap tab contains a visualization of the data stored in the designated address.

![image](https://github.com/YahyaElgabra/saturn/assets/114133351/7fcbe627-4461-445e-9986-069ea79327a9)

As you can see, there is a multitude of settings to change:
**Display Width**: pixel width of the screen.
**Display Height**: pixel height of the screen.
**Address**: The starting address where the pixel data is stored, normally `0x10008000 ($gp)`

There is also the unit settings next to the width and height, and these numbers represent how many pixels make up an unit.

Below is example code to help you play around with the display.

The result of running the code should look like the display above.

```
##############################################################################
# Example: Displaying Pixels
#
# This file demonstrates how to draw pixels with different colours to the
# bitmap display.
##############################################################################

######################## Bitmap Display Configuration ########################
# - Unit width in pixels: 8
# - Unit height in pixels: 8
# - Display width in pixels: 256
# - Display height in pixels: 256
# - Base Address for Display: 0x10008000 ($gp)
##############################################################################
    .data
ADDR_DSPL:
    .word 0x10008000
RED: .word 0xff0000
NUM: .word 0x00000008
    .text
    .globl main

main:
    lw $t1, RED             # $t1 = red
    li $t2, 0x00ff00        # $t2 = green
    li $t3, 0x0000ff        # $t3 = blue
    li $t4, 0xffffff        # $t4 = white

    lw $t0, ADDR_DSPL       # $t0 = base address for display
    sw $t1, 0($t0)          # paint the 1st unit (i.e., top-left) red
    sw $t2, 4($t0)          # paint the 2nd unit on the 1st row green
    # Notice how the next unit uses adds on 4.
    sw $t2, 32($t0)	        # paint the 8th unit on the 1st row green
    
    
    # Notice how the 2nd row starts on 128 because 256 width pixels / 8 pixels per unit = 32 units
    sw $t3, 128($t0)        # paint the 1st unit on the 2nd row blue
    sw $t2, 256($t0)        # paint the 1st unit on the 3rd row green
    sw $t3, 384($t0)        # paint the 1st unit on the 4th row blue
    
    lw $t5, NUM             # save NUM (8) into t5
    add $t5, $t5, $t0       # t5 = t5 + t0 (8 + ADDR_DSPL)
    sw $t4, ($t5)           # paint the 3rd unit on the 1st row white
exit:
    li $v0, 10              # terminate the program gracefully
    syscall


```
