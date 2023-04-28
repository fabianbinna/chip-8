use wasm_bindgen::prelude::*;
use rand::prelude::*;
use crate::utils::set_panic_hook;
extern crate web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

const PC_START: usize = 0x200;
const RAM_SIZE_BYTE: usize = 4096;
const FONT_START: usize = 0x50;
const VRAM_START: usize = 0xf00;
const VRAM_SIZE_BYTE: usize = 256;
const VF: usize = 0xf;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_WIDTH: usize = 64;

#[wasm_bindgen]
pub struct Processor {
    ram: [u8; RAM_SIZE_BYTE],
    stack: [usize; 12],
    v: [u8; 16],
    pc: usize,
    sp: usize,
    i: usize,
    delay_timer: u8,
    time: instant::Instant,
    sound_timer: u8,
    wait_key: bool,
    wait_key_reg: usize,
    pub halt: bool,
    key_state: [bool; 16]
}

#[wasm_bindgen]
impl Processor {

    pub fn new(rom: Vec<u8>) -> Self {
        set_panic_hook();

        // Load rom
        let mut ram = [0 as u8; RAM_SIZE_BYTE];
        for i in 0..rom.len() {
            ram[PC_START + i] = rom[i];
        }

        // Load font set
        let font_set: Vec<u8> = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];
        for i in 0..font_set.len() {
            ram[FONT_START + i] = font_set[i];
        }

        Processor{
            ram,
            stack: [0 as usize; 12],
            v: [0 as u8; 16],
            pc: PC_START,
            sp: 0,
            i: 0,
            delay_timer: 0,
            time: instant::Instant::now(),
            sound_timer: 0,
            wait_key: false,
            wait_key_reg: 0,
            halt: false,
            key_state: [false; 16]
        }
    }

    pub fn key_pressed(&mut self, key: usize) {
        self.key_state[key] = true;
    }

    pub fn key_released(&mut self, key: usize) {
        self.key_state[key] = false;
    }

    pub fn screen(&self) -> *const u8{
        self.ram[VRAM_START..VRAM_START+VRAM_SIZE_BYTE].as_ptr()
    }

    pub fn tick(&mut self) {
        if self.wait_key {
            for i in 0..self.key_state.len() {
                if self.key_state[i] {
                    self.v[self.wait_key_reg] = i as u8;
                    self.wait_key = false;
                    self.pc+=2;
                }
            }
        }

        if self.wait_key {
            return;
        }

        let elapsed = self.time.elapsed();
        if self.delay_timer > 0 && elapsed.as_millis() >= 1000/60 {
            self.delay_timer -= 1;
            self.time = instant::Instant::now();
        }

        let opcode = self.read_16_bit(self.pc);
        self.execute_opcode(opcode);
    }

    fn read_16_bit(&self, pointer: usize) -> usize {
        let left = self.ram[pointer] as u16;
        let right = self.ram[pointer + 1] as u16;
        (left << 8 | right).into()
    }

    // fn write_16_bit(&mut self, pointer: usize, value: usize) {
    //     self.ram[pointer] = (value >> 8) as u8;
    //     self.ram[pointer + 1] = value as u8;
    // }

    fn execute_opcode(&mut self, opcode: usize) {
        //log!("Running opcode: 0x{opcode:0>4x}");
        match opcode >> 12 {
            0x0 => self.op_0(opcode),
            0x1 => self.op_1(opcode),
            0x2 => self.op_2(opcode),
            0x3 => self.op_3(opcode),
            0x4 => self.op_4(opcode),
            0x5 => self.op_5(opcode),
            0x6 => self.op_6(opcode),
            0x7 => self.op_7(opcode),
            0x8 => self.op_8(opcode),
            0x9 => self.op_9(opcode),
            0xa => self.op_a(opcode),
            0xb => self.op_b(opcode),
            0xc => self.op_c(opcode),
            0xd => self.op_d(opcode),
            0xe => self.op_e(opcode),
            0xf => self.op_f(opcode),
            _ => println!("Unknown opcode: 0x{opcode:0>4x}")
        }
    }

    fn op_0(&mut self, opcode: usize) {
        match opcode {
            // Clears the screen.
            0xe0 => {
                for i in 0..VRAM_SIZE_BYTE {
                    self.ram[i + VRAM_START] = 0;
                }
            },
            // Returns from a subroutine.
            0xee => {
                self.sp-=1;
                self.pc = self.stack[self.sp];
            },
            _ => println!("Unknown opcode: 0x{opcode:0>4x}")
        }
        self.pc += 2;
    }

    // 1NNN. Jumps to address NNN.
    fn op_1(&mut self, opcode: usize) {
        let address = opcode & 0x0fff;
        // halt processor if loop detected.
        if address == self.pc {
            self.halt = true;
            log!("Processor halted.");
        }
        self.pc = address;
    }

    // 2NNN. Calls subroutine at NNN.
    fn op_2(&mut self, opcode: usize) {
        let address = opcode & 0x0fff;
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = address;
    }

    // 3XNN. Skips the next instruction if VX equals NN.
    // Usually the next instruction is a jump to skip a code block.
    fn op_3(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let value = opcode & 0x00ff;
        if self.v[reg_x] == value as u8 {
            self.pc+=2;
        }
        self.pc+=2;
    }

    // 4XNN. Skips the next instruction if VX does not equal NN.
    // Usually the next instruction is a jump to skip a code block.
    fn op_4(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let value = opcode & 0x00ff;
        if self.v[reg_x] != value as u8 {
            self.pc+=2;
        }
        self.pc+=2;
    }

    // 5XY0. Skips the next instruction if VX equals VY.
    // Usually the next instruction is a jump to skip a code block.
    fn op_5(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let reg_y = (opcode & 0x00f0) >> 4;
        if self.v[reg_x] == self.v[reg_y] {
            self.pc+=2;
        }
        self.pc+=2;
    }

    // 6XNN. Sets VX to NN.
    fn op_6(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let value = opcode & 0x00ff;
        self.v[reg_x] = value as u8;
        self.pc+=2;
    }

    // 7XNN. Adds NN to VX. (Carry flag is not changed).
    fn op_7(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let value = opcode & 0x00ff;
        self.v[reg_x] += value as u8;
        self.pc+=2;
    }

    fn op_8(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let reg_y = (opcode & 0x00f0) >> 4;
        match opcode & 0x000f {
            // 8XY0. Sets VX to the value of VY.
            0x0 => self.v[reg_x] = self.v[reg_y],
            // 8XY1. Sets VX to VX or VY. (Bitwise OR operation).
            0x1 => self.v[reg_x] |= self.v[reg_y],
            // 8XY2. Sets VX to VX and VY. (Bitwise AND operation).
            0x2 => self.v[reg_x] &= self.v[reg_y],
            // 8XY3. Sets VX to VX xor VY.
            0x3 => self.v[reg_x] ^= self.v[reg_y],
            // 8XY4. Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.
            0x4 => {
                let (result, carry) = self.v[reg_x].overflowing_add(self.v[reg_y]);
                self.v[reg_x] = result;
                self.v[VF] = carry as u8;
            },
            // 8XY5. VY is subtracted from VX.
            // VF is set to 0 when there's a borrow, and 1 when there is not.
            0x5 => {
                let (result, borrow) = self.v[reg_x].overflowing_sub(self.v[reg_y]);
                self.v[reg_x] = result;
                self.v[VF] = !borrow as u8;
            },
            // 8XY6. Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
            0x6 => {
                self.v[VF] = self.v[reg_x] & 0x1;
                self.v[reg_x] >>= 1;
            },
            // 8XY7. Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
            0x7 => {
                let (result, borrow) = self.v[reg_y].overflowing_sub(self.v[reg_x]);
                self.v[reg_x] = result;
                self.v[VF] = !borrow as u8;
            },
            // 8XYE. Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
            0xe => {
                self.v[VF] = self.v[reg_x] >> 7;
                self.v[reg_x] = self.v[reg_x] << 1;
            },
            _ => println!("Unknown opcode: 0x{opcode:0>4x}")
        }
        self.pc+=2;
    }

    // 9XY0. Skips the next instruction if VX does not equal VY.
    // Usually the next instruction is a jump to skip a code block.
    fn op_9(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let reg_y = (opcode & 0x00f0) >> 4;
        if self.v[reg_x] != self.v[reg_y] {
            self.pc+=2;
        }
        self.pc+=2;
    }

    // ANNN. Sets I to the address NNN.
    fn op_a(&mut self, opcode: usize) {
        self.i = opcode & 0x0fff;
        self.pc+=2;
    }

    // BNNN. Jumps to the address NNN plus V0.
    fn op_b(&mut self, opcode: usize) {
        self.pc = (opcode & 0x0fff) + self.v[0] as usize;
    }

    // CXNN. Sets VX to the result of a bitwise and operation on a random number 
    // (Typically: 0 to 255) and NN.
    fn op_c(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let value = opcode & 0x00ff;
        self.v[reg_x] = (rand::thread_rng().gen_range(0..255) & value) as u8;
        self.pc+=2;
    }

    // DXYN. Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    // Each row of 8 pixels is read as bit-coded starting from memory location I;
    // I value does not change after the execution of this instruction. As described above,
    // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
    // and to 0 if that does not happen
    fn op_d(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        let reg_y = (opcode & 0x00f0) >> 4;
        let height = opcode & 0x000f;
        self.v[VF] = 0;

        for byte in 0..height {
            let y = (self.v[reg_y] as usize + byte) % SCREEN_HEIGHT;
            for bit in 0..8 {
                let color = (self.ram[self.i + byte] & 2_i32.pow(7-bit) as u8) >= 1;
                let x = (self.v[reg_x] + bit as u8) as usize % SCREEN_WIDTH;
                let collision = self.set_pixel(x, y, color);
                if collision {
                    self.v[VF] = 1;
                }
            }
        }
        self.pc+=2;
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: bool) -> bool {
        let address = VRAM_START + y * 8 + x / 8;
        let data = self.ram[address];
        let bit = 7 - (x % 8);
        self.ram[address] = data ^ ((color as u8) << bit);
        ((data & 2_i32.pow(bit as u32) as u8) >= 1) && color
    }

    fn op_e(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        match opcode & 0xff {
            // EX9E. Skips the next instruction if the key stored in VX is pressed.
            // Usually the next instruction is a jump to skip a code block.
            0x9E => {
                if self.key_state[self.v[reg_x] as usize] {
                    self.pc += 2;
                }
            },
            // EXA1. Skips the next instruction if the key stored in VX is not pressed.
            // Usually the next instruction is a jump to skip a code block.
            0xA1 => {
                if !self.key_state[self.v[reg_x] as usize] {
                    self.pc += 2;
                }
            },
            _ => println!("Unknown opcode: 0x{opcode:0>4x}")
        }
        self.pc += 2;
    }

    fn op_f(&mut self, opcode: usize) {
        let reg_x = (opcode & 0x0f00) >> 8;
        match opcode & 0xff {
            // Sets VX to the value of the delay timer. 
            0x07 => self.v[reg_x] = self.delay_timer,
            // A key press is awaited, and then stored in VX 
            //(blocking operation, all instruction halted until next key event). 
            0x0A => {
                self.wait_key = true;
                self.wait_key_reg = reg_x;
                return;
            },
            // FX15. Sets the delay timer to VX.
            0x15 => self.delay_timer = self.v[reg_x],
            // FX18. Sets the sound timer to VX.
            0x18 => self.sound_timer = self.v[reg_x],
            // FX1E. Adds VX to I. VF is not affected.
            0x1E => self.i += self.v[reg_x] as usize,
            // FX29. Sets I to the location of the sprite for the character in VX.
            // Characters 0-F (in hexadecimal) are represented by a 4x5 font.
            0x29 => self.i = FONT_START + self.v[reg_x] as usize * 5,
            // FX33. Stores the binary-coded decimal representation of VX, 
            // with the hundreds digit in memory at location in I, the tens digit at location I+1, 
            // and the ones digit at location I+2. 
            0x33 => {
                self.ram[self.i] = self.v[reg_x] / 100;
                self.ram[self.i + 1] = (self.v[reg_x] % 100) / 10;
                self.ram[self.i + 2] = self.v[reg_x] % 10;
            },
            // FX55. Stores from V0 to VX (including VX) in memory, starting at address I. 
            // The offset from I is increased by 1 for each value written, but I itself is left unmodified.
            0x55 => {
                for i in 0..reg_x+1 as usize {
                    self.ram[self.i + i] = self.v[i];
                }
            },
            // FX65. Fills from V0 to VX (including VX) with values from memory, starting at address I. 
            // The offset from I is increased by 1 for each value read, but I itself is left unmodified.
            0x65 => {
                for i in 0..reg_x+1 as usize {
                    self.v[i] = self.ram[self.i + i];
                }
            },
            _ => println!("Unknown opcode: 0x{opcode:0>4x}")
        }
        self.pc += 2;
    }

}

#[cfg(test)]    
mod tests {
    use super::*;
    use test;

    #[test]
    fn op_0x00e0_clear_screen() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.ram[4060] = 1;
        assert_eq!(processor.ram[4060], 1);

        // act
        processor.execute_opcode(0x00e0);

        // assert
        assert_eq!(processor.ram[4060], 0);
        assert_eq!(processor.pc, 0x0202);
        
    }

    #[test]
    fn op_0x00ee_return_from_subroutine() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.stack[0] = 0x0236;
        processor.sp = 1;

        // act
        processor.execute_opcode(0x00EE);

        // assert
        assert_eq!(processor.pc, 0x0238);
    }

    #[test]
    fn op_0x1nnn_jump_to_address() {
        // arrange
        let mut processor = Processor::new(vec![]);

        // act
        processor.execute_opcode(0x1280);

        // assert
        assert_eq!(processor.pc, 0x0280);
    }

    #[test]
    fn op_0x2nnn_call_subroutine() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;

        // act
        processor.execute_opcode(0x2244);

        // assert
        assert_eq!(processor.pc, 0x0244);
        assert_eq!(processor.sp, 1);
        assert_eq!(processor.stack[0], 0x0222);
    }

    #[test]
    fn op_0x3xnn_skip_vx_equals_nn() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x34;

        // act
        processor.execute_opcode(0x3534);

        // assert
        assert_eq!(processor.pc, 0x0226);
    }

    #[test]
    fn op_0x3xnn_no_skip_vx_not_equals_nn() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x37;

        // act
        processor.execute_opcode(0x3534);

        // assert
        assert_eq!(processor.pc, 0x0224);
    }

    #[test]
    fn op_0x4xnn_skip_if_vx_not_equals_nn() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x37;

        // act
        processor.execute_opcode(0x4534);

        // assert
        assert_eq!(processor.pc, 0x0226);
    }

    #[test]
    fn op_0x4xnn_no_skip_if_vx_equals_nn() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x37;

        // act
        processor.execute_opcode(0x4537);

        // assert
        assert_eq!(processor.pc, 0x0224);
    }

    #[test]
    fn op_0x5xy0_skip_vx_equals_vy() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x37;
        processor.v[7] = 0x37;

        // act
        processor.execute_opcode(0x5570);

        // assert
        assert_eq!(processor.pc, 0x0226);
    }

    #[test]
    fn op_0x5xy0_no_skip_vx_not_equals_vy() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x37;
        processor.v[7] = 0x38;

        // act
        processor.execute_opcode(0x5570);

        // assert
        assert_eq!(processor.pc, 0x0224);
    }

    #[test]
    fn op_0x6xnn_set_vx_to_nn() {
        // arrange
        let mut processor = Processor::new(vec![]);

        // act
        processor.execute_opcode(0x6570);

        // assert
        assert_eq!(processor.v[5], 0x70);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x7xnn_add_nn_to_vx() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x05;

        // act
        processor.execute_opcode(0x7505);

        // assert
        assert_eq!(processor.v[5], 0x0a);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy0_set_vx_to_vy() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x05;
        processor.v[7] = 0xfa;

        // act
        processor.execute_opcode(0x8570);

        // assert
        assert_eq!(processor.v[5], 0xfa);
        assert_eq!(processor.v[7], 0xfa);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy1_set_vx_to_vx_or_vy_bitwise() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x05;
        processor.v[7] = 0xfa;

        // act
        processor.execute_opcode(0x8571);

        // assert
        assert_eq!(processor.v[5], 0xff);
        assert_eq!(processor.v[7], 0xfa);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy2_set_vx_to_vx_and_vy_bitwise() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0xfa;
        processor.v[7] = 0x0a;

        // act
        processor.execute_opcode(0x8572);

        // assert
        assert_eq!(processor.v[5], 0x0a);
        assert_eq!(processor.v[7], 0x0a);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy4_add_vy_to_vx_carry_set() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0xfe;
        processor.v[7] = 0x03;

        // act
        processor.execute_opcode(0x8574);

        // assert
        assert_eq!(processor.v[5], 0x01);
        assert_eq!(processor.v[7], 0x03);
        assert_eq!(processor.v[VF], 0x1);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy4_add_vy_to_vx_carry_not_set() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x0e;
        processor.v[7] = 0x03;

        // act
        processor.execute_opcode(0x8574);

        // assert
        assert_eq!(processor.v[5], 0x11);
        assert_eq!(processor.v[7], 0x03);
        assert_eq!(processor.v[VF], 0x0);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy5_subtract_vy_from_vx_borrow_not_set() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x01;
        processor.v[7] = 0x02;

        // act
        processor.execute_opcode(0x8575);

        // assert
        assert_eq!(processor.v[5], 0xff);
        assert_eq!(processor.v[7], 0x02);
        assert_eq!(processor.v[VF], 0x0);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy5_subtract_vy_from_vx_borrow_set() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x01;
        processor.v[7] = 0x01;

        // act
        processor.execute_opcode(0x8575);

        // assert
        assert_eq!(processor.v[5], 0x00);
        assert_eq!(processor.v[7], 0x01);
        assert_eq!(processor.v[VF], 0x1);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy6_shift_right() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x03;

        // act
        processor.execute_opcode(0x8506);

        // assert
        assert_eq!(processor.v[5], 0x01);
        assert_eq!(processor.v[VF], 0x1);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xy7_subtract_vx_from_vy_borrow_set() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x01;
        processor.v[7] = 0x01;

        // act
        processor.execute_opcode(0x8577);

        // assert
        assert_eq!(processor.v[5], 0x00);
        assert_eq!(processor.v[7], 0x01);
        assert_eq!(processor.v[VF], 0x1);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x8xye_shift_left() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[5] = 0x81;

        // act
        processor.execute_opcode(0x850e);

        // assert
        assert_eq!(processor.v[5], 0x02);
        assert_eq!(processor.v[VF], 0x1);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0x9xy0_skip_vx_not_equals_vy() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x37;
        processor.v[7] = 0x38;

        // act
        processor.execute_opcode(0x9570);

        // assert
        assert_eq!(processor.pc, 0x0226);
    }

    #[test]
    fn op_0x9xy0_no_skip_vx_equals_vy() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0222;
        processor.v[5] = 0x37;
        processor.v[7] = 0x37;

        // act
        processor.execute_opcode(0x9570);

        // assert
        assert_eq!(processor.pc, 0x0224);
    }

    #[test]
    fn op_0xannn_set_i_to_nnn() {
        // arrange
        let mut processor = Processor::new(vec![]);

        // act
        processor.execute_opcode(0xa123);

        // assert
        assert_eq!(processor.i, 0x0123);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0xbnnn_jump_to_nnn_plus_v0() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[0] = 0x8;

        // act
        processor.execute_opcode(0xb123);

        // assert
        assert_eq!(processor.pc, 0x012b);
    }

    #[test]
    fn op_0xcxnn_random_number_god() {
        // arrange
        let mut processor = Processor::new(vec![]);
        // let mut rng = rand::rng(123);

        // act
        processor.execute_opcode(0xc101);

        // assert
        // assert_eq!(processor.v[1], rng + 1);
        assert_eq!(processor.pc, 0x0202);
    }

    #[test]
    fn op_0xdxyn_draw_0_0() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[1] = 0;
        processor.v[2] = 0;
        processor.i = 0x300;
        let skull = vec![
            0x7e, 0xc9, 0xc9, 0xf7, 0x6a, 0x3e, 0x2a, 0x2a
        ];
        for i in 0..skull.len() {
            processor.ram[0x300 + i] = skull[i];
        }

        // act
        processor.execute_opcode(0xd128);

        // assert
        let expected_screen: Vec<u8> = vec![0x7e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            processor.ram[VRAM_START..VRAM_START+VRAM_SIZE_BYTE], 
            expected_screen);
        assert_eq!(processor.pc, 0x0202);
        assert_eq!(processor.v[VF], 0);
    }

    #[test]
    fn op_0xdxyn_draw_1_1() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[1] = 1;
        processor.v[2] = 1;
        processor.i = 0x300;
        let skull = vec![
            0x7e, 0xc9, 0xc9, 0xf7, 0x6a, 0x3e, 0x2a, 0x2a
        ];
        for i in 0..skull.len() {
            processor.ram[0x300 + i] = skull[i];
        }

        // act
        processor.execute_opcode(0xd128);

        // assert
        let expected_screen: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x35, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            processor.ram[VRAM_START..VRAM_START+VRAM_SIZE_BYTE], 
            expected_screen);
        assert_eq!(processor.pc, 0x0202);
        assert_eq!(processor.v[VF], 0);
    }

    #[test]
    fn op_0xdxyn_draw_9_59_overlap() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[1] = 60;
        processor.v[2] = 9;
        processor.i = 0x300;
        let skull = vec![
            0x7e, 0xc9, 0xc9, 0xf7, 0x6a, 0x3e, 0x2a, 0x2a
        ];
        for i in 0..skull.len() {
            processor.ram[0x300 + i] = skull[i];
        }

        // act
        processor.execute_opcode(0xd128);

        // assert
        let expected_screen: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0xa0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xa0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xa0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            processor.ram[VRAM_START..VRAM_START+VRAM_SIZE_BYTE], 
            expected_screen);
        assert_eq!(processor.pc, 0x0202);
        assert_eq!(processor.v[VF], 0);
    }

    #[test]
    fn op_0xdxyn_draw_28_59_overlap() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[1] = 59;
        processor.v[2] = 28;
        processor.i = 0x300;
        let skull = vec![
            0x7e, 0xc9, 0xc9, 0xf7, 0x6a, 0x3e, 0x2a, 0x2a
        ];
        for i in 0..skull.len() {
            processor.ram[0x300 + i] = skull[i];
        }

        // act
        processor.execute_opcode(0xd128);

        // assert
        let expected_screen: Vec<u8> = vec![0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0d, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1e];
        assert_eq!(
            processor.ram[VRAM_START..VRAM_START+VRAM_SIZE_BYTE], 
            expected_screen);
        assert_eq!(processor.pc, 0x0202);
        assert_eq!(processor.v[VF], 0);
    }

    #[test]
    fn op_0xdxyn_draw_28_59_overlap_collision() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[1] = 59;
        processor.v[2] = 28;
        processor.i = 0x300;
        processor.ram[VRAM_START] = 0b0100_0000;
        let skull = vec![
            0x7e, 0xc9, 0xc9, 0xf7, 0x6a, 0x3e, 0x2a, 0x2a
        ];
        for i in 0..skull.len() {
            processor.ram[0x300 + i] = skull[i];
        }

        // act
        processor.execute_opcode(0xd128);

        // assert
        let expected_screen: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0d, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1e];
        assert_eq!(
            processor.ram[VRAM_START..VRAM_START+VRAM_SIZE_BYTE], 
            expected_screen);
        assert_eq!(processor.pc, 0x0202);
        assert_eq!(processor.v[VF], 1);
    }

    #[test]
    fn op_0xdxyn_draw_28_59_overlap_no_collision() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[1] = 59;
        processor.v[2] = 28;
        processor.i = 0x300;
        processor.ram[VRAM_START] = 0b1000_0000;
        let skull = vec![
            0x7e, 0xc9, 0xc9, 0xf7, 0x6a, 0x3e, 0x2a, 0x2a
        ];
        for i in 0..skull.len() {
            processor.ram[0x300 + i] = skull[i];
        }

        // act
        processor.execute_opcode(0xd128);

        // assert
        let expected_screen: Vec<u8> = vec![0b1100_0000, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0d, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1e];
        assert_eq!(
            processor.ram[VRAM_START..VRAM_START+VRAM_SIZE_BYTE], 
            expected_screen);
        assert_eq!(processor.pc, 0x0202);
        assert_eq!(processor.v[VF], 0);
    }

    #[test]
    fn set_pixel_test() {
        // arrange
        let mut processor = Processor::new(vec![]);

        // act
        processor.set_pixel(4, 1, true);

        // assert

        assert_eq!(processor.ram[VRAM_START+SCREEN_WIDTH/8], 0b0000_1000);
    }

    #[test]
    fn op_0xex9e_skip() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0220;
        processor.v[3] = 5;
        processor.key_state[5] = true;

        // act
        processor.execute_opcode(0xe39e);

        // assert
        assert_eq!(processor.pc, 0x0224);
    }

    #[test]
    fn op_0xex9e_no_skip() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0220;
        processor.v[3] = 5;
        processor.key_state[5] = false;

        // act
        processor.execute_opcode(0xe39e);

        // assert
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xexa1_skip() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0220;
        processor.v[3] = 5;
        processor.key_state[5] = false;

        // act
        processor.execute_opcode(0xe3a1);

        // assert
        assert_eq!(processor.pc, 0x0224);
    }

    #[test]
    fn op_0xexa1_no_skip() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0220;
        processor.v[3] = 5;
        processor.key_state[5] = true;

        // act
        processor.execute_opcode(0xe3a1);

        // assert
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx07_set_delay_timer() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0220;
        processor.delay_timer = 15;

        // act
        processor.execute_opcode(0xf307);

        // assert
        assert_eq!(processor.v[3], 15);
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx0a_wait_for_key_pressed() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.pc = 0x0220;
        processor.key_state[12] = true;

        // act
        processor.execute_opcode(0xf30a);
        processor.tick();

        // assert
        assert_eq!(processor.v[3], 12);
        assert_eq!(processor.pc, 0x0224);
    }

    #[test]
    fn op_0xfx15_set_delay_timer_to_vx() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[3] = 35;
        processor.pc = 0x0220;

        // act
        processor.execute_opcode(0xf315);

        // assert
        assert_eq!(processor.delay_timer, 35);
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx18_set_sound_timer_to_vx() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[3] = 35;
        processor.pc = 0x0220;

        // act
        processor.execute_opcode(0xf318);

        // assert
        assert_eq!(processor.sound_timer, 35);
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx1e_add_vx_to_i() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[3] = 35;
        processor.pc = 0x0220;
        processor.i = 3;

        // act
        processor.execute_opcode(0xf31e);

        // assert
        assert_eq!(processor.i, 38);
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx29_set_i_to_sprite() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[3] = 0xd;
        processor.pc = 0x0220;

        // act
        processor.execute_opcode(0xf329);

        // assert
        assert_eq!(processor.i, FONT_START + 13 * 5);
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx33_store_decimal() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[3] = 123;
        processor.pc = 0x0220;
        processor.i = 0x300;

        // act
        processor.execute_opcode(0xf333);

        // assert
        assert_eq!(processor.ram[0x300], 1);
        assert_eq!(processor.ram[0x301], 2);
        assert_eq!(processor.ram[0x302], 3);
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx55_store_v_to_ram() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.v[0] = 0;
        processor.v[1] = 1;
        processor.v[2] = 2;
        processor.v[3] = 3;
        processor.pc = 0x0220;
        processor.i = 0x300;

        // act
        processor.execute_opcode(0xf355);

        // assert
        assert_eq!(processor.ram[0x300], 0);
        assert_eq!(processor.ram[0x301], 1);
        assert_eq!(processor.ram[0x302], 2);
        assert_eq!(processor.ram[0x303], 3);
        assert_eq!(processor.ram[0x304], 0);
        assert_eq!(processor.pc, 0x0222);
    }

    #[test]
    fn op_0xfx65_store_ram_to_v() {
        // arrange
        let mut processor = Processor::new(vec![]);
        processor.ram[0x300] = 0;
        processor.ram[0x301] = 1;
        processor.ram[0x302] = 2;
        processor.ram[0x303] = 3;
        processor.v[3] = 3;
        processor.pc = 0x0220;
        processor.i = 0x300;

        // act
        processor.execute_opcode(0xf365);

        // assert
        assert_eq!(processor.v[0], 0);
        assert_eq!(processor.v[1], 1);
        assert_eq!(processor.v[2], 2);
        assert_eq!(processor.v[3], 3);
        assert_eq!(processor.v[4], 0);
        assert_eq!(processor.pc, 0x0222);
    }

}
