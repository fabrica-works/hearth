use super::{Repository, digests_of, files_of, render};
use hearth_render_world::repositories::Generated;
#[must_use]
pub fn printed(generated: &Generated) -> Vec<(String, String)> {
    let repository = Repository::make("generated-property", &files_of(&generated.leaves));
    let output = render(&repository, &[], hearth::Form::JsonLines);
    digests_of(&String::from_utf8(output.stdout).expect("JSON lines"))
}

/// Whole digests from the independent render-domain world, printed as JSON does.
#[must_use]
pub fn digests(generated: &Generated, seed: u64) -> Vec<(String, String)> {
    hearth_render_world::fixtures::digests(&hearth_render_world::fixtures::properties::run(generated, seed))
        .into_iter()
        .map(|(name, digest)| (name, super::hex_digest(digest.0)))
        .collect()
}
#[must_use]
pub fn scenario(files: &[super::File]) -> super::Scenario<'_> {
    super::Scenario { files, form: hearth::Form::JsonLines, ..super::thin() }
}

/// The generated SMALL fleet has six targets, two tasks and sub-KiB templates.
#[must_use]
pub fn limits() -> hearth::Limits {
    let mut limits = super::small_limits();
    limits.domain.render = hearth_render_world::fixtures::properties::LIMITS;
    let render = limits.domain.render;
    limits.protocol.template = hearth_protocol_template::limits::Limits {
        expression: render.expression,
        nesting: render.nesting,
        template: render.template,
        statements: render.statements,
    };
    limits
}
#[must_use]
pub fn cut_pipes(files: &[super::File], seed: u64) -> crate::protocol::Run {
    crate::protocol::with_limits(&scenario(files), seed, &limits())
}

/// Four cut schedules per test retain all eight seeds for each of three fleets.
const CUT_BATCHES_PER_CASE: u64 = 2;
const CUT_SCHEDULES_PER_BATCH: u64 = super::WORLD_SEEDS.end / CUT_BATCHES_PER_CASE;
pub fn cut_pipe_digests(batch: u64) {
    let case = batch / CUT_BATCHES_PER_CASE;
    let generated = hearth_render_world::fixtures::properties::generated(case);
    let files = super::files_of(&generated.leaves);
    let digests = self::digests(&generated, case);
    let start = batch % CUT_BATCHES_PER_CASE * CUT_SCHEDULES_PER_BATCH;
    for seed in start..start + CUT_SCHEDULES_PER_BATCH {
        let run = cut_pipes(&files, seed);
        assert_eq!(super::digests_of(&super::text(&run)), digests, "case {case}, seed {seed}");
    }
}
