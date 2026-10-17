use super::{File, Form, Rendered, Repository, render, thin_with};
pub const VPN: &[File] = &[
    File { path: "envs/lab.json", body: b"{}", mode: 0o644 },
    File { path: "tasks/vpn/task.json", mode: 0o644, body: br#"{"settings":{"address":{"type":"string","doc":"Address."},"endpoint":{"type":"string","optional":true,"doc":"Endpoint."},"mtu":{"type":"int","default":1420,"doc":"MTU."}},"reads":{"vpn":{"exports":true}},"exports":{"address":{"type":"string","value":"{{ settings.address }}","doc":"Address."},"endpoint":{"type":"string","optional":true,"value":"{{ settings.endpoint }}","doc":"Endpoint."}}}"# },
    File { path: "tasks/vpn/tree/etc/vpn/peers.j2", mode: 0o644, body: b"{% for name, peer in exports.vpn | items %}{% if name != target.name %}{{ name }}={{ peer.address }}{% if peer.endpoint is defined %}@{{ peer.endpoint }}{% endif %};{% endif %}{% endfor +%}\n" },
    File { path: "tasks/vpn/tree/etc/vpn/mtu.j2", mode: 0o644, body: b"{{ settings.mtu }}\n" },
    File { path: "hosts/ion/ion.json", mode: 0o644, body: br#"{"vpn":{"address":"10.0.0.1"}}"# },
    File { path: "hosts/ion/ion-prod.json", mode: 0o644, body: br#"{"placement": {"kind": "vm", "firmware": "bios", "hardware": "ionos"}, "vpn": {"endpoint": "194.164.162.157"}}"# },
    File { path: "hosts/ion/ion-lab.json", mode: 0o644, body: br#"{"placement":{"kind":"vm","firmware":"efi","hardware":"incus"},"vpn":{"address":"10.1.0.1"}}"# },
    File { path: "hosts/carbon/carbon.json", mode: 0o644, body: br#"{"vpn":{"address":"10.0.0.2"}}"# },
    File { path: "hosts/carbon/carbon-prod.json", mode: 0o644, body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"}}"# },
    File { path: "hosts/carbon/carbon-lab.json", mode: 0o644, body: br#"{"placement":{"kind":"vm","firmware":"efi","hardware":"incus"}}"# },
];
pub const VPN_BAD: File = File {
    path: "hosts/ion/ion-prod.json",
    mode: 0o644,
    body: br#"{"placement": {"kind": "vm", "firmware": "bios", "hardware": "ionos"}, "vpn": {"endpoint": 7}}"#,
};
#[must_use]
pub fn vpn(broken: bool) -> Rendered {
    let repository = Repository::make("vpn", &thin_with(VPN));
    if broken {
        drop(repository.next(&[VPN_BAD], &[]));
    }
    let names: &[&[u8]] = if broken {
        &[b"carbon-lab", b"carbon-prod", b"ion-lab", b"ion-prod"]
    } else {
        &[b"carbon-lab", b"carbon-prod"]
    };
    render(&repository, names, Form::Text)
}
