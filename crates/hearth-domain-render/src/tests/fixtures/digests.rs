use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::bytes;

pub fn hex(text: &str) -> Box<[u8]> {
    let mut digits = 0_usize;
    for byte in text.bytes() {
        if byte != b' ' {
            digits = digits.checked_add(1).expect("the fixture fits in memory");
        }
    }
    assert_eq!(digits % 2, 0, "hex pairs are whole");
    let mut result = bytes::zeroed(digits / 2);
    let mut offset = 0_usize;
    let mut high = None;
    for byte in text.bytes() {
        if byte == b' ' {
            continue;
        }
        let digit =
            u8::try_from(char::from(byte).to_digit(16).expect("a hex digit")).expect("a hex digit fits in a byte");
        if let Some(first) = high {
            let value: u8 = first;
            *result.get_mut(offset).expect("a pair has room") =
                value.checked_mul(16).expect("a high digit fits").checked_add(digit).expect("a byte fits");
            offset = offset.checked_add(1).expect("the fixture fits in memory");
            high = None;
        } else {
            high = Some(digit);
        }
    }
    result
}

pub fn digest(text: &str) -> Digest {
    let bytes = hex(text);
    Digest(bytes.as_ref().try_into().expect("a digest has thirty-two bytes"))
}
