// Memory map
// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM

// The graphics system: The chip 8 has one instruction that draws sprite to the screen.
// Drawing is done in XOR mode and if a pixel is turned off as a result of drawing,
// the VF register is set. This is used for collision detection.

#[cfg(test)]
#[path = "./main_tests.rs"]
mod main_tests;

use std::fs::File;
use std::io;
use std::io::{Read};
use rand;
use crate::ProgramCounterInstruction::{GOTO, NEXT, SKIP};

fn main() -> io::Result<()> {
    // Set up render system and register input callbacks
    setup_graphics();
    setup_input();

    // Initialize the chip 8 system and load the game into the memory
    let mut chip8 = Chip8::default();
    chip8.load_game()?;

    loop { // Emulation loop
        chip8.emulate_cycle();

        if chip8.draw_flag { // If the draw flag is set, update the screen
            draw_graphics();
        }

        chip8.set_keys();
    }
}

struct Chip8 {
    // The chip 8 has 35 opcodes, all are 2 bytes long
    opcode: u16,
    // The chip 8 has 4K memory
    memory: [u8; 4096], // TODO Use vector instead : https://doc.rust-lang.org/std/vec/struct.Vec.html
    // The chip 8 has 15 8-bit general purpose registers named V0, V1 -> VE
    v: [u8; 16],
    // Index register and program counter (which have values from 0x000 to 0xFFF)
    i: u16,
    pc: u16,
    // The graphics of the Chip 8 are black and white and the screen has a total of 2048 pixels (64 x 32)
    gfx: [u8; 64 * 32],
    // Interrupts and hardware registers.
    // The Chip 8 has none, but there are two timer registers that count at 60 Hz. When set above zero they will count down to zero.
    delay_timer: u8,
    // The system’s buzzer sounds whenever the sound timer reaches zero.
    sound_timer: u8,
    // The stack is used to remember the current location before a jump is performed.
    // So anytime you perform a jump or call a subroutine, store the program counter in the stack before proceeding.
    // The system has 16 levels of stack
    stack: Vec<u16>,
    // the Chip 8 has a HEX based keypad (0x0-0xF), an array store the current state of the key.
    key: [u8; 16],
    draw_flag: bool,
}

impl Default for Chip8 {
    fn default() -> Chip8 {
        Chip8 {
            pc: 0x200,
            memory: [0; 4096],
            v: [0; 16],
            gfx: [0; 64 * 32],
            stack: Vec::with_capacity(16),
            key: [0; 16],
            opcode: 0,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            draw_flag: false,
        }
    }
}

impl Chip8 {
    fn load_game(&mut self) -> io::Result<()> {
        let mut file = File::open("pong.rom")?;
        let mut buffer: [u8; 246] = [0; 246];
        file.read(&mut buffer)?;
        for i in 0..buffer.len() {
            self.memory[i + 512] = buffer[i];
        }
        Ok(())
    }

    fn emulate_cycle(&mut self) {
        let opcode_first_byte = u16::from(self.memory[usize::from(self.pc)] << 8);
        let opcode_second_byte = u16::from(self.memory[usize::from(self.pc + 1)]);
        self.opcode = opcode_first_byte | opcode_second_byte;
        let nibbles = (
            (self.opcode & 0xF000) >> 12 as u8,
            (self.opcode & 0x0F00) >> 8 as u8,
            (self.opcode & 0x00F0) >> 4 as u8,
            (self.opcode & 0x000F) as u8
        );
        let nnn = (self.opcode & 0x0FFF) as u16;
        let nn = (self.opcode & 0x00FF) as u8;
        let n = (self.opcode & 0x000F) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;

        let program_counter_action = match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x000F { // TODO 0NNN Might be missing (it calls machine code routine at address NNN)
                0x0000 => self.op_0x00e0(),
                0x000E => self.op_0x00ee(),
                _ => panic!("Unknown opcode read : 0x{}", self.opcode)
            },
            0x1000 => self.op_0x1nnn(nnn),
            0x2000 => self.op_0x2nnn(nnn),
            0x3000 => self.op_0x3xnn(x, nn),
            0x4000 => self.op_0x4xnn(x, nn),
            0x5000 => self.op_0x5xy0(x, y),
            0x6000 => self.op_0x6xnn(x, nn),
            0x7000 => self.op_0x7xnn(x, nn),
            0x8000 => match n {
                0x0000 => self.op_0x8xy0(x, y),
                0x0001 => self.op_0x8xy1(x, y),
                0x0002 => self.op_0x8xy2(x, y),
                0x0003 => self.op_0x8xy3(x, y),
                0x0004 => self.op_0x8xy4(x, y),
                0x0005 => self.op_0x8xy5(x, y),
                0x0006 => self.op_0x8xy6(x),
                0x0007 => self.op_0x8xy7(x, y),
                0x000E => self.op_0x8xye(x),
                _ => panic!("Unknown opcode read : 0x{}", self.opcode)
            },
            0x9000 => self.op_0x9xy0(x, y),
            0xA000 => self.op_0xannn(nnn),
            0xB000 => self.op_0xbnnn(nnn),
            0xC000 => self.op_0xcxnn(x, nn),
            0xD000 => self.op_0xdxyn(x, y, nn),
            0xE000 => match n {
                0x000E => self.op_0xex9e(x),
                0x0001 => self.op_0xexa1(x),
                _ => panic!("Unknown opcode read : 0x{}", self.opcode)
            },
            _ => panic!("Unknown opcode read : 0x{}", self.opcode)
        };

        match program_counter_action {
            NEXT => self.pc += 2,
            SKIP => self.pc += 4,
            GOTO(addr) => self.pc = addr
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP");
            }
            self.sound_timer -= 1;
        }
    }

    //00E0: Clears the screen
    fn op_0x00e0(&self) -> ProgramCounterInstruction {
        self.clear_screen();
        NEXT
    }
    
    //00EE: Returns from subroutine
    fn op_0x00ee(&mut self) -> ProgramCounterInstruction {
        match self.stack.pop() {
            Some(previous_pc) => GOTO(previous_pc),
            None => panic!("Error: trying to pop the stack but it is empty"),
        }
    }

    //1NNN: Jumps to address NNN
    fn op_0x1nnn(&self, nnn: u16) -> ProgramCounterInstruction {
        GOTO(nnn)
    }

    //2NNN: Calls subroutine at NNN
    fn op_0x2nnn(&mut self, nnn: u16) -> ProgramCounterInstruction {
        self.stack.push(self.pc);
        GOTO(nnn)
    }

    //3XNN: Skips the next instruction if VX equals NN (Usually the next instruction ia a jump to skip a code block)
    fn op_0x3xnn(&self, x: usize, nn: u8) -> ProgramCounterInstruction {
        if self.v[x] == nn {
            SKIP
        } else {
            NEXT
        }
    }

    //4XNN: Skips the next instruction if VX does not equals NN (Usually the next instruction ia a jump to skip a code block)
    fn op_0x4xnn(&self, x: usize, nn: u8) -> ProgramCounterInstruction {
        if self.v[x] != nn {
            SKIP
        } else {
            NEXT
        }
    }

    //5XY0: Skips the next instruction if VX equals VY (Usually the next instruction ia a jump to skip a code block)
    fn op_0x5xy0(&self, x: usize, y: usize) -> ProgramCounterInstruction {
        return if self.v[x] == self.v[y] {
            SKIP
        } else {
            NEXT
        }
    }

    //6XNN: Sets VX to NN
    fn op_0x6xnn(&mut self, x: usize, nn: u8) -> ProgramCounterInstruction {
        self.v[x] = nn;
        NEXT
    }

    //7XNN: Adds NN to VX
    fn op_0x7xnn(&mut self, x: usize, nn: u8) -> ProgramCounterInstruction {
        let addend = self.v[x] as u16;
        let augend = nn as u16;
        self.v[x] = (augend + addend) as u8;
        NEXT
    }

    //8XY0: Sets VX to the value of VY
    fn op_0x8xy0(&mut self, x: usize, y: usize) -> ProgramCounterInstruction {
        self.v[x] = self.v[y];
        NEXT
    }

    //8XY1: Set VX to VX or VY (Bitwise OR operation)
    fn op_0x8xy1(&mut self, x: usize, y: usize) -> ProgramCounterInstruction {
        self.v[x] |= self.v[y];
        NEXT
    }

    //8XY2: Set VX to VX and VY (Bitwise AND operation)
    fn op_0x8xy2(&mut self, x: usize, y: usize) -> ProgramCounterInstruction {
        self.v[x] &= self.v[y];
        NEXT
    }

    //8XY3: Set VX to VX xor VY
    fn op_0x8xy3(&mut self, x: usize, y: usize) -> ProgramCounterInstruction {
        self.v[x] ^= self.v[y];
        NEXT
    }

    //8XY4: Adds VY to VX. VF is set to 1 when there's a carry and to 0 when there is not
    fn op_0x8xy4(&mut self, x: usize, y: usize) -> ProgramCounterInstruction {
        let result = (self.v[x] as u16) + (self.v[y] as u16);
        self.v[x] = result as u8;
        self.v[0x0F] = if result > 0xFF { 1 } else { 0 };
        NEXT
    }

    //8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_0x8xy5(&mut self, x: usize, y: usize) -> ProgramCounterInstruction {
        let result = self.v[x].wrapping_sub(self.v[y]);
        self.v[0x0F] = if self.v[x] > self.v[y] { 1 } else { 0 };
        self.v[x] = result as u8;
        NEXT
    }

    //8XY6: Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
    fn op_0x8xy6(&mut self, x: usize) -> ProgramCounterInstruction {
        self.v[0x0F] = self.v[x] & 0x1;
        self.v[x] >>= 1;
        NEXT
    }

    //8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_0x8xy7(&mut self, x: usize, y: usize) -> ProgramCounterInstruction {
        self.v[0x0F] = if self.v[y] > self.v[x] { 1 } else { 0 };
        let result = self.v[y].wrapping_sub(self.v[x]);
        self.v[x] = result as u8;
        NEXT
    }

    //8XYE: Stores the most significant bit of VX in VF and then shifts VX to the left by 1
    fn op_0x8xye(&mut self, x: usize) -> ProgramCounterInstruction {
        self.v[0x0F] = (self.v[x] & 0b1000_0000) >> 7;
        self.v[x] <<= 1;
        NEXT
    }

    //9XY0: Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block)
    fn op_0x9xy0(&self, x: usize, y: usize) -> ProgramCounterInstruction {
        if self.v[x] != self.v[y] {
            SKIP
        } else {
            NEXT
        }
    }

    //ANNN: Sets i to the address NNN
    fn op_0xannn(&mut self, nnn: u16) -> ProgramCounterInstruction {
        self.i = nnn;
        NEXT
    }

    //BNNN: Jumps to the address NNN plus V0
    fn op_0xbnnn(&mut self, nnn: u16) -> ProgramCounterInstruction {
        GOTO(u16::from(self.v[0]) + nnn)
    }

    //CXNN: Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN
    fn op_0xcxnn(&mut self, x: usize, nn: u8) -> ProgramCounterInstruction {
        let random_u8: u8 = rand::random();
        self.v[x] = random_u8 & nn;
        NEXT
    }

    //DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    // Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after
    // the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped
    // from set to unset when the sprite is drawn, and to 0 if that does not happen
    fn op_0xdxyn(&self, x: usize, y: usize, n: u8) -> ProgramCounterInstruction { //TODO : Test
        self.draw(self.v[x], self.v[y], n);
        NEXT
    }

    //EX9E: Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
    fn op_0xex9e(&self, x: usize) -> ProgramCounterInstruction { //TODO : Test
        if self.key_pressed() == self.v[x] {
            SKIP
        } else {
            NEXT
        }
    }

    //EXA1: Skips the next instruction if the key stored in VX is not pressed. (Usually the next instruction is a jump to skip a code block)
    fn op_0xexa1(&self, x: usize) -> ProgramCounterInstruction {
        if self.key_pressed() != self.v[x] { //TODO : Test
            SKIP
        } else {
            NEXT
        }
    }

    fn set_keys(&self) {
        todo!()
    }
    fn clear_screen(&self) {
        todo!()
    }
    fn draw(&self, vx: u8, vy: u8, n: u8) {
        todo!()
    }
    fn key_pressed(&self) -> u8 {
        todo!()
    }
}

enum ProgramCounterInstruction {
    NEXT,
    SKIP,
    GOTO(u16)
}

fn setup_graphics() {
    todo!()
}

fn setup_input() {
    todo!()
}

fn draw_graphics() {
    todo!()
}
