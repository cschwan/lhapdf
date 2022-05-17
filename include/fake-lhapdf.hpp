#ifndef FAKE_LHAPDF_HPP
#define FAKE_LHAPDF_HPP

#include <cassert>
#include <cstddef>
#include <string>
#include <vector>

namespace LHAPDF {

std::vector<std::string> const& availablePDFSets() {
    assert(false);
}

void setVerbosity(int) {
}

int verbosity() {
    return 0;
}

struct PDF {
    double alphasQ2(double) const {
        return 0.0;
    }

    double xfxQ2(int, double, double) const {
        return 0.0;
    }

    int lhapdfID() const {
        return 0;
    }

    double xMin() {
        return 0.0;
    }

    double xMax() {
        return 1.0;
    }
};

struct PDFSet {
    bool has_key(std::string const&) const {
        return false;
    }

    std::string const& get_entry(std::string const&) const {
        assert(false);
    }

    std::size_t size() const {
        return 0;
    }

    int lhapdfID() const {
        return 0;
    }
};

}

#endif
