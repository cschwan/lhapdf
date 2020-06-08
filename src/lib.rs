#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(missing_docs)]

//! (Unofficial) Rust wrapper for the [LHAPDF](https://lhapdf.hepforge.org) C++ library.

#[macro_use]
extern crate cfg_if;

use std::ffi::c_void;

cfg_if! {
    if #[cfg(not(feature = "docs-only"))] {
        #[macro_use]
        extern crate cpp;

        use std::ffi::{CStr, CString};
        use std::os::raw::c_char;

        cpp! {{
            #include <LHAPDF/LHAPDF.h>
        }}
    } else {
        use std::ptr;
    }
}

/// Get the names of all available PDF sets in the search path.
#[must_use]
pub fn available_pdf_sets() -> Vec<String> {
    cfg_if! {
        if #[cfg(feature = "docs-only")] {
            vec![]
        } else {
            let pdfs = unsafe {
                cpp!([] -> usize as "size_t" {
                    return static_cast<unsigned> (LHAPDF::availablePDFSets().size());
                })
            };

            let mut pdf_sets: Vec<String> = Vec::with_capacity(pdfs);

            for i in 0..pdfs {
                let cstr_ptr = unsafe {
                    cpp!([i as "size_t"] -> *const c_char as "const char *" {
                        return LHAPDF::availablePDFSets().at(i).c_str();
                    })
                };
                pdf_sets.push(
                    unsafe { CStr::from_ptr(cstr_ptr) }
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }

            pdf_sets
        }
    }
}

/// Wrapper to an LHAPDF object of the type `LHAPDF::PDF`.
pub struct Pdf {
    ptr: *mut c_void,
}

impl Pdf {
    /// Constructor. Create a new PDF with the given PDF `setname` and `member` ID.
    #[must_use]
    pub fn with_setname_and_member(setname: &str, member: i32) -> Self {
        cfg_if! {
            if #[cfg(feature = "docs-only")] {
                Self { ptr: ptr::null_mut::<c_void>() }
            } else {
                let setname = CString::new(setname).unwrap();
                let setname_ptr = setname.as_ptr();

                Self {
                    ptr: unsafe {
                        cpp!([setname_ptr as "const char *",
                              member as "int"] -> *mut c_void as "LHAPDF::PDF*" {
                            return LHAPDF::mkPDF(setname_ptr, member);
                        })
                    },
                }
            }
        }
    }

    /// Constructor. Create a new PDF with the given `lhaid` ID code.
    #[must_use]
    pub fn with_lhaid(lhaid: i32) -> Self {
        cfg_if! {
            if #[cfg(feature = "docs-only")] {
                Self { ptr: ptr::null_mut::<c_void>() }
            } else {
                Self {
                    ptr: unsafe {
                        cpp!([lhaid as "int"] -> *mut c_void as "LHAPDF::PDF*" {
                            return LHAPDF::mkPDF(lhaid);
                        })
                    },
                }
            }
        }
    }

    /// Get the PDF `x * f(x)` value at `x` and `q2` for the given PDG ID.
    #[must_use]
    pub fn xfx_q2(&self, id: i32, x: f64, q2: f64) -> f64 {
        cfg_if! {
            if #[cfg(feature = "docs-only")] {
                0.0
            } else {
                let self_ptr = self.ptr;

                unsafe {
                    cpp!([self_ptr as "LHAPDF::PDF *",
                                   id as "int",
                                   x as "double",
                                   q2 as "double"] -> f64 as "double" {
                        return self_ptr->xfxQ2(id, x, q2);
                    })
                }
            }
        }
    }

    /// Value of of the strong coupling at `q2` used by this PDF.
    #[must_use]
    pub fn alphas_q2(&self, q2: f64) -> f64 {
        cfg_if! {
            if #[cfg(feature = "docs-only")] {
                0.0
            } else {
                let self_ptr = self.ptr;

                unsafe {
                    cpp!([self_ptr as "LHAPDF::PDF *", q2 as "double"] -> f64 as "double" {
                        return self_ptr->alphasQ2(q2);
                    })
                }
            }
        }
    }
}

impl Drop for Pdf {
    fn drop(&mut self) {
        cfg_if! {
            if #[cfg(not(feature = "docs-only"))] {
                let self_ptr = self.ptr;

                unsafe {
                    cpp!([self_ptr as "LHAPDF::PDF *"] -> () as "void" {
                        delete self_ptr;
                    });
                }
            }
        }
    }
}