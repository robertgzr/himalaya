use erased_serde::Serialize;
use std::fmt::Debug;

use crate::{output::PrintTable, tui::RenderTuiTable};

pub trait Mboxes: Debug + Serialize + PrintTable + RenderTuiTable {
    //
}
