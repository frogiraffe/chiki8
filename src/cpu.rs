use rand::Rng;
use std::path::Path;
use std::str::FromStr;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Classic,
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "classic" => Ok(Self::Classic),
            _ => Err(format!("unknown profile '{value}'; expected 'classic'")),
        }
    }
}

pub struct Cpu {
    profile: Profile,
    pc: u16,
    sp: u8,
    stack: [u16; 16],

    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys: [bool; 16],
    prev_keys: [bool; 16],
    waiting_for_key_release: Option<u8>,
    v: [u8; 16],
    i: usize,

    pub st: u8,
    pub dt: u8,

    memory: [u8; 4096],
}

impl Cpu {
    pub fn get_display(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.screen
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
        let mut cpu = Cpu {
            profile,
            pc: PROGRAM_START,
            stack: [0; 16],
            sp: 0,

            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            keys: [false; 16],
            prev_keys: [false; 16],
            waiting_for_key_release: None,
            v: [0; 16],
            i: 0,

            st: 0,
            dt: 0,

            memory: [0; 4096],
        };
        cpu.set_fontset();
        cpu
    }

    pub fn reset(&mut self) {
        self.pc = PROGRAM_START;
        self.stack = [0; 16];
        self.sp = 0;
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.keys = [false; 16];
        self.prev_keys = [false; 16];
        self.waiting_for_key_release = None;
        self.v = [0; 16];
        self.i = 0;
        self.st = 0;
        self.dt = 0;
        self.memory = [0; 4096];
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
            0x0000 => match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => println!("Unknown opcode: {:X}", opcode),
            },
            0x1000 => self.op_1nnn(opcode),
            0x2000 => self.op_2nnn(opcode),
            0x3000 => self.op_3xkk(opcode),
            0x4000 => self.op_4xkk(opcode),
            0x5000 if opcode & 0x000F == 0 => self.op_5xy0(opcode),
            0x5000 => println!("Unknown opcode: {:X}", opcode),
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
            0x9000 if opcode & 0x000F == 0 => self.op_9xy0(opcode),
            0x9000 => println!("Unknown opcode: {:X}", opcode),
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
        match self.profile {
            Profile::Classic => self.v[x] |= self.v[y],
        }
    }
    fn op_8xy2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        match self.profile {
            Profile::Classic => self.v[x] &= self.v[y],
        }
    }
    fn op_8xy3(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        match self.profile {
            Profile::Classic => self.v[x] ^= self.v[y],
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
        match self.profile {
            Profile::Classic => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
        }
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
        match self.profile {
            Profile::Classic => {
                if self.v[x] & 0x80 != 0 {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
                self.v[x] <<= 1;
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
        }
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
                    let (x, y) = match self.profile {
                        Profile::Classic => (
                            (self.v[x] as usize + x_offset) % SCREEN_WIDTH,
                            (self.v[y] as usize + y_offset as usize) % SCREEN_HEIGHT,
                        ),
                    };
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
    fn op_fx33(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.memory[self.i] = self.v[x] / 100;
        self.memory[self.i + 1] = (self.v[x] / 10) % 10;
        self.memory[self.i + 2] = self.v[x] % 10;
    }
    fn op_fx55(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        match self.profile {
            Profile::Classic => {
                for i in 0..=x {
                    self.memory[self.i + i] = self.v[i];
                }
            }
        }
    }
    fn op_fx65(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        match self.profile {
            Profile::Classic => {
                for i in 0..=x {
                    self.v[i] = self.memory[self.i + i];
                }
            }
        }
    }

    #[cfg(test)]
    pub fn execute_opcode(&mut self, opcode: u16) {
        self.memory[self.pc as usize] = (opcode >> 8) as u8;
        self.memory[self.pc as usize + 1] = (opcode & 0xFF) as u8;
        self.decode_opcode();
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

            assert!(cpu.screen[63 + 31 * SCREEN_WIDTH]);
            assert!(cpu.screen[31 * SCREEN_WIDTH]);
            assert!(cpu.screen[6 + 31 * SCREEN_WIDTH]);
            assert!(cpu.screen[63]);
        }

        let checks: [(&str, fn(&mut Cpu)); 5] = [
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
        cpu.screen = [true; SCREEN_WIDTH * SCREEN_HEIGHT];
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

        cpu.execute_opcode(0x5011);

        assert_eq!(cpu.get_pc(), pc_before + 2);
    }

    #[test]
    fn invalid_9xy_variant_does_not_skip() {
        let mut cpu = Cpu::new(Profile::Classic);
        cpu.set_v(0, 0x42);
        cpu.set_v(1, 0x43);
        let pc_before = cpu.get_pc();

        cpu.execute_opcode(0x9011);

        assert_eq!(cpu.get_pc(), pc_before + 2);
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
