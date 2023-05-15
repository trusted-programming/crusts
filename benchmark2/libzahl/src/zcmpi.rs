use ::libc;
extern "C" {
    fn zseti(_: *mut C2RustUnnamed, _: libc::c_longlong);
    fn zcmp(_: *mut C2RustUnnamed, _: *mut C2RustUnnamed) -> libc::c_int;
    static mut libzahl_tmp_cmp: z_t;
}
pub type size_t = libc::c_ulong;
pub type __uint32_t = libc::c_uint;
pub type uint32_t = __uint32_t;
pub type zahl_char_t = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub sign: libc::c_int,
    pub used: size_t,
    pub alloced: size_t,
    pub chars: *mut zahl_char_t,
}
pub type z_t = [C2RustUnnamed; 1];
#[inline]
unsafe extern "C" fn zzero(mut a: *mut C2RustUnnamed) -> libc::c_int {
    return ((*a).sign == 0) as libc::c_int;
}
#[inline]
unsafe extern "C" fn zsignum(mut a: *mut C2RustUnnamed) -> libc::c_int {
    return (*a).sign;
}
#[no_mangle]
pub unsafe extern "C" fn zcmpi(
    mut a: *mut C2RustUnnamed,
    mut b: libc::c_longlong,
) -> libc::c_int {
    if b == 0 {
        return zsignum(a);
    }
    if zzero(a) != 0 {
        return if b > 0 as libc::c_int as libc::c_longlong {
            -(1 as libc::c_int)
        } else {
            (b < 0 as libc::c_int as libc::c_longlong) as libc::c_int
        };
    }
    zseti(libzahl_tmp_cmp.as_mut_ptr(), b);
    return zcmp(a, libzahl_tmp_cmp.as_mut_ptr());
}
