/*! */

use crate::speech_api::{
    property_bag_free_string, property_bag_get_string, property_bag_release,
    property_bag_set_string, PropertyId, SPXPROPERTYBAGHANDLE,
};
use crate::{Result, SpxHandle, Handle};
use std::{
    ffi::{CStr, CString},
    os::raw::c_int,
    ptr::null,
};

pub(crate) struct Properties {
    handle: SPXPROPERTYBAGHANDLE,
}

impl Properties {
    pub fn new(handle: SPXPROPERTYBAGHANDLE) -> Self {
        Properties { handle }
    }

    pub fn get_by_id(&self, id: PropertyId) -> Result<String> {
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

    pub fn get_by_name(&self, name: &str) -> Result<String> {
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

    pub fn put_by_id(&self, id: PropertyId, value: &str) -> Result<()> {
        let value = CString::new(value)?;
        unsafe {
            property_bag_set_string(
                self.handle,
                id as c_int,
                null(),
                value.as_ptr(),
            );
            Ok(())
        }
    }

    pub fn put_by_name(&self, name: &str, value: &str) -> Result<()> {
        let name = CString::new(name)?;
        let value = CString::new(value)?;
        unsafe {
            property_bag_set_string(
                self.handle,
                -1,
                name.as_ptr(),
                value.as_ptr(),
            );
            Ok(())
        }
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        unsafe { property_bag_release(self.handle) };
    }
}

impl SpxHandle for Properties {
    fn handle(&self) -> Handle {
        self.handle as Handle
    }
}
