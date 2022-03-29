#[cfg(test)]
mod main_tests {
    use crate::Chip8;

    #[test]
    fn op_0x3xnn_skip_instruction_when_vx_equals_nn() {
        let mut chip8 = Chip8::default();
        let x = 0;
        let nn = 0x0F;
        chip8.v[x] = nn;

        chip8.op_0x3xnn(x, nn);

        assert_eq!(chip8.pc, 0x204);
    }

    #[test]
    fn op_0x3xnn_does_not_skip_instruction_when_vx_dont_equals_nn() {
        let mut chip8 = Chip8::default();
        let x = 0;
        chip8.v[x] = 0x00;

        chip8.op_0x3xnn(x, 0x0F);

        assert_eq!(chip8.pc, 0x202);
    }
}
