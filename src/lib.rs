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
            #include <cstring>
            #include <cstddef>
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

/// Look up a PDF set name and member ID by the LHAPDF ID code. The set name and member ID are
/// returned as a tuple in an `Option`. If lookup fails, `None` is returned.
#[must_use]
pub fn lookup_pdf(lhaid: i32) -> Option<(String, i32)> {
    cfg_if! {
        if #[cfg(feature = "docs-only")] {
            None
        } else {
            let pair = unsafe {
                cpp!([lhaid as "int"] -> *const c_void as "const void *" {
                    return new std::pair<std::string, int>(LHAPDF::lookupPDF(lhaid));
                })
            };

            let set_name = unsafe { CStr::from_ptr(
                cpp!([pair as "std::pair<std::string, int>*"] -> *const c_char as "const char *" {
                    return pair->first.c_str();
                }))
            }.to_str().unwrap().to_string();
            let member_id = unsafe {
                cpp!([pair as "std::pair<std::string, int>*"] -> i32 as "int" {
                    return pair->second;
                })
            };

            unsafe {
                cpp!([pair as "std::pair<std::string, int>*"] -> () as "void" {
                    return delete pair;
                })
            };

            if (set_name == String::new()) && (member_id == -1) {
                return None;
            }

            Some((set_name, member_id))
        }
    }
}

/// Convenient way to set the verbosity level.
pub fn set_verbosity(verbosity: i32) {
    cfg_if! {
        if #[cfg(feature = "docs-only")] {
        } else {
            unsafe {
                cpp!([verbosity as "int"] { LHAPDF::setVerbosity(verbosity); });
            }
        }
    }
}

/// Convenient way to get the current verbosity level.
#[must_use]
pub fn verbosity() -> i32 {
    cfg_if! {
        if #[cfg(feature = "docs-only")] {
            -1
        } else {
            unsafe {
                cpp!([] -> i32 as "int" { return LHAPDF::verbosity(); })
            }
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

unsafe impl Send for Pdf {}
unsafe impl Sync for Pdf {}

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

/// Structure for storage of uncertainty info calculated over a PDF error set.
pub struct PdfUncertainty {
    /// The central value.
    pub central: f64,
    /// The unsymmetric error in positive direction.
    pub errplus: f64,
    /// The unsymmetric error in negative direction.
    pub errminus: f64,
    /// The symmetric error.
    pub errsymm: f64,
    /// The scale factor needed to convert between the PDF set's default confidence level and the
    /// requested confidence level.
    pub scale: f64,
    /// Extra variable for separate PDF and parameter variation errors with combined sets.
    pub errplus_pdf: f64,
    /// Extra variable for separate PDF and parameter variation errors with combined sets.
    pub errminus_pdf: f64,
    /// Extra variable for separate PDF and parameter variation errors with combined sets.
    pub errsymm_pdf: f64,
    /// Extra variable for separate PDF and parameter variation errors with combined sets.
    pub err_par: f64,
}

/// Class for PDF set metadata and manipulation.
pub struct PdfSet {
    ptr: *mut c_void,
}

impl PdfSet {
    /// Constructor from a set name.
    #[must_use]
    pub fn new(setname: &str) -> Self {
        cfg_if! {
            if #[cfg(feature = "docs-only")] {
                Self { ptr: ptr::null_mut::<c_void>() }
            } else {
                let setname = CString::new(setname).unwrap();
                let setname_ptr = setname.as_ptr();

                Self {
                    ptr: unsafe {
                        cpp!([setname_ptr as "const char *"] -> *mut c_void as "LHAPDF::PDFSet *" {
                            return new LHAPDF::PDFSet(setname_ptr);
                        })
                    },
                }
            }
        }
    }

    /// Retrieve a metadata string by key name.
    #[must_use]
    pub fn entry(&self, key: &str) -> Option<String> {
        cfg_if! {
            if #[cfg(feature = "docs-only")] {
                None
            } else {
                let self_ptr = self.ptr;
                let key = CString::new(key).unwrap();
                let key_ptr = key.as_ptr();

                unsafe {
                    let has_key = cpp!([self_ptr as "LHAPDF::PDFSet*", key_ptr as "const char*"] -> bool as "bool" {
                        return self_ptr->has_key(key_ptr);
                    });

                    if has_key {
                        let size = cpp!([self_ptr as "LHAPDF::PDFSet*", key_ptr as "const char*"] -> usize as "std::size_t" {
                            auto const& value = self_ptr->get_entry(key_ptr);
                            return value.size();
                        });
                        let value_ptr = CString::new(vec![b' '; size]).unwrap().into_raw();
                        cpp!([self_ptr as "LHAPDF::PDFSet*", key_ptr as "const char*", value_ptr as "char*"] {
                            auto const& value = self_ptr->get_entry(key_ptr);
                            std::strncpy(value_ptr, value.c_str(), value.size() + 1);
                        });
                        Some(CString::from_raw(value_ptr).into_string().unwrap())
                    } else {
                        None
                    }
                }
            }
        }
    }

    /// Make all the PDFs in this set.
    pub fn mk_pdfs(&self) -> Vec<Pdf> {
        cfg_if! {
            if #[cfg(feature = "docs-only")] {
                vec![]
            } else {
                let self_ptr = self.ptr;
                let mut pdfs = vec![];

                unsafe {
                    let vec = cpp!([self_ptr as "LHAPDF::PDFSet*"] -> *mut c_void as "std::vector<LHAPDF::PDF*>*" {
                        return new std::vector<LHAPDF::PDF*>(self_ptr->mkPDFs());
                    });
                    let size = cpp!([vec as "std::vector<LHAPDF::PDF*>*"] -> usize as "std::size_t" {
                        return vec->size();
                    });

                    for index in 0..size {
                        pdfs.push(Pdf { ptr: cpp!([vec as "std::vector<LHAPDF::PDF*>*", index as "std::size_t"] -> *mut c_void as "LHAPDF::PDF*" {
                            return vec->at(index);
                        }) });
                    }

                    cpp!([vec as "std::vector<LHAPDF::PDF*>*"] -> () as "void" {
                        delete vec;
                    });
                }

                pdfs
            }
        }
    }

    /// Calculate central value and error from vector values with appropriate formulae for this
    /// set.
    ///
    /// Warning: The values vector corresponds to the members of this PDF set and must be ordered
    /// accordingly.
    ///
    /// In the Hessian approach, the central value is the best-fit "values[0]" and the uncertainty
    /// is given by either the symmetric or asymmetric formula using eigenvector PDF sets.
    ///
    /// If the PDF set is given in the form of replicas, by default, the central value is given by
    /// the mean and is not necessarily "values[0]" for quantities with a non-linear dependence on
    /// PDFs, while the uncertainty is given by the standard deviation.
    ///
    /// The argument `cl` is used to rescale uncertainties to a particular confidence level (in
    /// percent); a negative number will rescale to the default CL for this set. The default value
    /// in LHAPDF is `100*erf(1/sqrt(2))=68.268949213709`, corresponding to 1-sigma uncertainties.
    ///
    /// If the PDF set is given in the form of replicas, then the argument `alternative` equal to
    /// `true` (default in LHAPDF: `false`) will construct a confidence interval from the
    /// probability distribution of replicas, with the central value given by the median.
    ///
    /// For a combined set, a breakdown of the separate PDF and parameter variation uncertainties
    /// is available. The parameter variation uncertainties are computed from the last `2*n`
    /// members of the set, with `n` the number of parameters.
    #[must_use]
    pub fn uncertainty(&self, values: &[f64], cl: f64, alternative: bool) -> PdfUncertainty {
        let mut central = 0.0;
        let mut errplus = 0.0;
        let mut errminus = 0.0;
        let mut errsymm = 0.0;
        let mut scale = 0.0;
        let mut errplus_pdf = 0.0;
        let mut errminus_pdf = 0.0;
        let mut errsymm_pdf = 0.0;
        let mut err_par = 0.0;

        cfg_if! {
            if #[cfg(not(feature = "docs-only"))] {
                let self_ptr = self.ptr;
                let values_ptr = values.as_ptr();
                let values_len = values.len();

                unsafe {
                    cpp!([self_ptr as "LHAPDF::PDFSet *", values_ptr as "double *",
                          values_len as "std::size_t", cl as "double", alternative as "bool",
                          mut central as "double", mut errplus as "double",
                          mut errminus as "double", mut errsymm as "double", mut scale as "double",
                          mut errplus_pdf as "double", mut errminus_pdf as "double",
                          mut errsymm_pdf as "double", mut err_par as "double"] -> () as "void" {
                        LHAPDF::PDFUncertainty res;
                        std::vector<double> vec_values(values_ptr, values_ptr + values_len);
                        self_ptr->uncertainty(res, vec_values, cl, alternative);
                        central = res.central;
                        errplus = res.errplus;
                        errminus = res.errminus;
                        errsymm = res.errsymm;
                        scale = res.scale;
                        errplus_pdf = res.errplus_pdf;
                        errminus_pdf = res.errminus_pdf;
                        errsymm_pdf = res.errsymm_pdf;
                        err_par = res.err_par;
                    });
                }
            }
        }

        PdfUncertainty {
            central,
            errplus,
            errminus,
            errsymm,
            scale,
            errplus_pdf,
            errminus_pdf,
            errsymm_pdf,
            err_par,
        }
    }
}

impl Drop for PdfSet {
    fn drop(&mut self) {
        cfg_if! {
            if #[cfg(not(feature = "docs-only"))] {
                let self_ptr = self.ptr;

                unsafe {
                    cpp!([self_ptr as "LHAPDF::PDFSet *"] -> () as "void" {
                        delete self_ptr;
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_available_pdf_sets() {
        let pdf_sets = available_pdf_sets();

        assert!(pdf_sets
            .iter()
            .any(|pdf_set| pdf_set == "NNPDF31_nlo_as_0118_luxqed"));
    }

    #[test]
    fn check_lookup_pdf() {
        assert!(matches!(lookup_pdf(324900), Some((name, member))
            if (name == "NNPDF31_nlo_as_0118_luxqed") && (member == 0)));
        assert!(matches!(lookup_pdf(324901), Some((name, member))
            if (name == "NNPDF31_nlo_as_0118_luxqed") && (member == 1)));
        assert!(matches!(lookup_pdf(-1), None));
    }

    #[test]
    fn check_pdf() {
        let pdf_0 = Pdf::with_setname_and_member("NNPDF31_nlo_as_0118_luxqed", 0);
        let pdf_1 = Pdf::with_lhaid(324900);

        let value_0 = pdf_0.xfx_q2(2, 0.5, 90.0 * 90.0);
        let value_1 = pdf_1.xfx_q2(2, 0.5, 90.0 * 90.0);

        assert_ne!(value_0, 0.0);
        assert_eq!(value_0, value_1);

        let value_0 = pdf_0.alphas_q2(90.0 * 90.0);
        let value_1 = pdf_1.alphas_q2(90.0 * 90.0);

        assert_ne!(value_0, 0.0);
        assert_eq!(value_0, value_1);
    }

    #[test]
    fn check_pdf_set() {
        let pdf_set = PdfSet::new("NNPDF31_nlo_as_0118_luxqed");

        assert!(matches!(pdf_set.entry("Particle"), Some(value) if value == "2212"));
        assert!(matches!(pdf_set.entry("Flavors"), Some(value)
            if value == "[-5, -4, -3, -2, -1, 21, 1, 2, 3, 4, 5, 22]"));
    }
}
