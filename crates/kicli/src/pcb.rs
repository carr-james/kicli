//! Parametric PCB operations over the KiCad inter-process API.
//!
//! This module drives a running KiCad through an nng socket. It creates board
//! outlines, fiducials, and registration holes, and it places footprints
//! coarsely. Every operation runs inside one commit, so one kicli invocation
//! becomes one undo step for the user.
