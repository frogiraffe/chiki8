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
        self.timers();
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
        let mut key_pressed = false;
        for i in 0..16 {
            if self.keys[i] {
                self.v[x] = i as u8;
                key_pressed = true;
            }
        }
        if !key_pressed {
            self.pc -= 2;
        }
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
        println!("{:?}", self.v);
    }
}
