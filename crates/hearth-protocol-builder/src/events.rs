//! Pipe routing for the git child (domain/render.md, 2.2).
use crate::{git, git::Intake, render, terminal::stream_ended};
use skein_lib::{Token, stream::Up};

pub(super) fn stream_up(child: &mut git::Child, owner: Token, up: Up) -> Option<Up> {
    if child.closing {
        return None;
    }
    if let Up::Failed(_) = &up {
        child.fault = Some(render::errors::Ended::Stream);
    }
    if child.error == Some(owner) {
        match up {
            Up::Bytes(bytes) => {
                child.tail.admit(&bytes);
                child.error_state = Intake::Demand;
            }
            Up::End => {
                child.error_state = Intake::Ended;
            }
            Up::Failed(_) => {
                child.error_state = Intake::Ended;
                child.failed = true;
            }
            Up::Room => unreachable!("read-only stderr"),
        }
        return None;
    }
    if child.output == Some(owner) && (child.output_state == Intake::Draining || child.failed) {
        match up {
            Up::Bytes(_) => child.output_state = Intake::Waiting,
            Up::End => child.output_state = Intake::Ended,
            Up::Failed(_) => {
                child.output_state = Intake::Ended;
                child.failed = true;
            }
            Up::Room => unreachable!("read-only stdout"),
        }
        return None;
    }
    if child.output == Some(owner) {
        child.output_state = if stream_ended(&up) { Intake::Ended } else { Intake::Waiting };
    }
    Some(up)
}
