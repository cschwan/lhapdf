[![Rust](https://github.com/cschwan/lhapdf/workflows/Rust/badge.svg)](https://github.com/cschwan/lhapdf/actions?query=workflow%3ARust)
[![Documentation](https://docs.rs/lhapdf/badge.svg)](https://docs.rs/lhapdf)
[![crates.io](https://img.shields.io/crates/v/lhapdf.svg)](https://crates.io/crates/lhapdf)

(Unofficial) Rust bindings for the [LHAPDF](https://lhapdf.hepforge.org) C++
library

# (Un)safeness

The struct `Pdf` implements `Send` and `Sync`, which is only safe as long as
the corresponding member functions in LHAPDF are truly thread safe. The
following versions are known not to be thread safe:

- 6.3.0, see [LHAPDF issue #2](https://gitlab.com/hepcedar/lhapdf/-/issues/2)
