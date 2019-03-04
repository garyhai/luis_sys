/// Struct and trait to retrieve or set a property value from a property collection.
///

use crate::speech_api::*;
use crate::{error::Unimplemented, hr, Result, SmartHandle};
use std::{
    ffi::{CStr, CString},
    fmt,
    os::raw::c_int,
    ptr::null,
};

/// Behavior of the property bag.
pub trait PropertyBag {
    fn get_by_id(&self, _id: PropertyId) -> Result<String> {
        Err(Unimplemented)
    }

    fn get_by_name(&self, _name: &str) -> Result<String> {
        Err(Unimplemented)
    }

    fn put_by_id<T: ToString>(&self, _id: PropertyId, _value: T) -> Result<()> {
        Err(Unimplemented)
    }

    fn put_by_name<T: ToString>(&self, _name: &str, _value: T) -> Result<()> {
        Err(Unimplemented)
    }
}

SmartHandle!(
    Properties,
    SPXPROPERTYBAGHANDLE,
    property_bag_release,
    property_bag_is_valid
);

impl PropertyBag for Properties {
    fn get_by_id(&self, id: PropertyId) -> Result<String> {
        let blank = CString::new("")?;
        unsafe {
            let v = property_bag_get_string(
                self.handle,
                id as c_int,
                null(),
                blank.as_ptr(),
            );
            let vs = CStr::from_ptr(v).to_owned().into_string()?;
            property_bag_free_string(v);
            Ok(vs)
        }
    }

    fn get_by_name(&self, name: &str) -> Result<String> {
        let name = CString::new(name)?;
        let blank = CString::new("").unwrap();
        unsafe {
            let v = property_bag_get_string(
                self.handle,
                -1,
                name.as_ptr(),
                blank.as_ptr(),
            );
            let vs = CStr::from_ptr(v).to_owned().into_string()?;
            property_bag_free_string(v);
            Ok(vs)
        }
    }

    fn put_by_id<T: ToString>(&self, id: PropertyId, value: T) -> Result<()> {
        let value = CString::new(value.to_string())?;
        hr! {
            property_bag_set_string(
                self.handle,
                id as c_int,
                null(),value.as_ptr()
        )}
    }

    fn put_by_name<T: ToString>(&self, name: &str, value: T) -> Result<()> {
        let name = CString::new(name)?;
        let value = CString::new(value.to_string())?;
        hr! {
            property_bag_set_string(self.handle, -1, name.as_ptr(), value.as_ptr())
        }
    }
}

/// Do nothing, just derive a Debug trait.
impl fmt::Debug for Properties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Properties {{handle: {}, ...}}.", self.handle as usize)
    }
}
