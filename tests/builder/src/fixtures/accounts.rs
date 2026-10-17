use super::{File, Rendered, Repository};
pub const ACCOUNTS: File = File { path: "tasks/motd/task.json",mode: 0o644,
body: br#"{"groups":{"ops":{"gid":1500},"mail":{"from":"package","gid":8}},"users":{"ci":{},"free":{"uid":1000,"home":"/home/free","shell":"/bin/bash","groups":["sudo","ops"]},"postfix":{"from":"package","uid":107,"gid":113}}}"# };
pub const ACCOUNTS_MOVED: File = File { path: "tasks/motd/task.json",mode: 0o644,
body: br#"{"groups":{"mail":{"from":"package","gid":8}},"users":{"ci":{},"free":{"uid":1001,"home":"/home/free","shell":"/bin/bash","groups":["sudo","video"]},"postfix":{"from":"package","uid":107,"gid":113}}}"# };
#[must_use]
pub fn diff_accounts() -> Rendered {
    let repository = Repository::make("accounts-diff", &super::thin_with(&[ACCOUNTS]));
    let from = repository.head();
    let to = repository.next(&[ACCOUNTS_MOVED], &[]);
    super::diff(&repository, &super::Asked::new(&from, &to))
}
