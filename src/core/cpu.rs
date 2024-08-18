use std::fmt::{self, Display, Formatter};

use crate::core::memory;
pub enum RegisterValue {
    A(u8),
    F(u8),
    B(u8),
    C(u8),
    D(u8),
    E(u8),
    H(u8),
    L(u8),
    SP(u16),
    PC(u16),
    AF(u16),
    BC(u16),
    DE(u16),
    HL(u16),
}
pub const FLAG_Z_BIT: u8 = 1 << 7;
pub const FLAG_N_BIT: u8 = 1 << 6;
pub const FLAG_H_BIT: u8 = 1 << 5;
pub const FLAG_C_BIT: u8 = 1 << 4;

pub enum Flag {
    Z(bool),
    N(bool),
    H(bool),
    C(bool),
}
pub struct CPU {
    pub register_a: u8,
    pub register_f: u8,
    pub register_b: u8,
    pub register_c: u8,
    pub register_d: u8,
    pub register_e: u8,
    pub register_h: u8,
    pub register_l: u8,
    pub register_sp: u16,
    pub register_pc: u16,
}

macro_rules! set_register_value {
    ($self:expr, $reg:expr, $($variant:ident => $field:ident),*) => {
        match $reg {
            $(
                RegisterValue::$variant(value) => {
                    $self.$field = *value;
                }
            )*
            _ => {}
        }
    };

    ($self:expr, $reg:expr, $($variant:ident => ($field1:ident, $field2:ident)),*) => {
        match $reg {
            $(
                RegisterValue::$variant(value) => {
                    $self.$field1 = (*value & 0xFF) as u8;
                    $self.$field2 = ((*value >> 8) & 0xFF) as u8;
                }
            )*
            _ => {}
        }
    };
}

impl Display for CPU {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Register\nAF: {:02X}{:02X} \nBC: {:02X}{:02X}\nDE: \
            {:02X}{:02X}\nHL: {:02X}{:02X}\nSP: {:04X}\nPC: {:04X}",
            self.register_a,
            self.register_f,
            self.register_b,
            self.register_c,
            self.register_d,
            self.register_e,
            self.register_h,
            self.register_l,
            self.register_sp,
            self.register_pc
        )
    }
}

macro_rules! ld {
    ($self:expr, $mem:ident, $variant:ident, $get_mem:ident, $len:expr) => {{
        let v = $self.$get_mem($mem);
        $self.set_value(&RegisterValue::$variant(v));
        $len
    }};
    ($self:expr, $to_v:ident, $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            $self.set_value(&RegisterValue::$to_v(v));
        }
        $len
    }};
    ($self:expr, $mem:ident, ff($to_v:ident), $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            if let RegisterValue::$to_v(addr0) = $self.get_value(&RegisterValue::$to_v(0)) {
                let addr = addr0 as u16 + 0xff00;
                $mem.set(addr, v);
            }
        }
        $len
    }};
    ($self:expr, $mem:ident, ($to_v:ident), $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            if let RegisterValue::$to_v(addr) = $self.get_value(&RegisterValue::$to_v(0)) {
                $mem.set(addr, v);
            }
        }
        $len
    }};
    ($self:expr, $mem:ident, $to_v:ident, ($from_v:ident), $len:expr) => {{
        if let RegisterValue::$from_v(addr) = $self.get_value(&RegisterValue::$from_v(0)) {
            $self.set_value(&RegisterValue::$to_v($mem.get(addr)));
        }
        $len
    }};
    ($self:expr, $mem:ident, "(HL)"$mem_shift:tt, $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            if let RegisterValue::HL(addr) = $self.get_value(&RegisterValue::HL(0)) {
                $mem.set(addr, v);
                $self.set_value(&RegisterValue::HL(addr $mem_shift 1))
            }
        }
        $len
    }};
    ($self:expr, $mem:ident, "(a8)", $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            let addr = $self.get_mem_a8($mem);
            $mem.set(addr, v);
        }
        $len
    }};
    ($self:expr, $mem:ident, "(a16)", $from_v:ident, $len:expr) => {{
        if let RegisterValue::$from_v(v) = $self.get_value(&RegisterValue::$from_v(0)) {
            let addr = $self.get_mem_u16($mem);
            $mem.set(addr, v);
        }
        $len
    }};
}
macro_rules! ldh {
    ($self:expr, $mem:ident, $to_v:ident, "(a8)", $len:expr) => {{
        let addr = $self.get_mem_u8($mem) as u16 + 0xff00;
        let v = $mem.get(addr);
        $self.set_value(&RegisterValue::$to_v(v));
        $len
    }};
}
macro_rules! push {
    ($self:expr, $mem:ident, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            $self.register_sp -= 1;
            $mem.set($self.register_sp, (v >> 8) as u8);
            $self.register_sp -= 1;
            $mem.set($self.register_sp, (v & 0xff) as u8);
        }
        $len
    }};
}

macro_rules! pop {
    ($self:expr, $mem:ident, $reg:ident, $len:expr) => {{
        let v =
            $mem.get($self.register_sp) as u16 + (($mem.get($self.register_sp + 1) as u16) << 8);
        $self.set_value(&RegisterValue::$reg(v));
        $self.register_sp += 2;
        $len
    }};
}

macro_rules! dec {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let h = ((v & 0xf) == 0);
            let v = v - 1;
            $self.set_value(&RegisterValue::$reg(v));
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(true));
            $self.set_flag(&Flag::H(h));
        }
        $len
    }};
}

macro_rules! inc {
    ($self:expr, $reg:ident, 4) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let v = v + 1;
            $self.set_value(&RegisterValue::$reg(v));
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H((v & 0xf) == 0));
        }
        4
    }};
    ($self:expr, $reg:ident, 8) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let v = v + 1;
            $self.set_value(&RegisterValue::$reg(v));
        }
        8
    }};
}

macro_rules! xor {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let z = ($self.register_a ^ v) == 0;
            $self.set_flag(&Flag::Z(z));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(false));
            $self.set_flag(&Flag::C(false));
        }
        $len
    }};
}
macro_rules! sub {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let z = ($self.register_a == v);
            let h = ($self.register_a & 0xf) < (v & 0xf);
            let c = $self.register_a < v;
            $self.register_a -= v;
            $self.set_flag(&Flag::Z(z));
            $self.set_flag(&Flag::N(true));
            $self.set_flag(&Flag::H(h));
            $self.set_flag(&Flag::C(c));
        }
        $len
    }};
}
macro_rules! bit {
    ($self:expr, $reg:ident, $shift:expr, $len:expr) => {{
        let v = ($self.$reg) & (1 << $shift);
        $self.set_flag(&Flag::Z(v == 0));
        $self.set_flag(&Flag::N(false));
        $self.set_flag(&Flag::H(true));
        $len
    }};
}
macro_rules! check_condition {
    ($self:expr, $flag:ident) => {{
        $self.get_flag(&Flag::$flag(false))
    }};
    ($self:expr, "N", $flag:ident) => {{
        !$self.get_flag(&Flag::$flag(false))
    }};
}

macro_rules! call {
    ($self:expr, $mem:ident, "a16", $len:expr) => {{
        let v = $self.get_mem_u16($mem);
        $self.register_sp -= 1;
        $mem.set($self.register_sp, ($self.register_pc >> 8) as u8);
        $self.register_sp -= 1;
        $mem.set($self.register_sp, ($self.register_pc & 0xff) as u8);
        $self.register_pc = v;
        $len
    }};
}

macro_rules! rl {
    ($self:expr, $reg:ident, $len:expr) => {{
        if let RegisterValue::$reg(v) = $self.get_value(&RegisterValue::$reg(0)) {
            let c = (v >= 128);
            let mut v = v << 1;
            if $self.get_flag(&Flag::C(false)) {
                v += 1;
            }
            $self.set_value(&RegisterValue::$reg(v));
            // set flag
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(false));
            $self.set_flag(&Flag::C(c));
        }
        $len
    }};
}

macro_rules! jr {
    ($self:expr, $mem:ident, $len:expr) => {{
        let addr = $self.get_pc_and_move();
        let v = ($mem.get(addr) as i8) as i16;
        $self.register_pc = ($self.register_pc as i16 + v) as u16;
        $len
    }};
    ($self:expr, $mem:ident, $flag:ident, $len0:expr, $len1:expr) => {{
        let addr = $self.get_pc_and_move();
        let v = ($mem.get(addr) as i8) as i16;
        let c = check_condition!($self, $flag);
        if c {
            $self.register_pc = ($self.register_pc as i16 + v) as u16;
            $len0
        } else {
            $len1
        }
    }};
    ($self:expr, $mem:ident, "N", $flag:ident, $len0:expr, $len1:expr) => {{
        let addr = $self.get_pc_and_move();
        let v = ($mem.get(addr) as i8) as i16;
        let c = check_condition!($self, "N", $flag);
        if c {
            $self.register_pc = ($self.register_pc as i16 + v) as u16;
            $len0
        } else {
            $len1
        }
    }};
}

fn rla(cpu: &mut CPU, len: u8) -> u8 {
    let c = cpu.register_a >= 128;
    let mut v = cpu.register_a << 1;
    if cpu.get_flag(&Flag::C(false)) {
        v += 1;
    }
    cpu.register_a = v;
    cpu.set_flag(&Flag::Z(false));
    cpu.set_flag(&Flag::N(false));
    cpu.set_flag(&Flag::H(false));
    cpu.set_flag(&Flag::C(c));
    len
}

fn ret(cpu: &mut CPU, mem: &memory::Memory) -> u8 {
    let v = mem.get(cpu.register_sp) as u16 + ((mem.get(cpu.register_sp + 1) as u16) << 8);
    cpu.register_sp += 2;
    cpu.register_pc = v;
    16
}

fn cp_impl(cpu: &mut CPU, t: u8) {
    let z = cpu.register_a == t;
    let h = (cpu.register_a & 0xf) < (t & 0xf);
    let c = cpu.register_a < t;
    cpu.set_flag(&Flag::Z(z));
    cpu.set_flag(&Flag::N(true));
    cpu.set_flag(&Flag::H(h));
    cpu.set_flag(&Flag::C(c));
}

macro_rules! cp {
    ($self:expr, $mem:ident, "d8", $len:expr) => {{
        let t = $self.get_mem_u8($mem);
        cp_impl($self, t);
        $len
    }};
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            register_f: 0,
            register_b: 0,
            register_c: 0,
            register_d: 0,
            register_e: 0,
            register_h: 0,
            register_l: 0,
            register_sp: 0,
            register_pc: 0,
        }
    }
    pub fn set_flag(&mut self, flag: &Flag) {
        let (v, flag_bit) = match flag {
            Flag::Z(v) => (v, FLAG_Z_BIT),
            Flag::N(v) => (v, FLAG_N_BIT),
            Flag::H(v) => (v, FLAG_H_BIT),
            Flag::C(v) => (v, FLAG_C_BIT),
        };
        if *v {
            self.register_f = self.register_f | flag_bit;
        } else {
            self.register_f = self.register_f & (0xff ^ flag_bit);
        }
    }

    pub fn set_value(&mut self, reg: &RegisterValue) {
        set_register_value!(self, reg,
            AF => (register_f, register_a),
            BC => (register_c, register_b),
            DE => (register_e, register_d),
            HL => (register_l, register_h)
        );

        set_register_value!(self, reg,
            A => register_a,
            F => register_f,
            B => register_b,
            C => register_c,
            D => register_d,
            E => register_e,
            H => register_h,
            L => register_l,
            SP => register_sp,
            PC => register_pc
        );
    }
    pub fn get_value(&self, reg: &RegisterValue) -> RegisterValue {
        match reg {
            RegisterValue::A(_) => RegisterValue::A(self.register_a),
            RegisterValue::F(_) => RegisterValue::F(self.register_f),
            RegisterValue::B(_) => RegisterValue::B(self.register_b),
            RegisterValue::C(_) => RegisterValue::C(self.register_c),
            RegisterValue::D(_) => RegisterValue::D(self.register_d),
            RegisterValue::E(_) => RegisterValue::E(self.register_e),
            RegisterValue::H(_) => RegisterValue::H(self.register_h),
            RegisterValue::L(_) => RegisterValue::L(self.register_l),
            RegisterValue::SP(_) => RegisterValue::SP(self.register_sp),
            RegisterValue::PC(_) => RegisterValue::PC(self.register_pc),
            RegisterValue::AF(_) => {
                RegisterValue::AF(((self.register_a as u16) << 8) + self.register_f as u16)
            }
            RegisterValue::BC(_) => {
                RegisterValue::BC(((self.register_b as u16) << 8) + self.register_c as u16)
            }
            RegisterValue::DE(_) => {
                RegisterValue::DE(((self.register_d as u16) << 8) + self.register_e as u16)
            }
            RegisterValue::HL(_) => {
                RegisterValue::HL(((self.register_h as u16) << 8) + self.register_l as u16)
            }
        }
    }
    pub fn get_flag(&self, flag: &Flag) -> bool {
        match flag {
            Flag::Z(_) => return (self.register_f & 0b10000000) != 0,
            Flag::N(_) => return (self.register_f & 0b01000000) != 0,
            Flag::H(_) => return (self.register_f & 0b00100000) != 0,
            Flag::C(_) => return (self.register_f & 0b00010000) != 0,
        }
    }
    fn get_pc_and_move(&mut self) -> u16 {
        let v = self.register_pc;
        self.register_pc += 1;
        v
    }
    fn get_mem_u8(&mut self, mem: &memory::Memory) -> u8 {
        mem.get(self.get_pc_and_move())
    }
    fn get_mem_a8(&mut self, mem: &memory::Memory) -> u16 {
        self.get_mem_u8(mem) as u16 + 0xff00
    }

    fn get_mem_u16(&mut self, mem: &memory::Memory) -> u16 {
        let v0 = self.get_mem_u8(mem) as u16;
        let v1 = self.get_mem_u8(mem) as u16;
        (v1 << 8) + v0
    }

    /// return cpu cycle in 4 MHz
    pub fn tick(&mut self, mem: &mut memory::Memory) -> u8 {
        let op_addr: u8 = mem.get(self.get_pc_and_move());
        // println!("instruction {:02x}", op_addr);
        let cpu_cycle_in_16mhz = match op_addr {
            0xcb => {
                let cb_op_addr: u8 = mem.get(self.get_pc_and_move());
                // println!("cb instruction {:02x}", cb_op_addr);
                match cb_op_addr {
                    0x11 => rl!(self, C, 8),
                    0x7c => return bit!(self, register_h, 7, 8),
                    _ => todo!("cb opcode 0x{:02X} \n{}", cb_op_addr, self),
                }
            }
            0x01 => return ld!(self, mem, BC, get_mem_u16, 12),
            0x03 => return inc!(self, BC, 8),
            0x04 => return inc!(self, B, 4),
            0x05 => return dec!(self, B, 4),
            0x06 => return ld!(self, mem, B, get_mem_u8, 8),
            0x0c => return inc!(self, C, 4),
            0x0d => return dec!(self, C, 4),
            0x0e => return ld!(self, mem, C, get_mem_u8, 8),
            0x11 => return ld!(self, mem, DE, get_mem_u16, 12),
            0x13 => return inc!(self, DE, 8),
            0x14 => return inc!(self, D, 4),
            0x15 => return dec!(self, D, 4),
            0x16 => return ld!(self, mem, D, get_mem_u8, 8),
            0x17 => return rla(self, 4),
            0x18 => return jr!(self, mem, 12),
            0x1a => return ld!(self, mem, A, (DE), 8),
            0x1c => return inc!(self, E, 4),
            0x1d => return dec!(self, E, 4),
            0x1e => return ld!(self, mem, E, get_mem_u8, 8),
            0x20 => return jr!(self, mem, "N", Z, 12, 8),
            0x21 => return ld!(self, mem, HL, get_mem_u16, 12),
            0x22 => return ld!(self, mem, "(HL)"+, A, 8),
            0x23 => return inc!(self, HL, 8),
            0x24 => return inc!(self, H, 4),
            0x25 => return dec!(self, H, 4),
            0x26 => return ld!(self, mem, H, get_mem_u8, 8),
            0x28 => return jr!(self, mem, Z, 12, 8),
            0x2c => return inc!(self, L, 4),
            0x2d => return dec!(self, L, 4),
            0x2e => return ld!(self, mem, L, get_mem_u8, 8),
            0x30 => return jr!(self, mem, "N", C, 12, 8),
            0x31 => return ld!(self, mem, SP, get_mem_u16, 12),
            0x32 => return ld!(self, mem, "(HL)"-, A, 8),
            0x33 => return inc!(self, SP, 8),
            0x38 => return jr!(self, mem, C, 12, 8),
            0x3c => return inc!(self, A, 4),
            0x3d => return dec!(self, A, 4),
            0x3e => return ld!(self, mem, A, get_mem_u8, 8),
            0x40 => return ld!(self, B, B, 4),
            0x41 => return ld!(self, B, C, 4),
            0x42 => return ld!(self, B, D, 4),
            0x43 => return ld!(self, B, E, 4),
            0x44 => return ld!(self, B, H, 4),
            0x45 => return ld!(self, B, L, 4),
            0x47 => return ld!(self, B, A, 4),
            0x4a => return ld!(self, C, D, 4),
            0x4b => return ld!(self, C, E, 4),
            0x4c => return ld!(self, C, H, 4),
            0x4d => return ld!(self, C, L, 4),
            0x4f => return ld!(self, C, A, 4),
            0x50 => return ld!(self, D, B, 4),
            0x51 => return ld!(self, D, C, 4),
            0x52 => return ld!(self, D, D, 4),
            0x53 => return ld!(self, D, E, 4),
            0x54 => return ld!(self, D, H, 4),
            0x55 => return ld!(self, D, L, 4),
            0x57 => return ld!(self, D, A, 4),
            0x60 => return ld!(self, H, B, 4),
            0x61 => return ld!(self, H, C, 4),
            0x62 => return ld!(self, H, D, 4),
            0x63 => return ld!(self, H, E, 4),
            0x64 => return ld!(self, H, H, 4),
            0x65 => return ld!(self, H, L, 4),
            0x67 => return ld!(self, H, A, 4),
            0x68 => return ld!(self, L, B, 4),
            0x70 => return ld!(self, mem, (HL), B, 8),
            0x71 => return ld!(self, mem, (HL), C, 8),
            0x72 => return ld!(self, mem, (HL), D, 8),
            0x73 => return ld!(self, mem, (HL), E, 8),
            0x74 => return ld!(self, mem, (HL), H, 8),
            0x75 => return ld!(self, mem, (HL), L, 8),
            0x77 => return ld!(self, mem, (HL), A, 8),
            0x78 => return ld!(self, A, B, 4),
            0x79 => return ld!(self, A, C, 4),
            0x7a => return ld!(self, A, D, 4),
            0x7b => return ld!(self, A, E, 4),
            0x7c => return ld!(self, A, H, 4),
            0x90 => return sub!(self, B, 4),
            0x91 => return sub!(self, C, 4),
            0x92 => return sub!(self, D, 4),
            0x93 => return sub!(self, E, 4),
            0x94 => return sub!(self, H, 4),
            0x95 => return sub!(self, L, 4),
            0xaf => return xor!(self, A, 4),
            0xc1 => return pop!(self, mem, BC, 12),
            0xc5 => return push!(self, mem, BC, 16),
            0xc9 => return ret(self, mem),
            0xcd => return call!(self, mem, "a16", 24),
            0xd1 => return pop!(self, mem, DE, 12),
            0xe0 => return ld!(self, mem, "(a8)", A, 12),
            0xe1 => return pop!(self, mem, HL, 12),
            0xe2 => return ld!(self, mem, ff(C), A, 8),
            0xea => return ld!(self, mem, "(a16)", A, 16),
            0xf0 => return ldh!(self, mem, A, "(a8)", 12),
            0xfe => return cp!(self, mem, "d8", 8),
            _ => todo!("opcode 0x{:02X} \n{}", op_addr, self),
        };
        cpu_cycle_in_16mhz / 4
    }
}
