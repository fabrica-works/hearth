//! A shortest line edit script with git's three-line unified context.
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Diff {
    Same,
    Binary,
    Hunks(Box<[Hunk]>),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Hunk {
    pub before: Span,
    pub after: Span,
    pub function: Box<[u8]>,
    pub lines: Box<[(Line, Box<[u8]>)]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: u32,
    pub count: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Line {
    Context,
    Removed,
    Added,
}

#[must_use]
pub fn lines(before: &[u8], after: &[u8]) -> Diff {
    if before == after {
        return Diff::Same;
    }
    if before.iter().take(8000).chain(after.iter().take(8000)).any(|b| *b == 0) {
        return Diff::Binary;
    }
    let a: Vec<_> = before.split_inclusive(|b| *b == b'\n').collect();
    let b: Vec<_> = after.split_inclusive(|b| *b == b'\n').collect();
    let mut lengths = vec![vec![0; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lengths[i][j] =
                if a[i] == b[j] { lengths[i + 1][j + 1] + 1 } else { lengths[i + 1][j].max(lengths[i][j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut script = Vec::new();
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            script.push((Line::Context, a[i]));
            i += 1;
            j += 1;
        } else if i < a.len() && (j == b.len() || lengths[i + 1][j] >= lengths[i][j + 1]) {
            script.push((Line::Removed, a[i]));
            i += 1;
        } else {
            script.push((Line::Added, b[j]));
            j += 1;
        }
    }
    let changed: Vec<_> =
        script.iter().enumerate().filter_map(|(i, (kind, _))| (*kind != Line::Context).then_some(i)).collect();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for i in changed {
        let start = i.saturating_sub(3);
        let end = (i + 4).min(script.len());
        if let Some(last) = ranges.last_mut().filter(|last| start <= last.1) {
            last.1 = end;
        } else {
            ranges.push((start, end));
        }
    }
    let mut hunks = Vec::new();
    for (start, end) in ranges {
        let prior_a = script[..start].iter().filter(|(k, _)| *k != Line::Added).count();
        let prior_b = script[..start].iter().filter(|(k, _)| *k != Line::Removed).count();
        let count_a = script[start..end].iter().filter(|(k, _)| *k != Line::Added).count();
        let count_b = script[start..end].iter().filter(|(k, _)| *k != Line::Removed).count();
        let function = a[..prior_a]
            .iter()
            .rev()
            .find(|line| line.first().is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_' || *b == b'$'))
            .map_or(&b""[..], |line| *line);
        let mut function = &function[..function.len().min(80)];
        while function.last().is_some_and(u8::is_ascii_whitespace) {
            function = &function[..function.len() - 1];
        }
        if let Err(error) = core::str::from_utf8(function)
            && error.error_len().is_none()
        {
            function = &function[..error.valid_up_to()];
        }
        hunks.push(Hunk {
            before: Span {
                start: u32::try_from(prior_a + usize::from(count_a != 0)).expect("scenario line count"),
                count: u32::try_from(count_a).expect("scenario line count"),
            },
            after: Span {
                start: u32::try_from(prior_b + usize::from(count_b != 0)).expect("scenario line count"),
                count: u32::try_from(count_b).expect("scenario line count"),
            },
            function: function.into(),
            lines: script[start..end].iter().map(|(k, line)| (*k, (*line).into())).collect(),
        });
    }
    Diff::Hunks(hunks.into())
}
