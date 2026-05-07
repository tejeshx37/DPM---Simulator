use crate::model::project::{ClosedHandle, OpenHandle, RecentHandle, UntitledHandle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectHandle {
    #[default]
    Invalid,
    Open(OpenHandle),
    Untitled(UntitledHandle),
    Closed(ClosedHandle),
    Recent(RecentHandle),
}
