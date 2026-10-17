use crate::{
    errors::{Format, Kind},
    form::{self, Form},
    limits,
    records::Record,
    tests::fixtures::{self, SchemaInput},
};
const FAULTS: &[(SchemaInput, &str, &str)] = &[
    (
        SchemaInput::Malformed(Format::Path),
        "malformed path\n",
        "{\"record\":\"error\",\"kind\":\"schema\",\"schema\":{\"format\":\"path\",\"kind\":\"malformed\"},\"class\":\"configuration\"}\n",
    ),
    (
        SchemaInput::Target { name: b"x9-prod", environment: b"prod" },
        "no target x9-prod in prod\n",
        "{\"record\":\"error\",\"kind\":\"schema\",\"schema\":{\"environment\":\"prod\",\"kind\":\"target\",\"name\":\"x9-prod\"},\"class\":\"configuration\"}\n",
    ),
    (
        SchemaInput::File(b"lab-ca/lab.crt"),
        "no file lab-ca/lab.crt at this revision\n",
        "{\"record\":\"error\",\"kind\":\"schema\",\"schema\":{\"kind\":\"file\",\"path\":\"lab-ca/lab.crt\"},\"class\":\"configuration\"}\n",
    ),
    (
        SchemaInput::Large { path: b"lab-ca/big.crt", max: 67_108_864 },
        "lab-ca/big.crt past 67108864 bytes\n",
        "{\"record\":\"error\",\"kind\":\"schema\",\"schema\":{\"kind\":\"large\",\"max\":67108864,\"path\":\"lab-ca/big.crt\"},\"class\":\"configuration\"}\n",
    ),
    (
        SchemaInput::Main,
        "debian.components must keep main on\n",
        "{\"record\":\"error\",\"kind\":\"schema\",\"schema\":{\"kind\":\"main\"},\"class\":\"configuration\"}\n",
    ),
    (
        SchemaInput::Security,
        "debian.snapshot.security is required with its pocket\n",
        "{\"record\":\"error\",\"kind\":\"schema\",\"schema\":{\"kind\":\"security\"},\"class\":\"configuration\"}\n",
    ),
];
#[test]
fn forms_and_reference_faults_print_their_message() {
    for (row, &(input, expected, _)) in FAULTS.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.kind = Kind::Schema(fixtures::schema_fault(input));
        error.at = None;
        error.also = None;
        let printed = form::print(Form::Text, &Record::Error(error), None, &limits::SHIPPED);
        assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "row {row}");
    }
}
#[test]
fn forms_and_reference_faults_print_their_json() {
    for (row, &(input, _, expected)) in FAULTS.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.kind = Kind::Schema(fixtures::schema_fault(input));
        error.at = None;
        error.also = None;
        let printed =
            form::print(Form::JsonLines, &Record::Error(error), Some(crate::records::Type::Error), &limits::SHIPPED);
        assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "row {row}");
    }
}
