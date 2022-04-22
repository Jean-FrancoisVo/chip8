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
            pc: 0x200, // TODO: Create struct and impl, will have next, jump and nothing, might be interesting to force emulation cycle to return a PC
                       // TODO: as every instruction must have an impact on PC
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

        // TODO somehow we should expect every function to return a program counter (pc)
        let program_counter_action = match self.opcode & 0xF000 {
            // 0x0000 => self.ops_0x0000()
            //1NNN: Jumps to address NNN
            0x1000 => self.op_0x1nnn(nnn),
            //2NNN: Calls subroutine at NNN
            0x2000 => self.op_0x2nnn(nnn),
            //3XNN: Skips the next instruction if VX equals NN (Usually the next instruction ia a jump to skip a code block)
            0x3000 => self.op_0x3xnn(x, nn),
            //4XNN: Skips the next instruction if VX does not equals NN (Usually the next instruction ia a jump to skip a code block)
            0x4000 => self.op_0x4xnn(x, nn),
            //5XY0: Skips the next instruction if VX equals VY (Usually the next instruction ia a jump to skip a code block)
            0x5000 => self.op_0x5xy0(x, y),
            //6XNN: Sets VX to NN
            0x6000 => self.op_0x6xnn(x, nn),
            //7XNN: Adds NN to VX
            0x7000 => self.op_0x7xnn(x, nn),
            0x8000 => self.ops_0x8000(x, y, n),
            //ANNN: Sets i to the address NNN
            0xA000 => self.op_0xAnnn(nnn),
            // TODO Others opcodes
            _ => panic!("Unknown opcode read : 0x{}", self.opcode)
        };

        match program_counter_action {
            ProgramCounterAction::NEXT => self.pc += 2,
            ProgramCounterAction::SKIP => self.pc += 4,
            ProgramCounterAction::GOTO(addr) => self.pc = addr
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

    // fn ops_0x0000(&mut self) {
    //     match self.opcode & 0x000F { // TODO 0NNN Might be missing (it calls machine code routine at address NNN)
    //         // 0x0000 => todo!("0x00E0 Clear the screen"),
    //         //00EE: Returns from subroutine
    //         0x000E => {
    //             match self.stack.pop() {
    //                 None => panic!("Error: trying to pop the stack but it is empty"),
    //                 Some(previous_pc) => self.pc = previous_pc
    //             }
    //         },
    //         _ => panic!("Unknown opcode read : 0x{}", self.opcode)
    //     }
    // }

    fn op_0x1nnn(&mut self, nnn: u16) -> ProgramCounterAction {
        return ProgramCounterAction::GOTO(nnn);
    }

    fn op_0x2nnn(&mut self, nnn: u16) -> ProgramCounterAction {
        self.stack.push(self.pc);
        return ProgramCounterAction::GOTO(nnn);
    }

    fn op_0x3xnn(&mut self, x: usize, nn: u8) -> ProgramCounterAction {
        return if self.v[x] == nn {
            ProgramCounterAction::SKIP
        } else {
            ProgramCounterAction::NEXT
        }
    }

    fn op_0x4xnn(&mut self, x: usize, nn: u8) -> ProgramCounterAction {
        return if self.v[x] != nn {
            ProgramCounterAction::SKIP
        } else {
            ProgramCounterAction::NEXT
        }
    }

    fn op_0x5xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        return if self.v[x] == self.v[y] {
            ProgramCounterAction::SKIP
        } else {
            ProgramCounterAction::NEXT
        }
    }

    fn op_0x6xnn(&mut self, x: usize, nn: u8) -> ProgramCounterAction {
        self.v[x] = nn;
        return ProgramCounterAction::NEXT;
    }

    fn op_0x7xnn(&mut self, x: usize, nn: u8) -> ProgramCounterAction {
        let addend = self.v[x] as u16;
        let augend = nn as u16;
        self.v[x] = (augend + addend) as u8;
        return ProgramCounterAction::NEXT;
    }

    fn ops_0x8000(&mut self, x: usize, y: usize, n: u8) -> ProgramCounterAction {
        return match n {
            //8XY0: Sets VX to the value of VY
            0x0000 => self.op_0x8xy0(x, y),
            //8XY1: Set VX to VX or VY (Bitwise OR operation)
            0x0001 => self.op_0x8xy1(x, y),
            //8XY2: Set VX to VX and VY (Bitwise AND operation)
            0x0002 => self.op_0x8xy2(x, y),
            //8XY3: Set VX to VX xor VY
            0x0003 => self.op_0x8xy3(x, y),
            //8XY4: Adds VY to VX. VF is set to 1 when there's a carry and to 0 when there is not
            0x0004 => self.op_0x8xy4(x, y),
            _ => panic!("Unknown opcode read : 0x{}", self.opcode)
        }
    }

    fn op_0x8xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        self.v[x] = self.v[y];
        return ProgramCounterAction::NEXT;
    }

    fn op_0x8xy1(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        self.v[x] |= self.v[y];
        return ProgramCounterAction::NEXT;
    }

    fn op_0x8xy2(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        self.v[x] &= self.v[y];
        return ProgramCounterAction::NEXT;
    }

    fn op_0x8xy3(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        self.v[x] ^= self.v[y];
        return ProgramCounterAction::NEXT;
    }

    fn op_0x8xy4(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        let result = (self.v[x] as u16) + (self.v[y] as u16);
        self.v[x] = result as u8;
        self.v[0x0F] = if result > 0xFF { 1 } else { 0 };
        return ProgramCounterAction::NEXT;
    }

    fn op_0xAnnn(&mut self, nnn: u16) -> ProgramCounterAction {
        self.i = nnn;
        return ProgramCounterAction::NEXT;
    }

    fn set_keys(&self) {
        todo!()
    }
}

enum ProgramCounterAction {
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
