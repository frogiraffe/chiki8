use rand::{rngs::StdRng, Rng, SeedableRng};
use std::path::Path;
use std::str::FromStr;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const PHYSICAL_SCREEN_WIDTH: usize = 128;
pub const PHYSICAL_SCREEN_HEIGHT: usize = 64;
const PROGRAM_START: u16 = 0x200;
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
const HIGH_FONT_BASE: usize = FONTSET.len();
const HIGH_FONT_HEIGHT: usize = 10;
const HIGH_FONTSET: [u8; 100] = [
    0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7, 0x7E, 0x3C, 0x18, 0x38, 0x58, 0x18,
    0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60,
    0xFF, 0xFF, 0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, 0x06, 0x0E,
    0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, 0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE,
    0x03, 0xC3, 0x7E, 0x3C, 0x3E, 0x7C, 0xE0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C,
    0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, 0x3C, 0x7E, 0xC3, 0xC3,
    0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, 0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03,
    0x3E, 0x7C,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Classic,
    SuperChip11,
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "classic" => Ok(Self::Classic),
            "superchip-1.1" => Ok(Self::SuperChip11),
            _ => Err(format!(
                "unknown profile '{value}'; expected 'classic' or 'superchip-1.1'"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepOutcome {
    Executed,
    Unsupported,
    Halted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DisplayMode {
    Low,
    High,
}

pub struct Cpu {
    profile: Profile,
    pc: u16,
    sp: u8,
    stack: [u16; 16],

    pub screen: [bool; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT],
    display_mode: DisplayMode,
    keys: [bool; 16],
    prev_keys: [bool; 16],
    waiting_for_key_release: Option<u8>,
    v: [u8; 16],
    i: usize,

    pub st: u8,
    pub dt: u8,

    memory: [u8; 4096],
    rpl: [u8; 8],
    halted: bool,
    rng: StdRng,
    initial_rng: StdRng,
}

impl Cpu {
    pub fn get_display(&self) -> &[bool; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT] {
        &self.screen
    }
    pub fn active_dimensions(&self) -> (usize, usize) {
        match self.display_mode {
            DisplayMode::Low => (SCREEN_WIDTH, SCREEN_HEIGHT),
            DisplayMode::High => (PHYSICAL_SCREEN_WIDTH, PHYSICAL_SCREEN_HEIGHT),
        }
    }
    pub fn keypress(&mut self, key: usize, pressed: bool) {
        if let (Some(previous), Some(current)) =
            (self.prev_keys.get_mut(key), self.keys.get_mut(key))
        {
            *previous = *current;
            *current = pressed;
        }
    }
    pub fn new(profile: Profile) -> Cpu {
        Self::with_rng(profile, StdRng::from_entropy())
    }

    fn with_rng(profile: Profile, rng: StdRng) -> Cpu {
        let mut cpu = Cpu {
            profile,
            pc: PROGRAM_START,
            stack: [0; 16],
            sp: 0,

            screen: [false; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT],
            display_mode: DisplayMode::Low,
            keys: [false; 16],
            prev_keys: [false; 16],
            waiting_for_key_release: None,
            v: [0; 16],
            i: 0,

            st: 0,
            dt: 0,

            memory: [0; 4096],
            rpl: [0; 8],
            halted: false,
            initial_rng: rng.clone(),
            rng,
        };
        cpu.set_fontset();
        cpu
    }

    #[cfg(test)]
    fn with_seed(profile: Profile, seed: u64) -> Cpu {
        Self::with_rng(profile, StdRng::seed_from_u64(seed))
    }

    pub fn reset(&mut self) {
        self.pc = PROGRAM_START;
        self.stack = [0; 16];
        self.sp = 0;
        self.screen = [false; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT];
        self.display_mode = DisplayMode::Low;
        self.keys = [false; 16];
        self.prev_keys = [false; 16];
        self.waiting_for_key_release = None;
        self.v = [0; 16];
        self.i = 0;
        self.st = 0;
        self.dt = 0;
        self.memory = [0; 4096];
        self.rpl = [0; 8];
        self.halted = false;
        self.rng = self.initial_rng.clone();
        self.set_fontset();
    }
    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        let rom = std::fs::read(path)
            .map_err(|error| format!("Could not read ROM '{}': {error}", path.display()))?;
        self.load_rom(&rom)
    }
    fn load_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        let start = PROGRAM_START as usize;
        let end = start + rom.len();
        if end > self.memory.len() {
            return Err(format!(
                "ROM is {} bytes; maximum supported size is {} bytes",
                rom.len(),
                self.memory.len() - start
            ));
        }
        self.memory[start..end].copy_from_slice(rom);
        Ok(())
    }
    fn set_fontset(&mut self) {
        self.memory[..FONTSET.len()].copy_from_slice(&FONTSET);
        self.memory[HIGH_FONT_BASE..HIGH_FONT_BASE + HIGH_FONTSET.len()]
            .copy_from_slice(&HIGH_FONTSET);
    }
    pub fn timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }
    pub fn tick(&mut self) -> StepOutcome {
        self.decode_opcode()
    }

    fn peek_opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16
    }

    fn supports_opcode(&self, opcode: u16) -> bool {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 | 0x00EE => true,
                0x00C1..=0x00CF | 0x00FB..=0x00FF => self.profile == Profile::SuperChip11,
                _ => false,
            },
            0x1000..=0x4000 | 0x6000 | 0x7000 | 0xA000..=0xC000 => true,
            0xD000 => {
                let rows = (opcode & 0x000F) as usize;
                let bytes = if rows == 0 {
                    if self.profile != Profile::SuperChip11
                        || self.display_mode != DisplayMode::High
                    {
                        return false;
                    }
                    32
                } else {
                    rows
                };
                self.i
                    .checked_add(bytes)
                    .is_some_and(|end| end <= self.memory.len())
            }
            0x5000 | 0x9000 => opcode & 0x000F == 0,
            0x8000 => matches!(opcode & 0x000F, 0x0..=0x7 | 0xE),
            0xE000 => matches!(opcode & 0x00FF, 0x9E | 0xA1),
            0xF000 => match opcode & 0x00FF {
                0x07 | 0x0A | 0x15 | 0x18 | 0x1E | 0x29 | 0x33 | 0x55 | 0x65 => true,
                0x30 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.profile == Profile::SuperChip11 && self.v[x] <= 9
                }
                0x75 | 0x85 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.profile == Profile::SuperChip11 && x <= 7
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn decode_opcode(&mut self) -> StepOutcome {
        if self.halted {
            return StepOutcome::Halted;
        }

        let opcode = self.peek_opcode();
        if !self.supports_opcode(opcode) {
            return StepOutcome::Unsupported;
        }
        self.pc += 2;

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                0x00C1..=0x00CF => self.op_00cn(opcode),
                0x00FB => self.op_00fb(),
                0x00FC => self.op_00fc(),
                0x00FD => {
                    self.halted = true;
                    return StepOutcome::Halted;
                }
                0x00FE => self.display_mode = DisplayMode::Low,
                0x00FF => self.display_mode = DisplayMode::High,
                _ => unreachable!(),
            },
            0x1000 => self.op_1nnn(opcode),
            0x2000 => self.op_2nnn(opcode),
            0x3000 => self.op_3xkk(opcode),
            0x4000 => self.op_4xkk(opcode),
            0x5000 if opcode & 0x000F == 0 => self.op_5xy0(opcode),
            0x5000 => unreachable!(),
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
                _ => unreachable!(),
            },
            0x9000 if opcode & 0x000F == 0 => self.op_9xy0(opcode),
            0x9000 => unreachable!(),
            0xA000 => self.op_annn(opcode),
            0xB000 => self.op_bnnn(opcode),
            0xC000 => self.op_cxkk(opcode),
            0xD000 => self.op_dxyn(opcode),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(opcode),
                0x00A1 => self.op_exa1(opcode),
                _ => unreachable!(),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(opcode),
                0x000A => self.op_fx0a(opcode),
                0x0015 => self.op_fx15(opcode),
                0x0018 => self.op_fx18(opcode),
                0x001E => self.op_fx1e(opcode),
                0x0029 => self.op_fx29(opcode),
                0x0030 => self.op_fx30(opcode),
                0x0033 => self.op_fx33(opcode),
                0x0055 => self.op_fx55(opcode),
                0x0065 => self.op_fx65(opcode),
                0x0075 => self.op_fx75(opcode),
                0x0085 => self.op_fx85(opcode),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
        StepOutcome::Executed
    }

    fn op_00e0(&mut self) {
        self.screen = [false; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT];
    }
    fn op_00cn(&mut self, opcode: u16) {
        let rows = (opcode & 0x000F) as usize;
        let shift = rows * PHYSICAL_SCREEN_WIDTH;
        let retained = self.screen.len() - shift;
        self.screen.copy_within(..retained, shift);
        self.screen[..shift].fill(false);
    }
    fn op_00fb(&mut self) {
        for row in self.screen.chunks_exact_mut(PHYSICAL_SCREEN_WIDTH) {
            row.copy_within(..PHYSICAL_SCREEN_WIDTH - 4, 4);
            row[..4].fill(false);
        }
    }
    fn op_00fc(&mut self) {
        for row in self.screen.chunks_exact_mut(PHYSICAL_SCREEN_WIDTH) {
            row.copy_within(4.., 0);
            row[PHYSICAL_SCREEN_WIDTH - 4..].fill(false);
        }
    }
    fn op_00ee(&mut self) {
        self.sp = self.sp.wrapping_sub(1);
        self.pc = self.stack[self.sp as usize];
    }
    fn op_1nnn(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }
    fn op_2nnn(&mut self, opcode: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp = self.sp.wrapping_add(1);
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
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] != kk {
            self.pc += 2;
        }
    }
    fn op_5xy0(&mut self, opcode: u16) {
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
        if self.profile == Profile::SuperChip11 {
            self.v[0xF] = 0;
        }
    }
    fn op_8xy2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] &= self.v[y];
        if self.profile == Profile::SuperChip11 {
            self.v[0xF] = 0;
        }
    }
    fn op_8xy3(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] ^= self.v[y];
        if self.profile == Profile::SuperChip11 {
            self.v[0xF] = 0;
        }
    }
    fn op_8xy4(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (new_vx, carry) = self.v[x].overflowing_add(self.v[y]);
        let new_vf = if carry { 1 } else { 0 };
        self.v[x] = new_vx;
        self.v[0xF] = new_vf;
    }
    fn op_8xy5(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (new_vx, borrow) = self.v[x].overflowing_sub(self.v[y]);
        let new_vf = if borrow { 0 } else { 1 };
        self.v[x] = new_vx;
        self.v[0xF] = new_vf;
    }
    fn op_8xy6(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let vx = self.v[x];
        match self.profile {
            Profile::Classic | Profile::SuperChip11 => {
                self.v[x] = vx >> 1;
                self.v[0xF] = vx & 1;
            }
        }
    }
    fn op_8xy7(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[0xF] = u8::from(!borrow);
    }
    fn op_8xye(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let vx = self.v[x];
        match self.profile {
            Profile::Classic | Profile::SuperChip11 => {
                self.v[x] = vx << 1;
                self.v[0xF] = vx >> 7;
            }
        }
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
        match self.profile {
            Profile::Classic => self.pc = (opcode & 0x0FFF) + self.v[0] as u16,
            Profile::SuperChip11 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                self.pc = (opcode & 0x00FF) + self.v[x] as u16;
            }
        }
    }
    fn op_cxkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = self.rng.gen::<u8>() & kk;
    }
    fn op_dxyn(&mut self, opcode: u16) {
        let x_register = ((opcode & 0x0F00) >> 8) as usize;
        let y_register = ((opcode & 0x00F0) >> 4) as usize;
        let rows = (opcode & 0x000F) as usize;
        if rows == 0 {
            self.draw_high_sprite(self.v[x_register] as usize, self.v[y_register] as usize);
            return;
        }

        let scale = if self.display_mode == DisplayMode::Low {
            2
        } else {
            1
        };
        let (width, height) = self.active_dimensions();
        let mut collided = false;
        for row in 0..rows {
            let y = self.v[y_register] as usize + row;
            for column in 0..8 {
                if self.memory[self.i + row] & (0x80 >> column) == 0 {
                    continue;
                }
                let x = self.v[x_register] as usize + column;
                let (x, y) = match self.profile {
                    Profile::Classic => (x % width, y % height),
                    Profile::SuperChip11 if x < width && y < height => (x, y),
                    Profile::SuperChip11 => continue,
                };
                for physical_y in y * scale..(y + 1) * scale {
                    for physical_x in x * scale..(x + 1) * scale {
                        let index = physical_x + physical_y * PHYSICAL_SCREEN_WIDTH;
                        collided |= self.screen[index];
                        self.screen[index] ^= true;
                    }
                }
            }
        }
        self.v[0xF] = u8::from(collided);
    }

    fn draw_high_sprite(&mut self, x: usize, y: usize) {
        let mut affected_rows = 0;
        for row in 0..16 {
            let screen_y = y + row;
            if screen_y >= PHYSICAL_SCREEN_HEIGHT {
                affected_rows += 1;
                continue;
            }
            let sprite = u16::from_be_bytes([
                self.memory[self.i + row * 2],
                self.memory[self.i + row * 2 + 1],
            ]);
            let mut collided = false;
            for column in 0..16 {
                let screen_x = x + column;
                if sprite & (0x8000 >> column) == 0 || screen_x >= PHYSICAL_SCREEN_WIDTH {
                    continue;
                }
                let index = screen_x + screen_y * PHYSICAL_SCREEN_WIDTH;
                collided |= self.screen[index];
                self.screen[index] ^= true;
            }
            affected_rows += u8::from(collided);
        }
        self.v[0xF] = affected_rows;
    }
    fn op_ex9e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;
        if self.keys.get(key).copied().unwrap_or(false) {
            self.pc += 2;
        }
    }
    fn op_exa1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;
        if !self.keys.get(key).copied().unwrap_or(false) {
            self.pc += 2;
        }
    }
    fn op_fx07(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[x] = self.dt;
    }
    fn op_fx0a(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        if let Some(key) = self.waiting_for_key_release {
            if !self.keys[key as usize] {
                self.v[x] = key;
                self.waiting_for_key_release = None;
                return;
            }
            self.pc -= 2;
            return;
        }

        for i in 0..16 {
            if self.keys[i] && !self.prev_keys[i] {
                self.waiting_for_key_release = Some(i as u8);
                self.pc -= 2;
                return;
            }
        }

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
    fn op_fx30(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i = HIGH_FONT_BASE + self.v[x] as usize * HIGH_FONT_HEIGHT;
    }
    fn op_fx33(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.memory[self.i] = self.v[x] / 100;
        self.memory[self.i + 1] = (self.v[x] / 10) % 10;
        self.memory[self.i + 2] = self.v[x] % 10;
    }
    fn op_fx55(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        match self.profile {
            Profile::Classic | Profile::SuperChip11 => {
                for i in 0..=x {
                    self.memory[self.i + i] = self.v[i];
                }
            }
        }
    }
    fn op_fx65(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        match self.profile {
            Profile::Classic | Profile::SuperChip11 => {
                for i in 0..=x {
                    self.v[i] = self.memory[self.i + i];
                }
            }
        }
    }
    fn op_fx75(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.rpl[..=x].copy_from_slice(&self.v[..=x]);
    }
    fn op_fx85(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[..=x].copy_from_slice(&self.rpl[..=x]);
    }

    #[cfg(test)]
    pub fn execute_opcode(&mut self, opcode: u16) -> StepOutcome {
        if self.halted {
            return StepOutcome::Halted;
        }
        self.memory[self.pc as usize] = (opcode >> 8) as u8;
        self.memory[self.pc as usize + 1] = (opcode & 0xFF) as u8;
        self.decode_opcode()
    }

    #[cfg(test)]
    pub fn get_v(&self, x: usize) -> u8 {
        self.v[x]
    }

    #[cfg(test)]
    pub fn set_v(&mut self, x: usize, val: u8) {
        self.v[x] = val;
    }

    #[cfg(test)]
    pub fn get_i(&self) -> usize {
        self.i
    }

    #[cfg(test)]
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    #[cfg(test)]
    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superchip_profile_outcome_contract() {
        assert_eq!("superchip-1.1".parse::<Profile>(), Ok(Profile::SuperChip11));
        assert!("SuperChip-1.1".parse::<Profile>().is_err());
        assert!("not-a-profile".parse::<Profile>().is_err());

        let mut cpu = Cpu::with_seed(Profile::Classic, 7);
        assert_eq!(cpu.execute_opcode(0x6001), StepOutcome::Executed);
        assert_eq!(cpu.v[0], 1);

        fn assert_unsupported_without_mutation(profile: Profile, opcode: u16) {
            let mut cpu = Cpu::with_seed(profile, 7);
            cpu.pc = 0x300;
            cpu.sp = 1;
            cpu.stack[0] = 0x234;
            cpu.screen[3] = true;
            cpu.keys[4] = true;
            cpu.prev_keys[5] = true;
            cpu.waiting_for_key_release = Some(6);
            cpu.v[7] = 8;
            cpu.i = 0x345;
            cpu.st = 9;
            cpu.dt = 10;
            cpu.memory[cpu.pc as usize] = (opcode >> 8) as u8;
            cpu.memory[cpu.pc as usize + 1] = opcode as u8;

            let pc = cpu.pc;
            let sp = cpu.sp;
            let stack = cpu.stack;
            let screen = cpu.screen;
            let display_mode = cpu.display_mode;
            let keys = cpu.keys;
            let prev_keys = cpu.prev_keys;
            let waiting_for_key_release = cpu.waiting_for_key_release;
            let v = cpu.v;
            let i = cpu.i;
            let st = cpu.st;
            let dt = cpu.dt;
            let memory = cpu.memory;
            let rpl = cpu.rpl;
            let halted = cpu.halted;
            let mut expected_rng = cpu.rng.clone();

            assert_eq!(cpu.decode_opcode(), StepOutcome::Unsupported);
            assert_eq!(cpu.profile, profile);
            assert_eq!(cpu.pc, pc);
            assert_eq!(cpu.sp, sp);
            assert_eq!(cpu.stack, stack);
            assert_eq!(cpu.screen, screen);
            assert_eq!(cpu.display_mode, display_mode);
            assert_eq!(cpu.keys, keys);
            assert_eq!(cpu.prev_keys, prev_keys);
            assert_eq!(cpu.waiting_for_key_release, waiting_for_key_release);
            assert_eq!(cpu.v, v);
            assert_eq!(cpu.i, i);
            assert_eq!(cpu.st, st);
            assert_eq!(cpu.dt, dt);
            assert_eq!(cpu.memory, memory);
            assert_eq!(cpu.rpl, rpl);
            assert_eq!(cpu.halted, halted);

            let mut actual_rng = cpu.rng.clone();
            assert_eq!(actual_rng.gen::<u64>(), expected_rng.gen::<u64>());
        }

        assert_unsupported_without_mutation(Profile::Classic, 0x00FD);
        assert_unsupported_without_mutation(Profile::SuperChip11, 0x5011);

        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        assert_eq!(cpu.execute_opcode(0x00FD), StepOutcome::Halted);
        assert!(cpu.halted);
        let pc = cpu.pc;
        let memory = cpu.memory;
        assert_eq!(cpu.tick(), StepOutcome::Halted);
        assert_eq!(cpu.pc, pc);
        assert_eq!(cpu.memory, memory);
    }

    #[test]
    fn seeded_random_sequence_repeats_after_reset() {
        let mut first = Cpu::with_seed(Profile::Classic, 42);
        let mut second = Cpu::with_seed(Profile::Classic, 42);

        let mut first_sequence = [0; 4];
        let mut second_sequence = [0; 4];
        for value in &mut first_sequence {
            assert_eq!(first.execute_opcode(0xC0FF), StepOutcome::Executed);
            *value = first.v[0];
        }
        for value in &mut second_sequence {
            assert_eq!(second.execute_opcode(0xC0FF), StepOutcome::Executed);
            *value = second.v[0];
        }
        assert_eq!(first_sequence, second_sequence);

        first.reset();
        let mut reset_sequence = [0; 4];
        for value in &mut reset_sequence {
            assert_eq!(first.execute_opcode(0xC0FF), StepOutcome::Executed);
            *value = first.v[0];
        }
        assert_eq!(reset_sequence, first_sequence);
    }

    #[test]
    fn superchip_display_low_mode_uses_physical_pixels_and_preserves_them_across_modes() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        assert_eq!(cpu.active_dimensions(), (64, 32));
        assert_eq!(cpu.get_display().len(), 128 * 64);

        cpu.i = 0x300;
        cpu.memory[0x300] = 0x80;
        assert_eq!(cpu.execute_opcode(0xD011), StepOutcome::Executed);
        for (x, y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            assert!(cpu.screen[x + y * 128]);
        }

        let pixels = cpu.screen;
        assert_eq!(cpu.execute_opcode(0x00FF), StepOutcome::Executed);
        assert_eq!(cpu.active_dimensions(), (128, 64));
        assert_eq!(cpu.screen, pixels);
        assert_eq!(cpu.execute_opcode(0x00FE), StepOutcome::Executed);
        assert_eq!(cpu.active_dimensions(), (64, 32));
        assert_eq!(cpu.screen, pixels);
    }

    #[test]
    fn superchip_display_scrolls_physical_rows_and_columns_with_vacated_strips() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        cpu.screen[0] = true;
        cpu.screen[127] = true;
        cpu.screen[128] = true;

        assert_eq!(cpu.execute_opcode(0x00C1), StepOutcome::Executed);
        assert!(!cpu.screen[..128].iter().any(|pixel| *pixel));
        assert!(cpu.screen[128]);
        assert!(cpu.screen[255]);
        assert!(cpu.screen[256]);

        cpu.screen = [false; 128 * 64];
        cpu.screen[0] = true;
        assert_eq!(cpu.execute_opcode(0x00CF), StepOutcome::Executed);
        assert!(cpu.screen[15 * 128]);
        assert!(!cpu.screen[..15 * 128].iter().any(|pixel| *pixel));

        cpu.screen = [false; 128 * 64];
        cpu.screen[0] = true;
        cpu.screen[127] = true;
        assert_eq!(cpu.execute_opcode(0x00FB), StepOutcome::Executed);
        assert!(cpu.screen[4]);
        assert!(!cpu.screen[..4].iter().any(|pixel| *pixel));
        assert_eq!(cpu.execute_opcode(0x00FC), StepOutcome::Executed);
        assert!(cpu.screen[0]);
        assert!(!cpu.screen[124..128].iter().any(|pixel| *pixel));
    }

    #[test]
    fn superchip_display_clips_edges_and_counts_affected_high_resolution_rows() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        cpu.i = 0x300;
        cpu.memory[0x300..0x302].copy_from_slice(&[0xFF, 0x80]);
        cpu.v[0] = 63;
        cpu.v[1] = 31;
        assert_eq!(cpu.execute_opcode(0xD012), StepOutcome::Executed);
        assert!(cpu.screen[126 + 62 * 128]);
        assert!(cpu.screen[127 + 63 * 128]);
        assert!(!cpu.screen[62 * 128]);

        assert_eq!(cpu.execute_opcode(0x00FF), StepOutcome::Executed);
        cpu.screen = [false; 128 * 64];
        cpu.i = 0x300;
        cpu.memory[0x300..0x320].fill(0);
        cpu.memory[0x300..0x304].copy_from_slice(&[0x80, 0x01, 0x80, 0x00]);
        cpu.v[0] = 127;
        cpu.v[1] = 63;
        assert_eq!(cpu.execute_opcode(0xD010), StepOutcome::Executed);
        assert!(cpu.screen[127 + 63 * 128]);
        assert_eq!(cpu.v[0xF], 15);

        cpu.v[1] = 62;
        assert_eq!(cpu.execute_opcode(0xD010), StepOutcome::Executed);
        assert_eq!(cpu.v[0xF], 15);
    }

    #[test]
    fn superchip_display_empty_side_clipped_and_colliding_rows_are_distinct() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        cpu.execute_opcode(0x00FF);
        cpu.i = 0x300;
        cpu.memory[0x300..0x320].fill(0);
        assert_eq!(cpu.execute_opcode(0xD010), StepOutcome::Executed);
        assert!(cpu.screen.iter().all(|pixel| !pixel));
        assert_eq!(cpu.v[0xF], 0);

        cpu.memory[0x300..0x302].copy_from_slice(&[0x80, 0x01]);
        cpu.v[0] = 127;
        assert_eq!(cpu.execute_opcode(0xD010), StepOutcome::Executed);
        assert!(cpu.screen[127]);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.execute_opcode(0xD010), StepOutcome::Executed);
        assert!(!cpu.screen[127]);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn superchip_display_rejects_profile_and_sprite_boundaries_before_mutation() {
        for opcode in [0x00C1, 0x00CF, 0x00FB, 0x00FC, 0x00FE, 0x00FF, 0xD010] {
            let mut cpu = Cpu::with_seed(Profile::Classic, 7);
            cpu.memory[cpu.pc as usize] = (opcode >> 8) as u8;
            cpu.memory[cpu.pc as usize + 1] = opcode as u8;
            cpu.screen[63 + 31 * 128] = true;
            let pc = cpu.pc;
            let screen = cpu.screen;

            assert_eq!(cpu.decode_opcode(), StepOutcome::Unsupported, "{opcode:04X}");
            assert_eq!(cpu.pc, pc, "{opcode:04X}");
            assert_eq!(cpu.screen, screen, "{opcode:04X}");
        }

        let mut classic = Cpu::with_seed(Profile::Classic, 7);
        classic.i = 0x300;
        classic.memory[0x300] = 0x81;
        classic.v[0] = 63;
        classic.v[1] = 31;
        assert_eq!(classic.execute_opcode(0xD011), StepOutcome::Executed);
        assert!(classic.screen[126 + 62 * 128]);
        assert!(classic.screen[12 + 62 * 128]);

        for (opcode, i) in [(0xD012, 4095), (0xD010, 4095)] {
            let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
            if opcode == 0xD010 {
                assert_eq!(cpu.execute_opcode(0x00FF), StepOutcome::Executed);
            }
            cpu.i = i;
            cpu.memory[cpu.pc as usize] = (opcode >> 8) as u8;
            cpu.memory[cpu.pc as usize + 1] = opcode as u8;
            let pc = cpu.pc;
            let screen = cpu.screen;
            let vf = cpu.v[0xF];

            assert_eq!(cpu.decode_opcode(), StepOutcome::Unsupported);
            assert_eq!(cpu.pc, pc);
            assert_eq!(cpu.screen, screen);
            assert_eq!(cpu.v[0xF], vf);
        }
    }

    #[test]
    fn superchip_state_high_font_endpoints_and_invalid_glyphs_are_bounded() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        assert_eq!(
            &cpu.memory[HIGH_FONT_BASE..HIGH_FONT_BASE + HIGH_FONTSET.len()],
            &HIGH_FONTSET
        );

        for (digit, expected_i) in [
            (0, HIGH_FONT_BASE),
            (9, HIGH_FONT_BASE + 9 * HIGH_FONT_HEIGHT),
        ] {
            cpu.v[3] = digit;
            assert_eq!(cpu.execute_opcode(0xF330), StepOutcome::Executed);
            assert_eq!(cpu.i, expected_i);
        }

        for digit in [10, 15] {
            cpu.v[3] = digit;
            cpu.i = 0x345;
            let pc = cpu.pc;
            assert_eq!(cpu.execute_opcode(0xF330), StepOutcome::Unsupported);
            assert_eq!(cpu.pc, pc);
            assert_eq!(cpu.i, 0x345);
        }
    }

    #[test]
    fn superchip_state_rpl_uses_exactly_eight_reset_cleared_bytes() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        cpu.v[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(cpu.execute_opcode(0xF075), StepOutcome::Executed);
        assert_eq!(cpu.rpl, [1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(cpu.execute_opcode(0xF775), StepOutcome::Executed);
        assert_eq!(cpu.rpl, [1, 2, 3, 4, 5, 6, 7, 8]);

        cpu.v[..8].fill(0);
        assert_eq!(cpu.execute_opcode(0xF785), StepOutcome::Executed);
        assert_eq!(&cpu.v[..8], &[1, 2, 3, 4, 5, 6, 7, 8]);

        for opcode in [0xF875, 0xFF75, 0xF885, 0xFF85] {
            let rpl = cpu.rpl;
            let registers = cpu.v;
            let pc = cpu.pc;
            assert_eq!(cpu.execute_opcode(opcode), StepOutcome::Unsupported);
            assert_eq!(cpu.pc, pc);
            assert_eq!(cpu.rpl, rpl);
            assert_eq!(cpu.v, registers);
        }

        cpu.reset();
        assert_eq!(cpu.rpl, [0; 8]);
    }

    #[test]
    fn superchip_state_reset_restores_every_mutable_field_and_is_idempotent() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 42);
        cpu.pc = 0x300;
        cpu.sp = 2;
        cpu.stack[..2].copy_from_slice(&[0x222, 0x333]);
        cpu.screen[8191] = true;
        cpu.display_mode = DisplayMode::High;
        cpu.keys[15] = true;
        cpu.prev_keys[0] = true;
        cpu.waiting_for_key_release = Some(7);
        cpu.v = [0xAA; 16];
        cpu.i = 0x345;
        cpu.st = 8;
        cpu.dt = 9;
        cpu.memory = [0xCC; 4096];
        cpu.rpl = [0xDD; 8];
        cpu.halted = true;
        cpu.rng.gen::<u64>();

        cpu.reset();
        assert_eq!(cpu.profile, Profile::SuperChip11);
        assert_eq!(cpu.pc, PROGRAM_START);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.stack, [0; 16]);
        assert!(cpu.screen.iter().all(|pixel| !pixel));
        assert_eq!(cpu.display_mode, DisplayMode::Low);
        assert_eq!(cpu.keys, [false; 16]);
        assert_eq!(cpu.prev_keys, [false; 16]);
        assert_eq!(cpu.waiting_for_key_release, None);
        assert_eq!(cpu.v, [0; 16]);
        assert_eq!(cpu.i, 0);
        assert_eq!(cpu.st, 0);
        assert_eq!(cpu.dt, 0);
        assert_eq!(cpu.rpl, [0; 8]);
        assert!(!cpu.halted);
        assert_eq!(&cpu.memory[..FONTSET.len()], &FONTSET);
        assert_eq!(
            &cpu.memory[HIGH_FONT_BASE..HIGH_FONT_BASE + HIGH_FONTSET.len()],
            &HIGH_FONTSET
        );
        assert!(cpu.memory[HIGH_FONT_BASE + HIGH_FONTSET.len()..]
            .iter()
            .all(|byte| *byte == 0));

        let mut expected_rng = Cpu::with_seed(Profile::SuperChip11, 42).rng;
        assert_eq!(cpu.rng.gen::<u64>(), expected_rng.gen::<u64>());
        cpu.reset();
        assert_eq!(cpu.profile, Profile::SuperChip11);
        assert_eq!(cpu.memory[HIGH_FONT_BASE..HIGH_FONT_BASE + HIGH_FONTSET.len()], HIGH_FONTSET);
        assert!(cpu.screen.iter().all(|pixel| !pixel));
        assert_eq!(cpu.rpl, [0; 8]);
    }

    #[test]
    fn superchip_state_uses_the_locked_profile_quirk_bundle() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);

        cpu.v[1] = 0b1000_0011;
        cpu.v[2] = 0b0100_0000;
        cpu.execute_opcode(0x8126);
        assert_eq!((cpu.v[1], cpu.v[2], cpu.v[0xF]), (0b0100_0001, 0b0100_0000, 1));

        cpu.i = 0x300;
        cpu.v[..=2].copy_from_slice(&[1, 2, 3]);
        cpu.execute_opcode(0xF255);
        assert_eq!(cpu.i, 0x300);
        cpu.execute_opcode(0xF265);
        assert_eq!(cpu.i, 0x300);

        cpu.v[1] = 0x40;
        cpu.execute_opcode(0xB123);
        assert_eq!(cpu.pc, 0x63);

        for opcode in [0x8121, 0x8122, 0x8123] {
            cpu.v[1] = 0x0F;
            cpu.v[2] = 0xF0;
            cpu.v[0xF] = 0xAB;
            cpu.execute_opcode(opcode);
            assert_eq!(cpu.v[0xF], 0);
        }
    }

    #[test]
    fn superchip_state_rejects_excluded_encodings_without_mutation() {
        for (opcode, register_value) in [
            (0x00C0, 0),
            (0xD010, 0),
            (0xF030, 10),
            (0xFF30, 15),
            (0xF875, 0),
            (0xFF75, 0),
            (0xF885, 0),
            (0xFF85, 0),
            (0x00D1, 0),
            (0x00B1, 0),
        ] {
            let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
            let x = ((opcode & 0x0F00) >> 8) as usize;
            cpu.v[x] = register_value;
            cpu.memory[cpu.pc as usize] = (opcode >> 8) as u8;
            cpu.memory[cpu.pc as usize + 1] = opcode as u8;
            let pc = cpu.pc;
            let registers = cpu.v;
            let i = cpu.i;
            let rpl = cpu.rpl;
            let screen = cpu.screen;

            assert_eq!(cpu.decode_opcode(), StepOutcome::Unsupported, "{opcode:04X}");
            assert_eq!(cpu.pc, pc, "{opcode:04X}");
            assert_eq!(cpu.v, registers, "{opcode:04X}");
            assert_eq!(cpu.i, i, "{opcode:04X}");
            assert_eq!(cpu.rpl, rpl, "{opcode:04X}");
            assert_eq!(cpu.screen, screen, "{opcode:04X}");
        }
    }

    #[test]
    fn superchip_state_rom_and_sprite_memory_endpoints_are_bounded() {
        let mut cpu = Cpu::with_seed(Profile::SuperChip11, 7);
        let initial_memory = cpu.memory;
        assert_eq!(cpu.load_rom(&[]), Ok(()));
        assert_eq!(cpu.memory, initial_memory);

        assert_eq!(cpu.load_rom(&[0x60]), Ok(()));
        assert_eq!(cpu.tick(), StepOutcome::Executed);
        assert_eq!(cpu.v[0], 0);

        cpu.reset();
        let capacity = cpu.memory.len() - PROGRAM_START as usize;
        assert_eq!(cpu.load_rom(&vec![0xAA; capacity]), Ok(()));
        assert_eq!(cpu.memory[4095], 0xAA);
        let memory = cpu.memory;
        assert!(cpu.load_rom(&vec![0xBB; capacity + 1]).is_err());
        assert_eq!(cpu.memory, memory);

        cpu.reset();
        cpu.i = 4095;
        cpu.memory[4095] = 0x80;
        assert_eq!(cpu.execute_opcode(0xD011), StepOutcome::Executed);
        let pixels = cpu.screen;
        let pc = cpu.pc;
        assert_eq!(cpu.execute_opcode(0xD012), StepOutcome::Unsupported);
        assert_eq!(cpu.pc, pc);
        assert_eq!(cpu.screen, pixels);
    }

    #[test]
    fn profile_parser_accepts_only_classic() {
        assert_eq!("classic".parse::<Profile>(), Ok(Profile::Classic));
        assert!("Classic".parse::<Profile>().is_err());
        assert!("not-a-profile".parse::<Profile>().is_err());
    }

    #[test]
    fn classic_profile_contract() {
        fn modern_vx_shifts(cpu: &mut Cpu) {
            cpu.v[1] = 0b1000_0011;
            cpu.v[2] = 0b0100_0000;
            cpu.execute_opcode(0x8126);
            assert_eq!(cpu.v[1], 0b0100_0001);
            assert_eq!(cpu.v[2], 0b0100_0000);
            assert_eq!(cpu.v[0xF], 1);

            cpu.v[1] = 0b1000_0001;
            cpu.v[2] = 0b0000_0001;
            cpu.execute_opcode(0x812E);
            assert_eq!(cpu.v[1], 0b0000_0010);
            assert_eq!(cpu.v[2], 0b0000_0001);
            assert_eq!(cpu.v[0xF], 1);
        }

        fn unchanged_i_after_memory_transfers(cpu: &mut Cpu) {
            cpu.i = 0x300;
            cpu.v[..=2].copy_from_slice(&[1, 2, 3]);
            cpu.execute_opcode(0xF255);
            assert_eq!(cpu.i, 0x300);
            assert_eq!(&cpu.memory[0x300..=0x302], &[1, 2, 3]);

            cpu.memory[0x300..=0x302].copy_from_slice(&[4, 5, 6]);
            cpu.execute_opcode(0xF265);
            assert_eq!(cpu.i, 0x300);
            assert_eq!(&cpu.v[..=2], &[4, 5, 6]);
        }

        fn v0_based_jump(cpu: &mut Cpu) {
            cpu.v[0] = 0x10;
            cpu.v[1] = 0x40;
            cpu.execute_opcode(0xB123);
            assert_eq!(cpu.pc, 0x133);
        }

        fn preserved_vf_after_logic(cpu: &mut Cpu) {
            for (opcode, x, y, expected) in [
                (0x8121, 0x0F, 0xF0, 0xFF),
                (0x8122, 0x0F, 0xF3, 0x03),
                (0x8123, 0xFF, 0x0F, 0xF0),
            ] {
                cpu.v[1] = x;
                cpu.v[2] = y;
                cpu.v[0xF] = 0xAB;
                cpu.execute_opcode(opcode);
                assert_eq!(cpu.v[1], expected);
                assert_eq!(cpu.v[0xF], 0xAB);
            }
        }

        fn two_axis_sprite_wrapping(cpu: &mut Cpu) {
            cpu.i = 0x300;
            cpu.memory[0x300] = 0b1100_0001;
            cpu.memory[0x301] = 0b1000_0000;
            cpu.v[0] = (SCREEN_WIDTH - 1) as u8;
            cpu.v[1] = (SCREEN_HEIGHT - 1) as u8;
            cpu.execute_opcode(0xD012);

            assert!(cpu.screen[126 + 62 * PHYSICAL_SCREEN_WIDTH]);
            assert!(cpu.screen[62 * PHYSICAL_SCREEN_WIDTH]);
            assert!(cpu.screen[12 + 62 * PHYSICAL_SCREEN_WIDTH]);
            assert!(cpu.screen[126]);
        }

        type ContractCheck = (&'static str, fn(&mut Cpu));
        let checks: [ContractCheck; 5] = [
            ("modern VX shifts", modern_vx_shifts),
            (
                "unchanged I after FX55/FX65",
                unchanged_i_after_memory_transfers,
            ),
            ("V0-based BNNN", v0_based_jump),
            ("preserved VF after logic", preserved_vf_after_logic),
            ("two-axis sprite wrapping", two_axis_sprite_wrapping),
        ];

        for (name, check) in checks {
            let mut cpu = Cpu::new(Profile::Classic);
            check(&mut cpu);
            cpu.reset();
            assert_eq!(cpu.profile, Profile::Classic, "{name}");
        }
    }

    #[test]
    fn test_op_00e0_clear_screen() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.screen = [true; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT];
        cpu.execute_opcode(0x00E0);
        assert!(cpu.screen.iter().all(|&p| !p));
    }

    #[test]
    fn test_op_1nnn_jump() {
        let mut cpu = Cpu::new(Profile::Classic);
        // 1NNN - JP addr
        cpu.execute_opcode(0x1ABC);
        assert_eq!(cpu.get_pc(), 0xABC);
    }

    #[test]
    fn test_op_2nnn_call_subroutine() {
        let mut cpu = Cpu::new(Profile::Classic);
        let initial_pc = cpu.get_pc();
        cpu.execute_opcode(0x2345);
        assert_eq!(cpu.get_pc(), 0x345);
        assert_eq!(cpu.stack[cpu.sp as usize - 1], initial_pc + 2);
    }

    #[test]
    fn test_op_00ee_return() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.execute_opcode(0x2400);
        cpu.execute_opcode(0x00EE);
        assert_eq!(cpu.get_pc(), 0x202);
    }

    #[test]
    fn test_op_3xkk_skip_if_equal() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        let pc_before = cpu.get_pc();
        cpu.execute_opcode(0x3042);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_3xkk_no_skip_if_not_equal() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        let pc_before = cpu.get_pc();
        cpu.execute_opcode(0x3043);
        assert_eq!(cpu.get_pc(), pc_before + 2);
    }

    #[test]
    fn test_op_4xkk_skip_if_not_equal() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        let pc_before = cpu.get_pc();
        // 4XKK - SNE Vx, byte
        cpu.execute_opcode(0x4043);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_5xy0_skip_if_vx_eq_vy() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        cpu.set_v(1, 0x42);
        let pc_before = cpu.get_pc();
        // 5XY0 - SE Vx, Vy
        cpu.execute_opcode(0x5010);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_6xkk_load_byte() {
        let mut cpu = Cpu::new(Profile::Classic);
        // 6XKK - LD Vx, byte
        cpu.execute_opcode(0x6A42);
        assert_eq!(cpu.get_v(0xA), 0x42);
    }

    #[test]
    fn test_op_7xkk_add_byte() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x10);
        // 7XKK - ADD Vx, byte
        cpu.execute_opcode(0x7005);
        assert_eq!(cpu.get_v(0), 0x15);
    }

    #[test]
    fn test_op_7xkk_add_overflow() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0xFF);
        cpu.execute_opcode(0x7002);
        assert_eq!(cpu.get_v(0), 0x01); // Wrap around
    }

    #[test]
    fn test_op_8xy0_load_vy_to_vx() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(1, 0x42);
        // 8XY0 - LD Vx, Vy
        cpu.execute_opcode(0x8010);
        assert_eq!(cpu.get_v(0), 0x42);
    }

    #[test]
    fn test_op_8xy1_or() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x0F);
        cpu.set_v(1, 0xF0);
        // 8XY1 - OR Vx, Vy
        cpu.execute_opcode(0x8011);
        assert_eq!(cpu.get_v(0), 0xFF);
    }

    #[test]
    fn test_op_8xy2_and() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x0F);
        cpu.set_v(1, 0xFF);
        // 8XY2 - AND Vx, Vy
        cpu.execute_opcode(0x8012);
        assert_eq!(cpu.get_v(0), 0x0F);
    }

    #[test]
    fn test_op_8xy3_xor() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0xFF);
        cpu.set_v(1, 0x0F);
        // 8XY3 - XOR Vx, Vy
        cpu.execute_opcode(0x8013);
        assert_eq!(cpu.get_v(0), 0xF0);
    }

    #[test]
    fn test_op_8xy4_add_no_carry() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x10);
        cpu.set_v(1, 0x20);
        // 8XY4 - ADD Vx, Vy
        cpu.execute_opcode(0x8014);
        assert_eq!(cpu.get_v(0), 0x30);
        assert_eq!(cpu.get_v(0xF), 0); // No carry
    }

    #[test]
    fn test_op_8xy4_add_with_carry() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0xFF);
        cpu.set_v(1, 0x02);
        cpu.execute_opcode(0x8014);
        assert_eq!(cpu.get_v(0), 0x01);
        assert_eq!(cpu.get_v(0xF), 1); // Carry
    }

    #[test]
    fn test_op_8xy5_sub_no_borrow() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x20);
        cpu.set_v(1, 0x10);
        // 8XY5 - SUB Vx, Vy
        cpu.execute_opcode(0x8015);
        assert_eq!(cpu.get_v(0), 0x10);
        assert_eq!(cpu.get_v(0xF), 1); // No borrow
    }

    #[test]
    fn test_op_8xy5_sub_with_borrow() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x10);
        cpu.set_v(1, 0x20);
        cpu.execute_opcode(0x8015);
        assert_eq!(cpu.get_v(0), 0xF0); // Wrap
        assert_eq!(cpu.get_v(0xF), 0); // Borrow
    }

    #[test]
    fn test_op_8xy6_shift_right() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0b00000011);
        // 8XY6 - SHR Vx
        cpu.execute_opcode(0x8006);
        assert_eq!(cpu.get_v(0), 0b00000001);
        assert_eq!(cpu.get_v(0xF), 1); // LSB was 1
    }

    #[test]
    fn test_op_8xye_shift_left() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0b10000001);
        // 8XYE - SHL Vx
        cpu.execute_opcode(0x800E);
        assert_eq!(cpu.get_v(0), 0b00000010);
        assert_eq!(cpu.get_v(0xF), 1); // MSB was 1
    }

    #[test]
    fn classic_flag_opcodes_snapshot_vf_alias_operand() {
        let mut cpu = Cpu::new(Profile::Classic);
        let mut flags = [0; 3];

        cpu.set_v(0xF, 0b0000_0011);
        cpu.execute_opcode(0x8FF6);
        flags[0] = cpu.get_v(0xF);

        cpu.reset();
        cpu.set_v(0, 5);
        cpu.set_v(0xF, 3);
        cpu.execute_opcode(0x8F07);
        flags[1] = cpu.get_v(0xF);

        cpu.reset();
        cpu.set_v(0xF, 0b1000_0001);
        cpu.execute_opcode(0x8FFE);
        flags[2] = cpu.get_v(0xF);

        assert_eq!(flags, [1, 1, 1]);
    }

    #[test]
    fn test_op_9xy0_skip_if_vx_ne_vy() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        cpu.set_v(1, 0x43);
        let pc_before = cpu.get_pc();
        // 9XY0 - SNE Vx, Vy
        cpu.execute_opcode(0x9010);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_annn_load_i() {
        let mut cpu = Cpu::new(Profile::Classic);
        // ANNN - LD I, addr
        cpu.execute_opcode(0xA123);
        assert_eq!(cpu.get_i(), 0x123);
    }

    #[test]
    fn test_op_bnnn_jump_v0() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x10);
        // BNNN - JP V0, addr
        cpu.execute_opcode(0xB100);
        assert_eq!(cpu.get_pc(), 0x110);
    }

    #[test]
    fn test_op_fx1e_add_i() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.i = 0x100;
        cpu.set_v(0, 0x10);
        // FX1E - ADD I, Vx
        cpu.execute_opcode(0xF01E);
        assert_eq!(cpu.get_i(), 0x110);
    }

    #[test]
    fn test_op_fx33_bcd() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.i = 0x300;
        cpu.set_v(0, 234);
        // FX33 - LD B, Vx
        cpu.execute_opcode(0xF033);
        assert_eq!(cpu.memory[0x300], 2);
        assert_eq!(cpu.memory[0x301], 3);
        assert_eq!(cpu.memory[0x302], 4);
    }

    #[test]
    fn test_op_fx55_store_registers() {
        let mut cpu = Cpu::new(Profile::Classic);
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
        let mut cpu = Cpu::new(Profile::Classic);
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

    #[test]
    fn test_timer_delay_decrement() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.dt = 10;
        cpu.timers();
        assert_eq!(cpu.dt, 9);
    }

    #[test]
    fn test_timer_sound_decrement() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.st = 5;
        cpu.timers();
        assert_eq!(cpu.st, 4);
    }

    #[test]
    fn test_timer_no_underflow() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.dt = 0;
        cpu.st = 0;
        cpu.timers();
        assert_eq!(cpu.dt, 0);
        assert_eq!(cpu.st, 0);
    }

    #[test]
    fn test_op_fx07_get_delay() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.dt = 0x42;
        // FX07 - LD Vx, DT
        cpu.execute_opcode(0xF007);
        assert_eq!(cpu.get_v(0), 0x42);
    }

    #[test]
    fn test_op_fx15_set_delay() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x30);
        // FX15 - LD DT, Vx
        cpu.execute_opcode(0xF015);
        assert_eq!(cpu.dt, 0x30);
    }

    #[test]
    fn test_op_fx18_set_sound() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x20);
        // FX18 - LD ST, Vx
        cpu.execute_opcode(0xF018);
        assert_eq!(cpu.st, 0x20);
    }

    #[test]
    fn test_screen_initial_state() {
        let cpu = Cpu::new(Profile::Classic);
        assert!(cpu.screen.iter().all(|&p| !p));
    }

    #[test]
    fn test_reset_clears_screen() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.screen[0] = true;
        cpu.screen[100] = true;
        cpu.reset();
        assert!(cpu.screen.iter().all(|&p| !p));
    }

    #[test]
    fn test_op_fx29_font_sprite() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x5); // Sprite for '5'
                           // FX29 - LD F, Vx
        cpu.execute_opcode(0xF029);
        // Font sprite 5 = 5 * 5 = 25
        assert_eq!(cpu.get_i(), 25);
    }

    #[test]
    fn new_cpu_contains_font_glyphs() {
        let cpu = Cpu::new(Profile::Classic);

        assert_eq!(&cpu.memory[..FONTSET.len()], &FONTSET);
    }

    #[test]
    fn reset_restores_font_glyphs() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.memory[0] = 0;

        cpu.reset();

        assert_eq!(&cpu.memory[..FONTSET.len()], &FONTSET);
    }

    #[test]
    fn stack_supports_sixteen_nested_calls() {
        let mut cpu = Cpu::new(Profile::Classic);

        for address in 0x300..0x310 {
            cpu.op_2nnn(0x2000 | address);
        }
        for _ in 0..16 {
            cpu.op_00ee();
        }

        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn key_skip_ignores_values_outside_keypad() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0xFF);
        let pc_before = cpu.get_pc();

        cpu.execute_opcode(0xE09E);

        assert_eq!(cpu.get_pc(), pc_before + 2);
    }

    #[test]
    fn keypress_ignores_values_outside_keypad() {
        let mut cpu = Cpu::new(Profile::Classic);

        cpu.keypress(16, true);

        assert!(cpu.keys.iter().all(|pressed| !pressed));
    }

    #[test]
    fn oversized_rom_is_rejected_without_mutating_memory() {
        let mut cpu = Cpu::new(Profile::Classic);
        let memory_before = cpu.memory;
        let rom = vec![0; cpu.memory.len() - cpu.pc as usize + 1];

        let result = cpu.load_rom(&rom);

        assert!(result.is_err());
        assert_eq!(cpu.memory, memory_before);
    }

    #[test]
    fn sys_instruction_does_not_clear_screen() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.screen[0] = true;

        cpu.execute_opcode(0x0120);

        assert!(cpu.screen[0]);
    }

    #[test]
    fn invalid_5xy_variant_does_not_skip() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        cpu.set_v(1, 0x42);
        let pc_before = cpu.get_pc();

        let outcome = cpu.execute_opcode(0x5011);

        assert_eq!(outcome, StepOutcome::Unsupported);
        assert_eq!(cpu.get_pc(), pc_before);
    }

    #[test]
    fn invalid_9xy_variant_does_not_skip() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        cpu.set_v(1, 0x43);
        let pc_before = cpu.get_pc();

        let outcome = cpu.execute_opcode(0x9011);

        assert_eq!(outcome, StepOutcome::Unsupported);
        assert_eq!(cpu.get_pc(), pc_before);
    }

    #[test]
    fn test_keypress() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.keypress(0x5, true);
        assert!(cpu.keys[0x5]);
        cpu.keypress(0x5, false);
        assert!(!cpu.keys[0x5]);
    }

    #[test]
    fn test_op_ex9e_skip_if_key_pressed() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x5);
        cpu.keypress(0x5, true);
        let pc_before = cpu.get_pc();
        // EX9E - SKP Vx
        cpu.execute_opcode(0xE09E);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }

    #[test]
    fn test_op_exa1_skip_if_key_not_pressed() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x5);
        cpu.keypress(0x5, false);
        let pc_before = cpu.get_pc();
        // EXA1 - SKNP Vx
        cpu.execute_opcode(0xE0A1);
        assert_eq!(cpu.get_pc(), pc_before + 4);
    }
}
