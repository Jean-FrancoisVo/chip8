// Memory map
// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM

// The graphics system: The chip 8 has one instruction that draws sprite to the screen.
// Drawing is done in XOR mode and if a pixel is turned off as a result of drawing,
// the VF register is set. This is used for collision detection.

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
    // The system has 16 levels of stack and in order to remember which level of the stack is used
    stack: [u16; 16],
    sp: u16,
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
            stack: [0; 16],
            key: [0; 16],
            opcode: 0,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            sp: 0,
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
        let first_byte = u16::from(self.memory[usize::from(self.pc)] << 8);
        let second_byte = u16::from(self.memory[usize::from(self.pc + 1)]);
        self.opcode = first_byte | second_byte;

        match self.opcode & 0xF000 {
            // TODO Others opcodes
            0xA000 => { // ANNN: Sets i to the address NNN
                self.i = self.opcode & 0x0FFF;
                self.pc += 2;
            }
            0x0000 => {
                match self.opcode & 0x000F {
                    0x0000 => todo!("0x00E0 Clear the screen"),
                    0x000E => todo!("0x00EE Returns from subroutine"),
                    _ => panic!("Unknown opcode read : 0x{}", self.opcode)
                }
            }
            _ => panic!("Unknown opcode read : 0x{}", self.opcode)
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

    fn set_keys(&self) {
        todo!()
    }
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
