//! Parametric PCB operations over the KiCad IPC API.
//!
//! This module drives a running KiCad through an nng socket. It creates board
//! outlines, fiducials, and registration holes. It places footprints coarsely.
//! Every operation runs inside one commit. See `spec/SPEC.md` §13.
