#[cfg(test)]
mod main_tests {
    use crate::Chip8;
    use crate::ProgramCounterAction::GOTO;
    use crate::ProgramCounterAction::NEXT;
    use crate::ProgramCounterAction::SKIP;

    #[test]
    fn op_0x1nnn_jumps_to_address_nnn() {
        let mut chip8 = Chip8::default();
        let nnn = 0xFFF;

        let result = chip8.op_0x1nnn(nnn);

        assert!(matches!(result, GOTO(nnn)));
    }

    #[test]
    fn op_0x2nnn_call_subroutine_at_nnn() {
        let mut chip8 = Chip8::default();
        let nnn = 0xFFF;

        let result = chip8.op_0x2nnn(nnn);

        assert!(matches!(result, GOTO(nnn)));
        assert_eq!(*chip8.stack.last().unwrap(), 0x200 as u16);
    }

    #[test]
    fn op_0x3xnn_skip_instruction_when_vx_equals_nn() {
        let mut chip8 = Chip8::default();
        let x = 0;
        let nn = 0x0F;
        chip8.v[x] = nn;

        let result = chip8.op_0x3xnn(x, nn);

        assert!(matches!(result, SKIP));
    }

    #[test]
    fn op_0x3xnn_does_not_skip_instruction_when_vx_dont_equals_nn() {
        let mut chip8 = Chip8::default();
        let x = 0;
        let nn = 0x0F;
        chip8.v[x] = 0x00;

        let result = chip8.op_0x3xnn(x, nn);

        assert!(matches!(result, NEXT));
    }

    #[test]
    fn op_0x4xnn_skip_instruction_when_vx_dont_equals_nn() {
        let mut chip8 = Chip8::default();
        let x = 0;
        let nn = 0x0F;
        chip8.v[x] = 0xCC;

        let result = chip8.op_0x4xnn(x, nn);

        assert!(matches!(result, SKIP));
    }

    #[test]
    fn op_0x4xnn_does_not_skip_instruction_when_vx_equals_nn() {
        let mut chip8 = Chip8::default();
        let x = 0;
        let nn = 0x0F;
        chip8.v[x] = nn;

        let result = chip8.op_0x4xnn(x, nn);

        assert!(matches!(result, NEXT));
    }

    #[test]
    fn op_0x5xy0_skip_instruction_when_vx_equals_vy() {
        let mut chip8 = Chip8::default();
        let x = 0;
        let y = 1;
        chip8.v[x] = 0xA;
        chip8.v[y] = 0xA;

        let result = chip8.op_0x5xy0(x, y);

        assert!(matches!(result, SKIP));
    }

    #[test]
    fn op_0x5xy0_does_not_skip_instruction_when_vx_dont_equals_vy() {
        let mut chip8 = Chip8::default();
        let x = 0;
        let y = 1;
        chip8.v[x] = 0xA;
        chip8.v[y] = 0xB;

        let result = chip8.op_0x5xy0(x, y);

        assert!(matches!(result, NEXT));
    }

    #[test]
    fn op_0x6xnn_sets_vx_to_nn() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let nn = 0xC;

        let result = chip8.op_0x6xnn(x, nn);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], nn);
    }

    #[test]
    fn op_0x7xnn_adds_nn_to_vx() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let nn = 0xC;
        chip8.v[x] = 0x1;

        let result = chip8.op_0x7xnn(x, nn);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0xD);
    }

    #[test]
    fn op_0x7xnn_adds_nn_to_vx_does_not_change_carry_flag() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let nn = 0xFF;
        chip8.v[x] = 0x1;

        let result = chip8.op_0x7xnn(x, nn);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0x00);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    fn op_0x8xy0_sets_vx_to_vy() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0x00;
        chip8.v[y] = 0xFF;

        let result = chip8.op_0x8xy0(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0xFF);
    }

    #[test]
    fn op_0x8xy1_sets_vx_to_vx_or_vy() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0xA0;
        chip8.v[y] = 0x0A;

        let result = chip8.op_0x8xy1(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0xAA);
    }

    #[test]
    fn op_0x8xy2_sets_vx_to_vx_and_vy() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0xA0;
        chip8.v[y] = 0x0A;

        let result = chip8.op_0x8xy2(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0x00);
    }

    #[test]
    fn op_0x8xy3_sets_vx_to_vx_xor_vy() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0xA0;
        chip8.v[y] = 0xAA;

        let result = chip8.op_0x8xy3(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0x0A);
    }

    #[test]
    fn op_0x8xy4_adds_vx_to_vy_without_carry_flag() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0x01;
        chip8.v[y] = 0x01;

        let result = chip8.op_0x8xy4(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0x02);
        assert_eq!(chip8.v[0x0F], 0);
    }

    #[test]
    fn op_0x8xy4_adds_vx_to_vy_with_carry_flag() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0xFF;
        chip8.v[y] = 0x01;

        let result = chip8.op_0x8xy4(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0x00);
        assert_eq!(chip8.v[0x0F], 1);
    }
}
