//! Policy: the choices of ghg serve, whether a burst of changes under `--watch` calls for a rebuild.

mod rebuild_choice;

pub(crate) use rebuild_choice::{Read, calls_for_rebuild, remembered};
