/// Where a page of a job's log is read, by line number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobLogCursor {
    Last,
    After(u64),
    Before(u64),
}
