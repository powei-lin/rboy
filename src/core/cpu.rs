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
            "Register\nAF: {:02x}{:02x} \nBC: {:02x}{:02x}\nDE: \
            {:02x}{:02x}\nHL: {:02x}{:02x}\nSP: {:04x}\nPC: {:04x}",
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
    ($self:expr, $mem:ident, "(HL-)", $variant:ident, $len:expr) => {{
        let v = $self.$variant;
        if let RegisterValue::HL(addr) = $self.get_value(&RegisterValue::HL(0)) {
            $mem.set(addr, v);
            $self.set_value(&RegisterValue::HL(addr - 1))
        }
        $len
    }};
    ($self:expr, $mem:ident, "(a8)", $variant:ident, $len:expr) => {{
        let v = $self.$variant;
        let addr = $mem.get($self.get_pc_and_move()) as u16 + 0xff00;
        $mem.set(addr, v);
        $len
    }};
    ($self:expr, $mem:ident, $to_v:ident, "d8", $len:expr) => {{
        let v = $self.get_mem_u8($mem);
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

macro_rules! inc {
    ($self:expr, $reg:ident, $len:expr) => {{
        $self.$reg += 1;
        $self.set_flag(&Flag::Z($self.$reg == 0));
        $self.set_flag(&Flag::N(false));
        $self.set_flag(&Flag::H(($self.$reg & 0xf) == 0));
        $len
    }};
}

macro_rules! xor {
    ($self:expr, $reg:ident, $len:expr) => {{
        let v = $self.register_a ^ $self.$reg;
        $self.set_flag(&Flag::Z(v == 0));
        $self.set_flag(&Flag::N(false));
        $self.set_flag(&Flag::H(false));
        $self.set_flag(&Flag::C(false));
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
            let c = (v >= 0b1000000);
            let mut v = v << 1;
            if $self.get_flag(&Flag::C(false)) {
                v += 1;
            }
            $self.set_value(&RegisterValue::$reg(v));
            // set flag
            $self.set_flag(&Flag::Z(v == 0));
            $self.set_flag(&Flag::N(false));
            $self.set_flag(&Flag::H(true));
            $self.set_flag(&Flag::C(c));
        }
        $len
    }};
}

macro_rules! jr {
    ($self:expr, $mem:ident, "N", $flag:ident, $len0:expr, $len1:expr) => {{
        let addr = $self.get_pc_and_move();
        let v = ($mem.get(addr) as i8) as i16;
        let c = check_condition!($self, "N", $flag);
        if c {
            $self.register_pc = ($self.register_pc as i16 + v) as u16;
            return $len0;
        } else {
            return $len1;
        }
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

    fn get_mem_u16(&mut self, mem: &memory::Memory) -> u16 {
        let v0 = self.get_mem_u8(mem) as u16;
        let v1 = self.get_mem_u8(mem) as u16;
        (v1 << 8) + v0
    }

    pub fn tick(&mut self, mem: &mut memory::Memory) -> u8 {
        let op_addr: u8 = mem.get(self.get_pc_and_move());
        println!("instruction {:02x}", op_addr);
        match op_addr {
            0xcb => {
                let cb_op_addr: u8 = mem.get(self.get_pc_and_move());
                println!("cb instruction {:02x}", cb_op_addr);
                match cb_op_addr {
                    0x11 => rl!(self, C, 8),
                    0x7c => return bit!(self, register_h, 7, 8),
                    _ => todo!("cb opcode 0x{:02X} \n{}", cb_op_addr, self),
                }
            }
            0x01 => return ld!(self, mem, BC, get_mem_u16, 12),
            0x06 => return ld!(self, mem, B, "d8", 8),
            0x0c => return inc!(self, register_c, 4),
            0x0e => return ld!(self, mem, C, get_mem_u8, 8),
            0x11 => return ld!(self, mem, DE, get_mem_u16, 12),
            0x1a => return ld!(self, mem, A, (DE), 8),
            0x20 => return jr!(self, mem, "N", Z, 12, 8),
            0x21 => return ld!(self, mem, HL, get_mem_u16, 12),
            0x31 => return ld!(self, mem, SP, get_mem_u16, 12),
            0x32 => return ld!(self, mem, "(HL-)", register_a, 8),
            0x3e => return ld!(self, mem, A, get_mem_u8, 8),
            0x4f => return ld!(self, C, A, 4),
            0x77 => return ld!(self, mem, (HL), A, 8),
            0xaf => return xor!(self, register_a, 4),
            0xc5 => return push!(self, mem, BC, 16),
            0xcd => return call!(self, mem, "a16", 24),
            0xe0 => return ld!(self, mem, "(a8)", register_a, 12),
            0xe2 => return ld!(self, mem, ff(C), A, 8),
            _ => todo!("opcode 0x{:02X} \n{}", op_addr, self),
        }
    }
}
