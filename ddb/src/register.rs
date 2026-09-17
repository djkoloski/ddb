use core::ffi::c_uint;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterKind {
    /// A general-purpose register
    Gp,
    /// A subregister of a general-purpose register
    SubGp,
    /// A floating-point register
    Fp,
    /// A debug register
    Debug,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct f80 {
    value: [c_uint; 4],
}

impl f80 {
    pub const fn from_bytes(bytes: [u8; 10]) -> Self {
        Self {
            value: [
                c_uint::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                c_uint::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                c_uint::from_ne_bytes([bytes[8], bytes[9], 0, 0]),
                0,
            ],
        }
    }

    pub const fn to_ne_bytes(&self) -> [u8; 10] {
        let b = [
            self.value[0].to_ne_bytes(),
            self.value[1].to_ne_bytes(),
            self.value[2].to_ne_bytes(),
        ];
        [
            b[0][0], b[0][1], b[0][2], b[0][3], b[1][0], b[1][1], b[1][2],
            b[1][3], b[2][0], b[2][1],
        ]
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct u8x8 {
    value: [c_uint; 2],
}

impl u8x8 {
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        Self {
            value: [
                c_uint::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                c_uint::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            ],
        }
    }

    pub const fn to_ne_bytes(&self) -> [u8; 8] {
        let b = [self.value[0].to_ne_bytes(), self.value[1].to_ne_bytes()];
        [
            b[0][0], b[0][1], b[0][2], b[0][3], b[1][0], b[1][1], b[1][2],
            b[1][3],
        ]
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct u8x16 {
    value: [c_uint; 4],
}

impl u8x16 {
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            value: [
                c_uint::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                c_uint::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                c_uint::from_ne_bytes([
                    bytes[8], bytes[9], bytes[10], bytes[11],
                ]),
                c_uint::from_ne_bytes([
                    bytes[12], bytes[13], bytes[14], bytes[15],
                ]),
            ],
        }
    }

    pub const fn to_ne_bytes(&self) -> [u8; 16] {
        let b = [
            self.value[0].to_ne_bytes(),
            self.value[1].to_ne_bytes(),
            self.value[2].to_ne_bytes(),
            self.value[3].to_ne_bytes(),
        ];
        [
            b[0][0], b[0][1], b[0][2], b[0][3], b[1][0], b[1][1], b[1][2],
            b[1][3], b[2][0], b[2][1], b[2][2], b[2][3], b[3][0], b[3][1],
            b[3][2], b[3][3],
        ]
    }
}

macro_rules! define_register_types {
    (
        $(#[$metas:meta])*
        pub enum $register_value:ident: $register_type:ident {
            $($variant:ident($ty:ty)),* $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $register_type {
            $($variant,)*
        }

        impl ::core::fmt::Display for $register_type {
            fn fmt(
                &self,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                match self {
                    $(Self::$variant =>write!(
                        f,
                        "{}",
                        ::core::stringify!($variant),
                    ),)*
                }
            }
        }

        impl $register_type {
            /// Reads a value of this register type from the given pointer.
            ///
            /// # Safety
            ///
            /// The provided pointer must be valid for a read of the value of
            /// this register type.
            pub unsafe fn read(self, ptr: *const ()) -> $register_value {
                match self {
                    $(
                        Self::$variant => $register_value::$variant(
                            unsafe { ptr.cast::<$ty>().read() }
                        ),
                    )*
                }
            }
        }

        $(#[$metas])*
        pub enum $register_value {
            $($variant($ty),)*
        }

        impl $register_value {
            pub fn ty(&self) -> $register_type {
                match self {
                    $(Self::$variant(_) => $register_type::$variant,)*
                }
            }

            /// Writes this register value to the given pointer.
            ///
            /// # Safety
            ///
            /// The provided pointer must be valid for a write of the value of
            /// this register value.
            pub unsafe fn write(self, ptr: *mut ()) {
                match self {
                    $(
                        Self::$variant(value) => unsafe {
                            ptr.cast::<$ty>().write(value)
                        },
                    )*
                }
            }
        }

        #[derive(Debug)]
        pub struct RegisterTypeError {
            pub expected: $register_type,
            pub actual: $register_type,
        }

        impl ::core::fmt::Display for RegisterTypeError {
            fn fmt(
                &self,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                write!(
                    f,
                    "expected register value of type {}, but found value of \
                    type {}",
                    self.expected,
                    self.actual,
                )
            }
        }

        $(
            impl ::core::convert::From<$ty> for $register_value {
                fn from(value: $ty) -> Self {
                    Self::$variant(value)
                }
            }

            impl ::core::convert::TryFrom<$register_value> for $ty {
                type Error = RegisterTypeError;

                fn try_from(
                    value: $register_value,
                ) -> Result<Self, Self::Error> {
                    if let $register_value::$variant(x) = value {
                        Ok(x)
                    } else {
                        Err(RegisterTypeError {
                            expected: $register_type::$variant,
                            actual: value.ty(),
                        })
                    }
                }
            }
        )*
    }
}

define_register_types! {
    #[derive(Clone, Debug)]
    pub enum RegisterValue: RegisterType {
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        F32(f32),
        F64(f64),
        F80(f80),
        U8x8(u8x8),
        U8x16(u8x16),
    }
}

macro_rules! impl_register_conversion_via_cast {
    ($($from_ty:ty as $to_ty:ty),* $(,)?) => {
        $(
            impl From<$to_ty> for RegisterValue {
                fn from(value: $to_ty) -> RegisterValue {
                    RegisterValue::from(value as $from_ty)
                }
            }

            impl TryFrom<RegisterValue> for $to_ty {
                type Error = RegisterTypeError;

                fn try_from(value: RegisterValue) -> Result<Self, Self::Error> {
                    Ok(<$from_ty>::try_from(value)? as Self)
                }
            }
        )*
    }
}

impl_register_conversion_via_cast! {
    u8 as i8,
    u16 as i16,
    u32 as i32,
    u64 as i64,
}

pub struct RegisterInfo {
    pub name: &'static str,
    pub dwarf_id: i32,
    pub kind: RegisterKind,
    pub ty: RegisterType,
    pub offset: usize,
}

macro_rules! define_registers {
    (
        $(#[$metas:meta])*
        pub enum $register:ident { .. }

        gpr_64 { $($gpr_64_name:ident = $gpr_64_dwarf_id:expr;)* }
        gpr_32 { $($gpr_32_name:ident = $gpr_32_super:ident;)* }
        gpr_16 { $($gpr_16_name:ident = $gpr_16_super:ident;)* }
        gpr_8h { $($gpr_8h_name:ident = $gpr_8h_super:ident;)* }
        gpr_8l { $($gpr_8l_name:ident = $gpr_8l_super:ident;)* }
        fpr { $($fpr_name:ident = $fpr_dwarf_id:expr, $fpr_user_name:ident;)* }
        fpr_st { $($fpr_st_name:ident = $fpr_st_number:expr;)* }
        fpr_mm { $($fpr_mm_name:ident = $fpr_mm_number:expr;)* }
        fpr_xmm { $($fpr_xmm_name:ident = $fpr_xmm_number:expr;)* }
        dr { $($dr_name:ident = $dr_number:expr;)* }
    ) => {
        $(#[$metas])*
        #[allow(non_camel_case_types)]
        pub enum $register {
            $($gpr_64_name,)*
            $($gpr_32_name,)*
            $($gpr_16_name,)*
            $($gpr_8h_name,)*
            $($gpr_8l_name,)*
            $($fpr_name,)*
            $($fpr_st_name,)*
            $($fpr_mm_name,)*
            $($fpr_xmm_name,)*
            $($dr_name,)*
        }

        impl $register {
            pub const DEBUG_REGS_COUNT: usize = 0
                $(+ { let _ = Self::$dr_name; 1 })*;
            pub const DEBUG_REGS: [Self; Self::DEBUG_REGS_COUNT] = [
                $(Self::$dr_name,)*
            ];

            pub const fn from_dwarf_id(dwarf_id: i32) -> Option<Self> {
                #[allow(unreachable_patterns)]
                Some(match dwarf_id {
                    -1 => return None,
                    $($gpr_64_dwarf_id => Self::$gpr_64_name,)*
                    $($fpr_dwarf_id => Self::$fpr_name,)*
                    $(x if x == 33 + $fpr_st_number => Self::$fpr_st_name,)*
                    $(x if x == 41 + $fpr_mm_number => Self::$fpr_mm_name,)*
                    $(x if x == 17 + $fpr_xmm_number => Self::$fpr_xmm_name,)*
                    _ => return None,
                })
            }

            pub fn from_name(name: &str) -> Option<Self> {
                Some(match name {
                    $(::core::stringify!($gpr_64_name) => Self::$gpr_64_name,)*
                    $(::core::stringify!($gpr_32_name) => Self::$gpr_32_name,)*
                    $(::core::stringify!($gpr_16_name) => Self::$gpr_16_name,)*
                    $(::core::stringify!($gpr_8h_name) => Self::$gpr_8h_name,)*
                    $(::core::stringify!($gpr_8l_name) => Self::$gpr_8l_name,)*
                    $(::core::stringify!($fpr_name) => Self::$fpr_name,)*
                    $(::core::stringify!($fpr_st_name) => Self::$fpr_st_name,)*
                    $(::core::stringify!($fpr_mm_name) => Self::$fpr_mm_name,)*
                    $(
                        ::core::stringify!($fpr_xmm_name)
                            => Self::$fpr_xmm_name,
                    )*
                    $(::core::stringify!($dr_name) => Self::$dr_name,)*
                    _ => return None,
                })
            }

            pub const fn info(self) -> &'static RegisterInfo {
                const REGISTER_INFOS: &[RegisterInfo] = &[
                    $(gpr_info!($gpr_64_name, $gpr_64_dwarf_id),)*
                    $(
                        gpr_sub_info!(
                            $gpr_32_name,
                            $gpr_32_super,
                            RegisterType::U32,
                            0,
                        ),
                    )*
                    $(
                        gpr_sub_info!(
                            $gpr_16_name,
                            $gpr_16_super,
                            RegisterType::U16,
                            0,
                        ),
                    )*
                    $(
                        gpr_sub_info!(
                            $gpr_8h_name,
                            $gpr_8h_super,
                            RegisterType::U8,
                            1,
                        ),
                    )*
                    $(
                        gpr_sub_info!(
                            $gpr_8l_name,
                            $gpr_8l_super,
                            RegisterType::U8,
                            0,
                        ),
                    )*
                    $(fpr_info!($fpr_name, $fpr_dwarf_id, $fpr_user_name),)*
                    $(fpr_st_info!($fpr_st_name, $fpr_st_number),)*
                    $(fpr_mm_info!($fpr_mm_name, $fpr_mm_number),)*
                    $(fpr_xmm_info!($fpr_xmm_name, $fpr_xmm_number),)*
                    $(dr_info!($dr_name, $dr_number),)*
                ];

                &REGISTER_INFOS[self as usize]
            }
        }
    };
}

macro_rules! gpr_info {
    ($name:ident, $dwarf_id:expr $(,)?) => {
        RegisterInfo {
            name: ::core::stringify!($name),
            dwarf_id: $dwarf_id,
            kind: RegisterKind::Gp,
            ty: RegisterType::U64,
            offset: ::core::mem::offset_of!(::libc::user, regs)
                + ::core::mem::offset_of!(::libc::user_regs_struct, $name),
        }
    };
}

macro_rules! gpr_sub_info {
    ($name:ident, $super:ident, $ty:expr, $offset:expr $(,)?) => {
        RegisterInfo {
            name: ::core::stringify!($name),
            dwarf_id: -1,
            kind: RegisterKind::SubGp,
            ty: $ty,
            offset: ::core::mem::offset_of!(::libc::user, regs)
                + ::core::mem::offset_of!(::libc::user_regs_struct, $super)
                + $offset,
        }
    };
}

const fn size_of_pointee<T>(_: *const T) -> usize {
    size_of::<T>()
}

macro_rules! size_of {
    ($struct:path, $field:ident) => {
        const {
            let value = ::core::mem::MaybeUninit::<$struct>::uninit();
            let ptr = value.as_ptr();
            let field_ptr = unsafe { ::core::ptr::addr_of!((*ptr).$field) };
            size_of_pointee(field_ptr)
        }
    };
}

macro_rules! fpr_info {
    ($name:ident, $dwarf_id:expr, $user_name:ident $(,)?) => {
        RegisterInfo {
            name: ::core::stringify!($name),
            dwarf_id: $dwarf_id,
            kind: RegisterKind::Fp,
            ty: const {
                match size_of!(::libc::user_fpregs_struct, $user_name) {
                    1 => RegisterType::U8,
                    2 => RegisterType::U16,
                    4 => RegisterType::U32,
                    8 => RegisterType::U64,
                    _ => panic!("invalid size for register type"),
                }
            },
            offset: ::core::mem::offset_of!(::libc::user, i387)
                + ::core::mem::offset_of!(
                    ::libc::user_fpregs_struct,
                    $user_name
                ),
        }
    };
}

macro_rules! fpr_st_info {
    ($name:ident, $number:expr $(,)?) => {
        RegisterInfo {
            name: ::core::stringify!($name),
            dwarf_id: 33 + $number,
            kind: RegisterKind::Fp,
            ty: RegisterType::F80,
            offset: ::core::mem::offset_of!(::libc::user, i387)
                + ::core::mem::offset_of!(::libc::user_fpregs_struct, st_space)
                + 16 * $number,
        }
    };
}

macro_rules! fpr_mm_info {
    ($name:ident, $number:expr $(,)?) => {
        RegisterInfo {
            name: ::core::stringify!($name),
            dwarf_id: 41 + $number,
            kind: RegisterKind::Fp,
            ty: RegisterType::U8x8,
            offset: ::core::mem::offset_of!(::libc::user, i387)
                + ::core::mem::offset_of!(::libc::user_fpregs_struct, st_space)
                + 16 * $number,
        }
    };
}

macro_rules! fpr_xmm_info {
    ($name:ident, $number:expr $(,)?) => {
        RegisterInfo {
            name: ::core::stringify!($name),
            dwarf_id: 17 + $number,
            kind: RegisterKind::Fp,
            ty: RegisterType::U8x16,
            offset: ::core::mem::offset_of!(::libc::user, i387)
                + ::core::mem::offset_of!(
                    ::libc::user_fpregs_struct,
                    xmm_space
                )
                + 16 * $number,
        }
    };
}

macro_rules! dr_info {
    ($name:ident, $number:expr) => {
        RegisterInfo {
            name: ::core::stringify!($name),
            dwarf_id: -1,
            kind: RegisterKind::Debug,
            ty: RegisterType::U64,
            offset: ::core::mem::offset_of!(::libc::user, u_debugreg)
                + 8 * $number,
        }
    };
}

define_registers! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Register { .. }

    gpr_64 {
        rax = 0;
        rdx = 1;
        rcx = 2;
        rbx = 3;
        rsi = 4;
        rdi = 5;
        rbp = 6;
        rsp = 7;
        r8 = 8;
        r9 = 9;
        r10 = 10;
        r11 = 11;
        r12 = 12;
        r13 = 13;
        r14 = 14;
        r15 = 15;
        rip = 16;
        eflags = 49;
        cs = 51;
        fs = 54;
        gs = 55;
        ss = 52;
        ds = 53;
        es = 50;

        orig_rax = -1;
    }

    gpr_32 {
        eax = rax;
        edx = rdx;
        ecx = rcx;
        ebx = rbx;
        esi = rsi;
        edi = rdi;
        ebp = rbp;
        esp = rsp;
        r8d = r8;
        r9d = r9;
        r10d = r10;
        r11d = r11;
        r12d = r12;
        r13d = r13;
        r14d = r14;
        r15d = r15;
    }

    gpr_16 {
        ax = rax;
        dx = rdx;
        cx = rcx;
        bx = rbx;
        si = rsi;
        di = rdi;
        bp = rbp;
        sp = rsp;
        r8w = r8;
        r9w = r9;
        r10w = r10;
        r11w = r11;
        r12w = r12;
        r13w = r13;
        r14w = r14;
        r15w = r15;
    }

    gpr_8h {
        ah = rax;
        dh = rdx;
        ch = rcx;
        bh = rbx;
    }

    gpr_8l {
        al = rax;
        dl = rdx;
        cl = rcx;
        bl = rbx;
        sil = rsi;
        dil = rdi;
        bpl = rbp;
        spl = rsp;
        r8b = r8;
        r9b = r9;
        r10b = r10;
        r11b = r11;
        r12b = r12;
        r13b = r13;
        r14b = r14;
        r15b = r15;
    }

    fpr {
        fcw = 65, cwd;
        fsw = 66, swd;
        ftw = -1, ftw;
        fop = -1, fop;
        frip = -1, rip;
        frdp = -1, rdp;
        mxcsr = 64, mxcsr;
        mxcsrmask = -1, mxcr_mask;
    }

    fpr_st {
        st0 = 0;
        st1 = 1;
        st2 = 2;
        st3 = 3;
        st4 = 4;
        st5 = 5;
        st6 = 6;
        st7 = 7;
    }

    fpr_mm {
        mm0 = 0;
        mm1 = 1;
        mm2 = 2;
        mm3 = 3;
        mm4 = 4;
        mm5 = 5;
        mm6 = 6;
        mm7 = 7;
    }

    fpr_xmm {
        xmm0 = 0;
        xmm1 = 1;
        xmm2 = 2;
        xmm3 = 3;
        xmm4 = 4;
        xmm5 = 5;
        xmm6 = 6;
        xmm7 = 7;
        xmm8 = 8;
        xmm9 = 9;
        xmm10 = 10;
        xmm11 = 11;
        xmm12 = 12;
        xmm13 = 13;
        xmm14 = 14;
        xmm15 = 15;
    }

    dr {
        dr0 = 0;
        dr1 = 1;
        dr2 = 2;
        dr3 = 3;
        dr4 = 4;
        dr5 = 5;
        dr6 = 6;
        dr7 = 7;
    }
}
