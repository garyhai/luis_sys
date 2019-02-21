// Quick and dirty "DRY"
#[macro_export]
macro_rules! DeriveSpxHandle {
    ( $name:ident, $release:ident $(, $check:ident)? ) => (
        impl SpxHandle for $name {
            fn handle(&self) -> Handle {
                self.handle as Handle
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
            }
        }
    )
}

#[macro_export]
macro_rules! create_prop {
    ($prop_get:ident, $prop_put:ident, $id:expr) => (
        pub fn $prop_get(&self) -> Result<String> {
            self.props.get_by_id($id)
        }

        pub fn $prop_put(&self, v: &str) -> Result {
            self.props.put_by_id($id, v)
        }
    )
}

#[macro_export]
macro_rules! SmartHandle {
    ( $name:ident, $release:ident, $check:ident ) => (
        crate::DeriveSpxHandle!($name, $release, $check);
        pub struct $name {
            handle: Handle,
        }

        impl $name {
            pub fn new(handle: Handle) -> Self {
                $name { handle }
            }
        }

        impl From<Handle> for $name {
            fn from(handle: Handle) -> Self {
                Self::new(handle)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(::std::ptr::null_mut())
            }
        }
    )
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
