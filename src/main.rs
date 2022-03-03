// Memory map
// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM

// The graphics system: The chip 8 has one instruction that draws sprite to the screen.
// Drawing is done in XOR mode and if a pixel is turned off as a result of drawing,
// the VF register is set. This is used for collision detection.

fn main() {
    // The chip 8 has 35 opcodes, all are 2 bytes long
    let opcode: u16;

    // The chip 8 has 4K memory
    let memory: [u8; 4096];

    // The chip 8 has 15 8-bit general purpose registers named V0, V1 -> VE
    let v: [u8; 16];

    // Index register and program counter (which have values from 0x000 to 0xFFF)
    let i: u16;
    let pc: u16;

    // The graphics of the Chip 8 are black and white and the screen has a total of 2048 pixels (64 x 32)
    let gfx: [u8; 64 * 32];

    // Interrupts and hardware registers.
    // The Chip 8 has none, but there are two timer registers that count at 60 Hz. When set above zero they will count down to zero.
    let delay_timer: u8;

    // The system’s buzzer sounds whenever the sound timer reaches zero.
    let sound_timer: u8;

    // The stack is used to remember the current location before a jump is performed.
    // So anytime you perform a jump or call a subroutine, store the program counter in the stack before proceeding.
    // The system has 16 levels of stack and in order to remember which level of the stack is used
    let stack: [u16; 16];
    let sp: u16;

    // the Chip 8 has a HEX based keypad (0x0-0xF), an array store the current state of the key.
    let key: [u8; 16];

    // Set up render system and register input callbacks
    setup_graphics();
    setup_input();

    // Initialize the chip 8 system and load the game into the memory
    let chip8: Chip8;
    chip8.initialize();
    chip8.load_game("pong");

    loop { // Emulation loop
        chip8.emulate_cycle();

        if chip8.draw_flag { // If the draw flag is set, update the screen
            draw_graphics();
        }

        chip8.set_keys();
    }
}

pub(crate) struct Chip8 {
    draw_flag: bool
}

impl Chip8 {
    pub(crate) fn initialize(&self) {
        todo!()
    }
    pub(crate) fn load_game(&self, p0: &str) {
        todo!()
    }

    pub(crate) fn emulate_cycle(&self) {
        todo!()
    }

    pub(crate) fn set_keys(&self) {
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
