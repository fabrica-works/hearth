use hearth_domain_render::vocabulary::Request;
#[must_use]
pub fn configuration_requests(requests: &[Request]) -> Vec<Request> {
    requests
        .iter()
        .filter(|request| match request {
            Request::Find { .. } | Request::Resolve { .. } => false,
            Request::List { .. } | Request::Read { .. } | Request::Keep { .. } | Request::Hunks { .. } => true,
        })
        .cloned()
        .collect()
}
