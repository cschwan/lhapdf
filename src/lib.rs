#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::use_self)] // TODO: remove this line

//! (Unofficial) Rust wrapper for the [LHAPDF](https://lhapdf.hepforge.org) C++ library.

use cxx::{let_cxx_string, Exception, UniquePtr};
use std::convert::TryFrom;
use std::fmt::{self, Formatter};
use std::result;
use thiserror::Error;

#[cxx::bridge(namespace = "LHAPDF")]
mod ffi {
    /// Structure for storage of uncertainty info calculated over a PDF error set.
    struct PdfUncertainty {
        /// The central value.
        pub central: f64,
        /// The unsymmetric error in positive direction.
        pub errplus: f64,
        /// The unsymmetric error in negative direction.
        pub errminus: f64,
        /// The symmetric error.
        pub errsymm: f64,
        /// The scale factor needed to convert between the PDF set's default confidence level and
        /// the requested confidence level.
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

    unsafe extern "C++" {
        include!("LHAPDF/LHAPDF.h");

        fn availablePDFSets() -> &'static CxxVector<CxxString>;
        fn setVerbosity(verbosity: i32);
        fn verbosity() -> i32;

        type PDF;

        fn alphasQ2(self: &PDF, q2: f64) -> Result<f64>;
        fn xfxQ2(self: &PDF, id: i32, x: f64, q2: f64) -> Result<f64>;
        fn lhapdfID(self: &PDF) -> i32;

        type PDFSet;

        fn has_key(self: &PDFSet, key: &CxxString) -> bool;
        fn get_entry(self: &PDFSet, key: &CxxString) -> &'static CxxString;
        fn size(self: &PDFSet) -> usize;
        fn lhapdfID(self: &PDFSet) -> i32;

        include!("lhapdf/include/wrappers.hpp");

        fn pdf_with_setname_and_member(setname: &CxxString, member: i32) -> Result<UniquePtr<PDF>>;
        fn pdf_with_set_and_member(set: &PDFSet, member: i32) -> Result<UniquePtr<PDF>>;
        fn pdf_with_lhaid(lhaid: i32) -> Result<UniquePtr<PDF>>;
        fn pdfset_new(setname: &CxxString) -> Result<UniquePtr<PDFSet>>;
        fn pdfset_from_pdf(pdf: &PDF) -> UniquePtr<PDFSet>;

        fn lookup_pdf_setname(lhaid: i32, setname: Pin<&mut CxxString>);
        fn lookup_pdf_memberid(lhaid: i32) -> i32;
        fn get_pdfset_error_type(set: &PDFSet, setname: Pin<&mut CxxString>);

        fn pdf_uncertainty(
            pdfset: &PDFSet,
            values: &[f64],
            cl: f64,
            alternative: bool,
        ) -> PdfUncertainty;
    }
}

/// Error struct that wraps all exceptions thrown by the LHAPDF library.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct LhapdfError {
    exc: Exception,
}

/// Type definition for results with an [`LhapdfError`].
pub type Result<T> = result::Result<T, LhapdfError>;

pub use ffi::PdfUncertainty;

/// Get the names of all available PDF sets in the search path.
#[must_use]
pub fn available_pdf_sets() -> Vec<String> {
    ffi::availablePDFSets()
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect()
}

/// Convert an LHAID to an LHAPDF set name and member ID.
#[must_use]
pub fn lookup_pdf(lhaid: i32) -> Option<(String, i32)> {
    let_cxx_string!(cxx_setname = "");
    ffi::lookup_pdf_setname(lhaid, cxx_setname.as_mut());

    let setname = cxx_setname.to_string_lossy();
    let memberid = ffi::lookup_pdf_memberid(lhaid);

    if (setname == "") && (memberid == -1) {
        None
    } else {
        Some((setname.to_string(), memberid))
    }
}

/// Convenient way to set the verbosity level.
pub fn set_verbosity(verbosity: i32) {
    ffi::setVerbosity(verbosity);
}

/// Convenient way to get the current verbosity level.
#[must_use]
pub fn verbosity() -> i32 {
    ffi::verbosity()
}

/// Wrapper to an LHAPDF object of the type `LHAPDF::PDF`.
pub struct Pdf {
    ptr: UniquePtr<ffi::PDF>,
}

impl fmt::Debug for Pdf {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Pdf")
            .field("lhaid", &self.ptr.lhapdfID())
            .finish()
    }
}

impl Pdf {
    /// Constructor. Create a new PDF with the given `lhaid` ID code.
    ///
    /// # Errors
    ///
    /// TODO
    pub fn with_lhaid(lhaid: i32) -> Result<Self> {
        ffi::pdf_with_lhaid(lhaid)
            .map(|ptr| Self { ptr })
            .map_err(|exc| LhapdfError { exc })
    }

    /// Constructor. Create a new PDF with the given PDF `setname` and `member` ID.
    ///
    /// # Errors
    ///
    /// TODO
    pub fn with_setname_and_member(setname: &str, member: i32) -> Result<Self> {
        let_cxx_string!(cxx_setname = setname.to_string());
        ffi::pdf_with_setname_and_member(&cxx_setname, member)
            .map(|ptr| Self { ptr })
            .map_err(|exc| LhapdfError { exc })
    }

    /// Get the PDF `x * f(x)` value at `x` and `q2` for the given PDG ID.
    ///
    /// # Panics
    ///
    /// If the value of either `x` or `q2` is not within proper boundaries this method will panic.
    #[must_use]
    pub fn xfx_q2(&self, id: i32, x: f64, q2: f64) -> f64 {
        self.ptr.xfxQ2(id, x, q2).unwrap()
    }

    /// Value of of the strong coupling at `q2` used by this PDF.
    ///
    /// # Panics
    ///
    /// If the value of `q2` is not within proper boundaries this method will panic.
    #[must_use]
    pub fn alphas_q2(&self, q2: f64) -> f64 {
        self.ptr.alphasQ2(q2).unwrap()
    }

    /// Get the info class that actually stores and handles the metadata.
    #[must_use]
    pub fn set(&self) -> PdfSet {
        PdfSet {
            ptr: ffi::pdfset_from_pdf(&self.ptr),
        }
    }
}

unsafe impl Send for Pdf {}
unsafe impl Sync for Pdf {}

/// Class for PDF set metadata and manipulation.
pub struct PdfSet {
    ptr: UniquePtr<ffi::PDFSet>,
}

impl fmt::Debug for PdfSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("PdfSet")
            .field("lhaid", &self.ptr.lhapdfID())
            .finish()
    }
}

impl PdfSet {
    /// Constructor from a set name.
    #[must_use]
    pub fn new(setname: &str) -> Result<Self> {
        let_cxx_string!(cxx_setname = setname);

        ffi::pdfset_new(&cxx_setname)
            .map(|ptr| Self { ptr })
            .map_err(|exc| LhapdfError { exc })
    }

    /// Retrieve a metadata string by key name.
    #[must_use]
    pub fn entry(&self, key: &str) -> Option<String> {
        let_cxx_string!(cxx_key = key);

        if self.ptr.has_key(&cxx_key) {
            Some(self.ptr.get_entry(&cxx_key).to_string_lossy().into_owned())
        } else {
            None
        }
    }

    /// Get the type of PDF errors in this set (replicas, symmhessian, hessian, custom, etc.).
    pub fn error_type(&self) -> String {
        let_cxx_string!(string = "");

        ffi::get_pdfset_error_type(&self.ptr, string.as_mut());
        string.to_string_lossy().into_owned()
    }

    /// Make all the PDFs in this set.
    pub fn mk_pdfs(&self) -> Vec<Pdf> {
        (0..i32::try_from(self.ptr.size()).unwrap_or_else(|_| unreachable!()))
            .map(|member| Pdf {
                ptr: ffi::pdf_with_set_and_member(&self.ptr, member)
                    .unwrap_or_else(|_| unreachable!()),
            })
            .collect()
    }

    /// Calculate central value and error from vector values with appropriate formulae for this
    /// set.
    ///
    /// Warning: The values vector corresponds to the members of this PDF set and must be ordered
    /// accordingly.
    ///
    /// In the Hessian approach, the central value is the best-fit "values\[0\]" and the uncertainty
    /// is given by either the symmetric or asymmetric formula using eigenvector PDF sets.
    ///
    /// If the PDF set is given in the form of replicas, by default, the central value is given by
    /// the mean and is not necessarily "values\[0]\" for quantities with a non-linear dependence on
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
        ffi::pdf_uncertainty(&self.ptr, values, cl, alternative)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn available_pdf_sets() {
        let pdf_sets = super::available_pdf_sets();

        assert!(pdf_sets
            .iter()
            .any(|pdf_set| pdf_set == "NNPDF31_nlo_as_0118_luxqed"));
    }

    #[test]
    fn set_verbosity() {
        super::set_verbosity(0);
        assert_eq!(verbosity(), 0);
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
    fn check_pdf() -> Result<()> {
        let pdf_0 = Pdf::with_setname_and_member("NNPDF31_nlo_as_0118_luxqed", 0)?;
        let pdf_1 = Pdf::with_lhaid(324900)?;

        let value_0 = pdf_0.xfx_q2(2, 0.5, 90.0 * 90.0);
        let value_1 = pdf_1.xfx_q2(2, 0.5, 90.0 * 90.0);

        assert_ne!(value_0, 0.0);
        assert_eq!(value_0, value_1);

        let value_0 = pdf_0.alphas_q2(90.0 * 90.0);
        let value_1 = pdf_1.alphas_q2(90.0 * 90.0);

        assert_ne!(value_0, 0.0);
        assert_eq!(value_0, value_1);

        Ok(())
    }

    #[test]
    fn check_pdf_set() -> Result<()> {
        let pdf_set = PdfSet::new("NNPDF31_nlo_as_0118_luxqed")?;

        assert!(matches!(pdf_set.entry("Particle"), Some(value) if value == "2212"));
        assert!(matches!(pdf_set.entry("Flavors"), Some(value)
            if value == "[-5, -4, -3, -2, -1, 21, 1, 2, 3, 4, 5, 22]"));

        assert_eq!(pdf_set.error_type(), "replicas");

        Ok(())
    }
}
