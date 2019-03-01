// Quick and dirty "DRY"

#[macro_export]
macro_rules! DeriveHandle {
    ( $name:ident, $t:ty, $release:ident $(, $check:ident)? ) => (
        impl crate::Handle<$t> for $name {
            fn handle(&self) -> $t {
                self.handle
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    $( if !$check(self.handle) { return; } )?
                    $release(self.handle);
                }
                log::trace!("{}({}) is released",
                    stringify!($name),
                    self.handle as usize);
                self.handle = crate::INVALID_HANDLE;
            }
        }

        unsafe impl Send for $name {}
    )
}

#[macro_export]
macro_rules! SmartHandle {
    ( $name:ident, $t:ty, $release:ident, $check:ident ) => {
        crate::DeriveHandle!($name, $t, $release, $check);
        pub struct $name {
            handle: $t,
        }

        impl $name {
            pub fn new(handle: $t) -> Self {
                $name { handle }
            }

            #[allow(dead_code)]
            pub fn is_valid(&self) -> bool {
                unsafe { $check(self.handle) }
            }

            #[allow(dead_code)]
            pub fn release(&mut self) {
                if self.is_valid() {
                    unsafe { $release(self.handle) };
                }
                self.handle = crate::INVALID_HANDLE as $t;
            }
        }

        impl std::ops::Deref for $name {
            type Target = $t;

            fn deref(&self) -> &$t {
                &self.handle
            }
        }

        impl From<$t> for $name {
            fn from(handle: $t) -> Self {
                Self::new(handle)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(crate::INVALID_HANDLE as $t)
            }
        }
    };
}

#[macro_export]
macro_rules! hr {
    ($ffi:expr) => {
        crate::ffi_result(unsafe { $ffi })
    };
}

#[macro_export]
macro_rules! ffi_get_string {
    ($f:ident, $h:expr $(, $sz:expr)?) => ({
        let _max_len = 1024;
        $(
            let _max_len = $sz;
        )?
        let s = String::with_capacity(_max_len + 1);
        let buf = r#try!(::std::ffi::CString::new(s));
        let buf_ptr = buf.into_raw();
        unsafe {
            r#try!(hr($f($h, buf_ptr, _max_len as u32)));
            let output = ::std::ffi::CString::from_raw(buf_ptr);
            r#try!(output.into_string())
        }
    })
}
