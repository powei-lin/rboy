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

    pub fn set_value(&mut self, reg: &RegisterValue) {
        set_register_value!(self, reg,
            AF => (register_f, register_a),
            BC => (register_c, register_b),
            DE => (register_e, register_d)
        );

        set_register_value!(self, reg,
            A => register_a,
            B => register_b,
            C => register_c,
            D => register_d,
            E => register_e,
            F => register_f,
            H => register_h,
            L => register_l,
            SP => register_sp,
            PC => register_pc
        );
    }

    pub fn tick(&mut self, mem: &mut memory::Memory) -> u8 {
        let op_addr: u8 = mem.get(self.register_pc);
        self.register_pc += 1;
        if op_addr == 0xcb {};
        match op_addr {
            0xcb => {
                let cb_op_addr: u8 = mem.get(self.register_pc);
                self.register_pc += 1;
                match cb_op_addr {
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
        0u8
    }
}
