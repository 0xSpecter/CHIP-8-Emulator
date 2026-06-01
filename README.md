# CHIP-8 Emulator

A CHIP-8 emulator written fully in rust using the crate ```pixels``` for rendering

## Compiler
Includes a simple Mnemonic to opcode compiler for writing roms

To compile use this command
```
./compiler in.txt out.ch8
```
### Instruction Set

| Mnemonic | Operands | Opcode | Description |
|----------|----------|--------|-------------|
| `cls`    |          | 00E0   | Clear the display |
| `ret`    |          | 00EE   | Return from subroutine |
| `jp`     | nnn      | 1nnn   | Jump to address |
| `call`   | nnn      | 2nnn   | Call subroutine at address |
| `se`     | x kk     | 3xkk   | Skip if Vx == byte |
| `sne`    | x kk     | 4xkk   | Skip if Vx != byte |
| `sev`    | x y      | 5xy0   | Skip if Vx == Vy |
| `ld`     | x kk     | 6xkk   | Set Vx = byte |
| `add`    | x kk     | 7xkk   | Set Vx = Vx + byte |
| `mov`    | x y      | 8xy0   | Set Vx = Vy |
| `or`     | x y      | 8xy1   | Set Vx = Vx OR Vy |
| `and`    | x y      | 8xy2   | Set Vx = Vx AND Vy |
| `xor`    | x y      | 8xy3   | Set Vx = Vx XOR Vy |
| `addr`   | x y      | 8xy4   | Set Vx = Vx + Vy, VF = carry |
| `sub`    | x y      | 8xy5   | Set Vx = Vx - Vy, VF = borrow |
| `shr`    | x y      | 8xy6   | Set Vx = Vx >> 1, VF = LSB |
| `subn`   | x y      | 8xy7   | Set Vx = Vy - Vx, VF = borrow |
| `shl`    | x y      | 8xyE   | Set Vx = Vx << 1, VF = MSB |
| `snev`   | x y      | 9xy0   | Skip if Vx != Vy |
| `ldi`    | nnn      | Annn   | Set I = address |
| `jpv`    | nnn      | Bnnn   | Jump to V0 + address |
| `rnd`    | x kk     | Cxkk   | Set Vx = random byte AND kk |
| `drw`    | x y n    | Dxyn   | Draw sprite at (Vx, Vy), height n |
| `skp`    | x        | Ex9E   | Skip if key Vx is pressed |
| `sknp`   | x        | ExA1   | Skip if key Vx is not pressed |
| `gdt`    | x        | Fx07   | Set Vx = delay timer |
| `key`    | x        | Fx0A   | Wait for key press, store in Vx |
| `sdt`    | x        | Fx15   | Set delay timer = Vx |
| `sst`    | x        | Fx18   | Set sound timer = Vx |
| `addi`   | x        | Fx1E   | Set I = I + Vx |
| `font`   | x        | Fx29   | Set I = sprite address for digit Vx |
| `bcd`    | x        | Fx33   | Store BCD of Vx at I, I+1, I+2 |
| `stor`   | x        | Fx55   | Store V0..Vx in memory at I |
| `read`   | x        | Fx65   | Read V0..Vx from memory at I |

#### Example
```
jp 0xf23
addr 1 5
key 2
```

### References
* [CHIP-8 Walkthrough](https://austinmorlan.com/posts/chip8_emulator/)
* [CHIP-8 Specs](https://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf)
* [CHIP-8 Specs 2](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
