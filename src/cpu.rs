use rand::Rng;
pub const SCALE: usize = 15;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const FONTSET_LEN: usize = 80;
const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
pub struct Cpu {
    pc: u16,
    sp: u8,
    stack: [u16; 16],

    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub prev_screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys: [bool; 16],
    prev_keys: [bool; 16],  // Onceki frame'deki tuş durumu
    waiting_for_key_release: Option<u8>,  // FX0A için tuş bırakma bekleme
    v: [u8; 16],
    i: usize,

    pub st: u8,
    pub dt: u8,

    memory: [u8; 4096],
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn get_display(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.screen
    }
    pub fn get_last_buf(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.prev_screen
    }

    pub fn keypress(&mut self, key: usize, pressed: bool) {
        self.prev_keys[key] = self.keys[key];
        self.keys[key] = pressed;
    }
    pub fn new() -> Cpu {
        Cpu {
            pc: 0x200,
            stack: [0; 16],
            sp: 0,

            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            prev_screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            keys: [false; 16],
            prev_keys: [false; 16],
            waiting_for_key_release: None,
            v: [0; 16],
            i: 0,

            st: 0,
            dt: 0,

            memory: [0; 4096],
        }
    }

    pub fn load_font(&mut self) {
        (0..FONTSET_LEN).for_each(|i| {
            self.memory[i] = FONTSET[i];
        });
    }

    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.stack = [0; 16];
        self.sp = 0;
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.prev_screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.keys = [false; 16];
        self.prev_keys = [false; 16];
        self.waiting_for_key_release = None;
        self.v = [0; 16];
        self.i = 0;
        self.st = 0;
        self.dt = 0;
        self.memory = [0; 4096];
    }
    pub fn load(&mut self, path: &String) {
        let rom = std::fs::read(path).unwrap();
        for (i, byte) in rom.iter().enumerate() {
            self.memory[i + self.pc as usize] = *byte;
        }
    }
    fn set_fontset(&mut self) {
        for (i, &byte) in FONTSET.iter().enumerate() {
            self.memory[i] = byte;
        }
    }
    pub fn timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }
    pub fn tick(&mut self) {
        self.decode_opcode();
    }
    fn fetch_opcode(&mut self) -> u16 {
        let opcode =
            (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;
        opcode
    }
    pub fn decode_opcode(&mut self) {
        let opcode: u16 = self.fetch_opcode();
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => self.op_00e0(),
                0x000E => self.op_00ee(),
                _ => println!("Unknown opcode: {:X}", opcode),
            },
            0x1000 => self.op_1nnn(opcode),
            0x2000 => self.op_2nnn(opcode),
            0x3000 => self.op_3xkk(opcode),
            0x4000 => self.op_4xkk(opcode),
            0x5000 => self.op_5xy0(opcode),
            0x6000 => self.op_6xkk(opcode),
            0x7000 => self.op_7xkk(opcode),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy0(opcode),
                0x0001 => self.op_8xy1(opcode),
                0x0002 => self.op_8xy2(opcode),
                0x0003 => self.op_8xy3(opcode),
                0x0004 => self.op_8xy4(opcode),
                0x0005 => self.op_8xy5(opcode),
                0x0006 => self.op_8xy6(opcode),
                0x0007 => self.op_8xy7(opcode),
                0x000E => self.op_8xye(opcode),
                _ => println!("Unknown opcode: {:X}", opcode),
            },
            0x9000 => self.op_9xy0(opcode),
            0xA000 => self.op_annn(opcode),
            0xB000 => self.op_bnnn(opcode),
            0xC000 => self.op_cxkk(opcode),
            0xD000 => self.op_dxyn(opcode),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(opcode),
                0x00A1 => self.op_exa1(opcode),
                _ => println!("Unknown opcode: {:X}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(opcode),
                0x000A => self.op_fx0a(opcode),
                0x0015 => self.op_fx15(opcode),
                0x0018 => self.op_fx18(opcode),
                0x001E => self.op_fx1e(opcode),
                0x0029 => self.op_fx29(opcode),
                0x0033 => self.op_fx33(opcode),
                0x0055 => self.op_fx55(opcode),
                0x0065 => self.op_fx65(opcode),
                _ => println!("Unknown opcode: {:X}", opcode),
            },
            _ => println!("Unknown opcode: {:X}", opcode),
        }
    }

    fn op_00e0(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }
    fn op_00ee(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp = self.sp.wrapping_sub(1)
    }
    fn op_1nnn(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }
    fn op_2nnn(&mut self, opcode: u16) {
        self.sp = self.sp.wrapping_add(1);
        self.stack[self.sp as usize] = self.pc;
        self.pc = opcode & 0x0FFF;
    }
    fn op_3xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] == kk {
            self.pc += 2;
        }
    }
    fn op_4xkk(&mut self, opcode: u16) {
        //skip next instruction if Vx != kk
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] != kk {
            self.pc += 2;
        }
    }
    fn op_5xy0(&mut self, opcode: u16) {
        //skip next instruction if Vx = Vy
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }
    fn op_6xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = kk;
    }
    fn op_7xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = self.v[x].wrapping_add(kk)
    }
    fn op_8xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] = self.v[y];
    }
    fn op_8xy1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] |= self.v[y];
    }
    fn op_8xy2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] &= self.v[y];
    }
    fn op_8xy3(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] ^= self.v[y];
    }
    fn op_8xy4(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let (new_vx, carry) = self.v[x].overflowing_add(self.v[y]);
        let new_vf = if carry { 1 } else { 0 };
        self.v[x] = new_vx;
        self.v[0xF] = new_vf;
    }
    fn op_8xy5(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x] as i8;
        let vy = self.v[y] as i8;
        let (new_vx, borrow) = self.v[x].overflowing_sub(self.v[y]);
        let new_vf = if borrow { 0 } else { 1 };
        self.v[x] = new_vx;
        self.v[0xF] = new_vf;
    }
    fn op_8xy6(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[x] & 0x1;
        self.v[x] >>= 1;
    }
    fn op_8xy7(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] > self.v[y] {
            self.v[0xF] = 0;
        } else {
            self.v[0xF] = 1;
        }
        self.v[x] = self.v[y].wrapping_sub(self.v[x])
    }
    fn op_8xye(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if self.v[x] & 0x80 != 0 {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[x] <<= 1;
    }
    fn op_9xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }
    fn op_annn(&mut self, opcode: u16) {
        self.i = (opcode & 0x0FFF) as usize;
    }
    fn op_bnnn(&mut self, opcode: u16) {
        self.pc = (opcode & 0x0FFF) + self.v[0] as u16;
    }
    fn op_cxkk(&mut self, opcode: u16) {
        let mut rng = rand::thread_rng();
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = rng.gen::<u8>() & kk;
    }
    fn op_dxyn(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = opcode & 0x000F;
        let mut flipped = false;
        for y_offset in 0..n {
            let sprite = self.i + y_offset as usize;
            let pixel = self.memory[sprite];
            for x_offset in 0..8 {
                if (pixel & (0x80 >> x_offset)) != 0 {
                    let x = (self.v[x] as usize + x_offset) % SCREEN_WIDTH;
                    let y = (self.v[y] as usize + y_offset as usize) % SCREEN_HEIGHT;
                    if self.screen[x + y * SCREEN_WIDTH] {
                        flipped = true;
                    }
                    self.screen[x + y * SCREEN_WIDTH] ^= true;
                }
            }
        }
        if flipped {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
    }
    fn op_ex9e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;
        if self.keys[key] {
            self.pc += 2;
        }
    }
    fn op_exa1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;
        if !self.keys[key] {
            self.pc += 2;
        }
    }
    fn op_fx07(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[x] = self.dt;
    }
    fn op_fx0a(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        
        // Eger bir tus birakilmasini bekliyorsak
        if let Some(key) = self.waiting_for_key_release {
            // Tus birakildi mi kontrol et
            if !self.keys[key as usize] {
                self.v[x] = key;
                self.waiting_for_key_release = None;
                return;
            }
            // Hala basili, bekle
            self.pc -= 2;
            return;
        }
        
        // Yeni tus basildi mi kontrol et
        for i in 0..16 {
            if self.keys[i] && !self.prev_keys[i] {
                // Tus yeni basildi, birakilmasini bekle
                self.waiting_for_key_release = Some(i as u8);
                self.pc -= 2;
                return;
            }
        }
        
        // Hic tus basilmadi, bekle
        self.pc -= 2;
    }
    fn op_fx15(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.dt = self.v[x];
    }
    fn op_fx18(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.st = self.v[x];
    }
    fn op_fx1e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i += self.v[x] as usize;
    }
    fn op_fx29(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let sprite = self.v[x];
        self.i = sprite as usize * 5;
    }
    fn op_fx33(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.memory[self.i] = self.v[x] / 100;
        self.memory[self.i + 1] = (self.v[x] / 10) % 10;
        self.memory[self.i + 2] = self.v[x] % 10;
    }
    fn op_fx55(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.memory[self.i + i] = self.v[i];
        }
    }
    fn op_fx65(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.v[i] = self.memory[self.i + i];
        }
    }

    // Test helper: opcode'u memory'ye yaz ve calistir
    #[cfg(test)]
    pub fn execute_opcode(&mut self, opcode: u16) {
        self.memory[self.pc as usize] = (opcode >> 8) as u8;
        self.memory[self.pc as usize + 1] = (opcode & 0xFF) as u8;
        self.decode_opcode();
    }

    // Test helper: register degerini al
    #[cfg(test)]
    pub fn get_v(&self, x: usize) -> u8 {
        self.v[x]
    }

    // Test helper: register degerini ayarla
    #[cfg(test)]
    pub fn set_v(&mut self, x: usize, val: u8) {
        self.v[x] = val;
    }

    // Test helper: I register degerini al
    #[cfg(test)]
    pub fn get_i(&self) -> usize {
        self.i
    }

    // Test helper: PC degerini al
    #[cfg(test)]
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    // Test helper: PC degerini ayarla
    #[cfg(test)]
    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CPU OPCODE TESTLERI ====================

    #[test]
    fn test_op_00e0_clear_screen() {
        let mut cpu = Cpu::new();
        // Ekrani doldur
        cpu.screen = [true; SCREEN_WIDTH * SCREEN_HEIGHT];
        // 00E0 - CLS
        cpu.execute_opcode(0x00E0);
        // Ekran temizlenmeli
        assert!(cpu.screen.iter().all(|&p| !p));
    }

    #[test]
    fn test_op_1nnn_jump() {
        let mut cpu = Cpu::new();
        // 1NNN - JP addr
        cpu.execute_opcode(0x1ABC);
        assert_eq!(cpu.get_pc(), 0xABC);
    }

    #[test]
    fn test_op_2nnn_call_subroutine() {
        let mut cpu = Cpu::new();
        let initial_pc = cpu.get_pc();
        // 2NNN - CALL addr
        cpu.execute_opcode(0x2345);
        assert_eq!(cpu.get_pc(), 0x345);
        // Stack'te eski PC olmali
        assert_eq!(cpu.stack[cpu.sp as usize], initial_pc + 2);
    }

    #[test]
    fn test_op_00ee_return() {
        let mut cpu = Cpu::new();
        // Once subroutine cagir
        cpu.execute_opcode(0x2400);
        // Sonra don
        cpu.execute_opcode(0x00EE);
        // PC eski yerine donmeli (0x200 + 2 = 0x202)
        assert_eq!(cpu.get_pc(), 0x202);
    }

    #[test]
    fn test_op_3xkk_skip_if_equal() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x42);
        let pc_before = cpu.get_pc();
        // 3XKK - SE Vx, byte (esitse atla)
        cpu.execute_opcode(0x3042);
        // Esit oldugu icin 4 byte atlamali (2 opcode + 2 skip)
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_3xkk_no_skip_if_not_equal() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x42);
        let pc_before = cpu.get_pc();
        // Esit degil
        cpu.execute_opcode(0x3043);
        assert_eq!(cpu.get_pc(), pc_before + 2);
    }

    #[test]
    fn test_op_4xkk_skip_if_not_equal() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x42);
        let pc_before = cpu.get_pc();
        // 4XKK - SNE Vx, byte
        cpu.execute_opcode(0x4043);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_5xy0_skip_if_vx_eq_vy() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x42);
        cpu.set_v(1, 0x42);
        let pc_before = cpu.get_pc();
        // 5XY0 - SE Vx, Vy
        cpu.execute_opcode(0x5010);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_6xkk_load_byte() {
        let mut cpu = Cpu::new();
        // 6XKK - LD Vx, byte
        cpu.execute_opcode(0x6A42);
        assert_eq!(cpu.get_v(0xA), 0x42);
    }

    #[test]
    fn test_op_7xkk_add_byte() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x10);
        // 7XKK - ADD Vx, byte
        cpu.execute_opcode(0x7005);
        assert_eq!(cpu.get_v(0), 0x15);
    }

    #[test]
    fn test_op_7xkk_add_overflow() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0xFF);
        cpu.execute_opcode(0x7002);
        assert_eq!(cpu.get_v(0), 0x01); // Wrap around
    }

    #[test]
    fn test_op_8xy0_load_vy_to_vx() {
        let mut cpu = Cpu::new();
        cpu.set_v(1, 0x42);
        // 8XY0 - LD Vx, Vy
        cpu.execute_opcode(0x8010);
        assert_eq!(cpu.get_v(0), 0x42);
    }

    #[test]
    fn test_op_8xy1_or() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x0F);
        cpu.set_v(1, 0xF0);
        // 8XY1 - OR Vx, Vy
        cpu.execute_opcode(0x8011);
        assert_eq!(cpu.get_v(0), 0xFF);
    }

    #[test]
    fn test_op_8xy2_and() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x0F);
        cpu.set_v(1, 0xFF);
        // 8XY2 - AND Vx, Vy
        cpu.execute_opcode(0x8012);
        assert_eq!(cpu.get_v(0), 0x0F);
    }

    #[test]
    fn test_op_8xy3_xor() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0xFF);
        cpu.set_v(1, 0x0F);
        // 8XY3 - XOR Vx, Vy
        cpu.execute_opcode(0x8013);
        assert_eq!(cpu.get_v(0), 0xF0);
    }

    #[test]
    fn test_op_8xy4_add_no_carry() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x10);
        cpu.set_v(1, 0x20);
        // 8XY4 - ADD Vx, Vy
        cpu.execute_opcode(0x8014);
        assert_eq!(cpu.get_v(0), 0x30);
        assert_eq!(cpu.get_v(0xF), 0); // No carry
    }

    #[test]
    fn test_op_8xy4_add_with_carry() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0xFF);
        cpu.set_v(1, 0x02);
        cpu.execute_opcode(0x8014);
        assert_eq!(cpu.get_v(0), 0x01);
        assert_eq!(cpu.get_v(0xF), 1); // Carry
    }

    #[test]
    fn test_op_8xy5_sub_no_borrow() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x20);
        cpu.set_v(1, 0x10);
        // 8XY5 - SUB Vx, Vy
        cpu.execute_opcode(0x8015);
        assert_eq!(cpu.get_v(0), 0x10);
        assert_eq!(cpu.get_v(0xF), 1); // No borrow
    }

    #[test]
    fn test_op_8xy5_sub_with_borrow() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x10);
        cpu.set_v(1, 0x20);
        cpu.execute_opcode(0x8015);
        assert_eq!(cpu.get_v(0), 0xF0); // Wrap
        assert_eq!(cpu.get_v(0xF), 0); // Borrow
    }

    #[test]
    fn test_op_8xy6_shift_right() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0b00000011);
        // 8XY6 - SHR Vx
        cpu.execute_opcode(0x8006);
        assert_eq!(cpu.get_v(0), 0b00000001);
        assert_eq!(cpu.get_v(0xF), 1); // LSB was 1
    }

    #[test]
    fn test_op_8xye_shift_left() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0b10000001);
        // 8XYE - SHL Vx
        cpu.execute_opcode(0x800E);
        assert_eq!(cpu.get_v(0), 0b00000010);
        assert_eq!(cpu.get_v(0xF), 1); // MSB was 1
    }

    #[test]
    fn test_op_9xy0_skip_if_vx_ne_vy() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x42);
        cpu.set_v(1, 0x43);
        let pc_before = cpu.get_pc();
        // 9XY0 - SNE Vx, Vy
        cpu.execute_opcode(0x9010);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_annn_load_i() {
        let mut cpu = Cpu::new();
        // ANNN - LD I, addr
        cpu.execute_opcode(0xA123);
        assert_eq!(cpu.get_i(), 0x123);
    }

    #[test]
    fn test_op_bnnn_jump_v0() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x10);
        // BNNN - JP V0, addr
        cpu.execute_opcode(0xB100);
        assert_eq!(cpu.get_pc(), 0x110);
    }

    #[test]
    fn test_op_fx1e_add_i() {
        let mut cpu = Cpu::new();
        cpu.i = 0x100;
        cpu.set_v(0, 0x10);
        // FX1E - ADD I, Vx
        cpu.execute_opcode(0xF01E);
        assert_eq!(cpu.get_i(), 0x110);
    }

    #[test]
    fn test_op_fx33_bcd() {
        let mut cpu = Cpu::new();
        cpu.i = 0x300;
        cpu.set_v(0, 234);
        // FX33 - LD B, Vx
        cpu.execute_opcode(0xF033);
        assert_eq!(cpu.memory[0x300], 2); // Yuzler
        assert_eq!(cpu.memory[0x301], 3); // Onlar
        assert_eq!(cpu.memory[0x302], 4); // Birler
    }

    #[test]
    fn test_op_fx55_store_registers() {
        let mut cpu = Cpu::new();
        cpu.i = 0x300;
        cpu.set_v(0, 1);
        cpu.set_v(1, 2);
        cpu.set_v(2, 3);
        // FX55 - LD [I], Vx
        cpu.execute_opcode(0xF255);
        assert_eq!(cpu.memory[0x300], 1);
        assert_eq!(cpu.memory[0x301], 2);
        assert_eq!(cpu.memory[0x302], 3);
    }

    #[test]
    fn test_op_fx65_load_registers() {
        let mut cpu = Cpu::new();
        cpu.i = 0x300;
        cpu.memory[0x300] = 10;
        cpu.memory[0x301] = 20;
        cpu.memory[0x302] = 30;
        // FX65 - LD Vx, [I]
        cpu.execute_opcode(0xF265);
        assert_eq!(cpu.get_v(0), 10);
        assert_eq!(cpu.get_v(1), 20);
        assert_eq!(cpu.get_v(2), 30);
    }

    // ==================== TIMER TESTLERI ====================

    #[test]
    fn test_timer_delay_decrement() {
        let mut cpu = Cpu::new();
        cpu.dt = 10;
        cpu.timers();
        assert_eq!(cpu.dt, 9);
    }

    #[test]
    fn test_timer_sound_decrement() {
        let mut cpu = Cpu::new();
        cpu.st = 5;
        cpu.timers();
        assert_eq!(cpu.st, 4);
    }

    #[test]
    fn test_timer_no_underflow() {
        let mut cpu = Cpu::new();
        cpu.dt = 0;
        cpu.st = 0;
        cpu.timers();
        assert_eq!(cpu.dt, 0);
        assert_eq!(cpu.st, 0);
    }

    #[test]
    fn test_op_fx07_get_delay() {
        let mut cpu = Cpu::new();
        cpu.dt = 0x42;
        // FX07 - LD Vx, DT
        cpu.execute_opcode(0xF007);
        assert_eq!(cpu.get_v(0), 0x42);
    }

    #[test]
    fn test_op_fx15_set_delay() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x30);
        // FX15 - LD DT, Vx
        cpu.execute_opcode(0xF015);
        assert_eq!(cpu.dt, 0x30);
    }

    #[test]
    fn test_op_fx18_set_sound() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x20);
        // FX18 - LD ST, Vx
        cpu.execute_opcode(0xF018);
        assert_eq!(cpu.st, 0x20);
    }

    // ==================== DISPLAY TESTLERI ====================

    #[test]
    fn test_screen_initial_state() {
        let cpu = Cpu::new();
        assert!(cpu.screen.iter().all(|&p| !p));
    }

    #[test]
    fn test_reset_clears_screen() {
        let mut cpu = Cpu::new();
        cpu.screen[0] = true;
        cpu.screen[100] = true;
        cpu.reset();
        assert!(cpu.screen.iter().all(|&p| !p));
    }

    #[test]
    fn test_op_fx29_font_sprite() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x5); // Sprite for '5'
        // FX29 - LD F, Vx
        cpu.execute_opcode(0xF029);
        // Font sprite 5 = 5 * 5 = 25
        assert_eq!(cpu.get_i(), 25);
    }

    // ==================== KEYBOARD TESTLERI ====================

    #[test]
    fn test_keypress() {
        let mut cpu = Cpu::new();
        cpu.keypress(0x5, true);
        assert!(cpu.keys[0x5]);
        cpu.keypress(0x5, false);
        assert!(!cpu.keys[0x5]);
    }

    #[test]
    fn test_op_ex9e_skip_if_key_pressed() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x5);
        cpu.keypress(0x5, true);
        let pc_before = cpu.get_pc();
        // EX9E - SKP Vx
        cpu.execute_opcode(0xE09E);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_exa1_skip_if_key_not_pressed() {
        let mut cpu = Cpu::new();
        cpu.set_v(0, 0x5);
        cpu.keypress(0x5, false);
        let pc_before = cpu.get_pc();
        // EXA1 - SKNP Vx
        cpu.execute_opcode(0xE0A1);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }
}
