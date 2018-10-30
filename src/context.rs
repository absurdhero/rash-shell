/// An evaluation context defines evaluation settings
/// and stores the current shell state.
pub struct Context {
    pub interactive: bool,
    pub last_return: Option<i32>,
}

