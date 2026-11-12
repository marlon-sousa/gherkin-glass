//! Policy: the choices of ghg serve, the bursts of changes waiting under `--watch` merged into one.

mod burst_merging;

pub(crate) use burst_merging::merged_bursts;
