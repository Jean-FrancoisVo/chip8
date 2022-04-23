#[cfg(test)]
mod main_tests {
    use crate::Chip8;
    use crate::ProgramCounterInstruction::GOTO;
    use crate::ProgramCounterInstruction::NEXT;
    use crate::ProgramCounterInstruction::SKIP;

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

    #[test]
    fn op_0x8xy5_subtract_vy_to_vx_without_borrow_flag() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0xFF;
        chip8.v[y] = 0x01;

        let result = chip8.op_0x8xy5(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0xFE);
        assert_eq!(chip8.v[0x0F], 1);
    }

    #[test]
    fn op_0x8xy5_subtract_vy_to_vx_with_borrow_flag() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0x00;
        chip8.v[y] = 0x01;

        let result = chip8.op_0x8xy5(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0xFF);
        assert_eq!(chip8.v[0x0F], 0);
    }

    #[test]
    fn op_0x8xy6_shift_right_vx_by_1_and_store_the_least_significant_bit_in_vf() {
        let mut chip8 = Chip8::default();
        let x = 1;
        chip8.v[x] = 0x03;

        let result = chip8.op_0x8xy6(x);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0x01);
        assert_eq!(chip8.v[0x0F], 1);
    }

    #[test]
    fn op_0x8xy7_subtract_vx_to_vy_and_store_in_vx_without_borrow_flag() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0x01;
        chip8.v[y] = 0x02;

        let result = chip8.op_0x8xy7(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0x01);
        assert_eq!(chip8.v[0x0F], 1);
    }

    #[test]
    fn op_0x8xy7_subtract_vx_to_vy_and_store_in_vx_with_borrow_flag() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0x02;
        chip8.v[y] = 0x01;

        let result = chip8.op_0x8xy7(x, y);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0xFF);
        assert_eq!(chip8.v[0x0F], 0);
    }

    #[test]
    fn op_0x8xye_shift_left_vx_by_1_and_store_the_most_significant_bit_in_vf() {
        let mut chip8 = Chip8::default();
        let x = 1;
        chip8.v[x] = 0xF0;

        let result = chip8.op_0x8xye(x);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.v[x], 0xE0);
        assert_eq!(chip8.v[0x0F], 1);
    }

    #[test]
    fn op_0x9xy0_skip_when_vx_is_different_from_vy() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0xF0;
        chip8.v[y] = 0x01;

        let result = chip8.op_0x9xy0(x, y);

        assert!(matches!(result, SKIP));
    }

    #[test]
    fn op_0x9xy0_next_when_vx_is_equal_from_vy() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let y = 2;
        chip8.v[x] = 0x01;
        chip8.v[y] = 0x01;

        let result = chip8.op_0x9xy0(x, y);

        assert!(matches!(result, NEXT));
    }

    #[test]
    fn op_0xannn_sets_i_to_nnn() {
        let mut chip8 = Chip8::default();
        let nnn: u16 = 0x55;

        let result = chip8.op_0xannn(nnn);

        assert!(matches!(result, NEXT));
        assert_eq!(chip8.i, nnn)
    }

    #[test]
    fn op_0xbnnn_jumps_to_nnn_plus_v0() {
        let mut chip8 = Chip8::default();
        chip8.v[0] = 1;
        let nnn: u16 = 0x55;
        let final_address = nnn + u16::from(chip8.v[0]);

        let result = chip8.op_0xbnnn(nnn);

        assert!(matches!(result, GOTO(final_address)));
    }

    #[test]
    fn op_0xcxnn_return_next_and_set_vx_to_random() {
        let mut chip8 = Chip8::default();
        let x = 1;
        let nn: u8 = 0xFF;

        let result = chip8.op_0xcxnn(x, nn);

        assert!(matches!(result, NEXT));
    }
}
