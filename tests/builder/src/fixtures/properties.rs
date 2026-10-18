use super::{Repository, digests_of, files_of, render};
use hearth_render_world::repositories::Generated;
#[must_use]
pub fn printed(generated: &Generated) -> Vec<(String, String)> {
    let repository = Repository::make("generated-property", &files_of(&generated.leaves));
    let output = render(&repository, &[], hearth::Form::JsonLines);
    digests_of(&String::from_utf8(output.stdout).expect("JSON lines"))
}
