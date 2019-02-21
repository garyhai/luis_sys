/*! */

use crate::speech_api::{
    property_bag_free_string, property_bag_get_string, property_bag_is_valid,
    property_bag_release, property_bag_set_string, PropertyId,
};
use crate::{Handle, Result, SmartHandle, SpxHandle};
use std::{
    ffi::{CStr, CString},
    os::raw::c_int,
    ptr::null,
};

SmartHandle!(Properties, property_bag_release, property_bag_is_valid);

impl Properties {
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
