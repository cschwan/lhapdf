#ifndef WRAPPERS_HPP
#define WRAPPERS_HPP

#include <LHAPDF/LHAPDF.h>
#include <lhapdf/src/lib.rs.h>
#include <rust/cxx.h>

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

namespace LHAPDF {

inline std::unique_ptr<PDF> pdf_with_setname_and_member(
    std::string const& setname,
    std::int32_t member
) {
    return std::unique_ptr<PDF>(mkPDF(setname, member));
}

inline std::unique_ptr<PDF> pdf_with_set_and_member(
    PDFSet const& set,
    std::int32_t member
) {
    return pdf_with_setname_and_member(set.name(), member);
}

inline std::unique_ptr<PDF> pdf_with_lhaid(std::int32_t lhaid) {
    return std::unique_ptr<PDF>(mkPDF(lhaid));
}

inline std::unique_ptr<PDFSet> pdfset_new(std::string const& setname) {
    return std::unique_ptr<PDFSet>(new PDFSet(setname));
}

inline std::unique_ptr<PDFSet> pdfset_from_pdf(PDF const& pdf) {
    return std::unique_ptr<PDFSet>(new PDFSet(pdf.set()));
}

inline void lookup_pdf_setname(std::int32_t lhaid, std::string& setname) {
    setname = lookupPDF(lhaid).first;
}

inline std::int32_t lookup_pdf_memberid(std::int32_t lhaid) {
    return lookupPDF(lhaid).second;
}

inline void get_pdfset_error_type(PDFSet const& set, std::string& error_type) {
    error_type = set.errorType();
}

inline PdfUncertainty pdf_uncertainty(
    PDFSet const& pdfset,
    rust::Slice<double const> values,
    double cl,
    bool alternative
) {
    std::vector<double> const vector(values.begin(), values.end());
    auto const uncertainty = pdfset.uncertainty(vector, cl, alternative);

    PdfUncertainty result;
    result.central = uncertainty.central;
    result.errplus = uncertainty.errplus;
    result.errminus = uncertainty.errminus;
    result.errsymm = uncertainty.errsymm;
    result.scale = uncertainty.scale;
    result.errplus_pdf = uncertainty.errplus_pdf;
    result.errminus_pdf = uncertainty.errminus_pdf;
    result.errsymm_pdf = uncertainty.errsymm_pdf;
    result.err_par = uncertainty.err_par;

    return result;
}

}

#endif
