/*! */

use crate::speech_api::{
    property_bag_free_string, property_bag_get_string, property_bag_is_valid,
    property_bag_release, property_bag_set_string, PropertyId,
    SPXPROPERTYBAGHANDLE,
};
use crate::{error::Unimplemented, hr, Result, SmartHandle};
use std::{
    ffi::{CStr, CString},
    fmt,
    os::raw::c_int,
    ptr::null,
};

pub trait PropertyBag {
    fn get_by_id(&self, _id: PropertyId) -> Result<String> {
        Err(Unimplemented)
    }

    fn get_by_name(&self, _name: &str) -> Result<String> {
        Err(Unimplemented)
    }

    fn put_by_id(&self, _id: PropertyId, _value: &str) -> Result<()> {
        Err(Unimplemented)
    }

    fn put_by_name(&self, _name: &str, _value: &str) -> Result<()> {
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

    fn put_by_id(&self, id: PropertyId, value: &str) -> Result<()> {
        let value = CString::new(value)?;
        hr! {
            property_bag_set_string(
                self.handle,
                id as c_int,
                null(),value.as_ptr()
        )}
    }

    fn put_by_name(&self, name: &str, value: &str) -> Result<()> {
        let name = CString::new(name)?;
        let value = CString::new(value)?;
        hr! {
            property_bag_set_string(self.handle, -1, name.as_ptr(), value.as_ptr())
        }
    }
}

impl fmt::Debug for Properties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Properties {{handle: {}, ...}}.", self.handle as usize)
    }
}
