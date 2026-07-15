use common::frame::{self, FrameHeader, FRAME_HEADER_LEN};
use protocol_gateway::codec::{encode_frame, try_decode_frame, verify_crc, MAX_PAYLOAD_SIZE};

#[test]
fn incomplete_header_needs_more_data() {
    assert!(try_decode_frame(&[0_u8; 10]).unwrap().is_none());
}

#[test]
fn incomplete_payload_needs_more_data() {
    let payload = b"hello";
    let header = FrameHeader::new(
        0x0001,
        1,
        payload.len() as u32,
        frame::payload_crc32(payload),
    )
    .unwrap();
    let mut buffer = Vec::from(header.encode().as_slice());
    buffer.extend_from_slice(&payload[..3]);

    assert!(try_decode_frame(&buffer).unwrap().is_none());
}

#[test]
fn complete_frame_is_decoded() {
    let encoded = encode_frame(0x0001, 42, b"hello").unwrap();
    let (header, payload, consumed) = try_decode_frame(&encoded).unwrap().unwrap();

    assert_eq!(header.msg_type, 0x0001);
    assert_eq!(header.seq_id, 42);
    assert_eq!(payload, b"hello");
    assert_eq!(consumed, encoded.len());
}

#[test]
fn bad_magic_is_rejected() {
    let mut buffer = [0_u8; FRAME_HEADER_LEN];
    buffer[0..4].copy_from_slice(&0xDEADBEEF_u32.to_be_bytes());

    assert!(try_decode_frame(&buffer).is_err());
}

#[test]
fn crc_verification_accepts_original_and_rejects_tampering() {
    let payload = b"test payload";
    let valid = FrameHeader::new(
        0x0001,
        1,
        payload.len() as u32,
        frame::payload_crc32(payload),
    )
    .unwrap();
    let invalid = FrameHeader::new(0x0001, 1, payload.len() as u32, 0xDEADBEEF).unwrap();

    assert!(verify_crc(&valid, payload));
    assert!(!verify_crc(&invalid, payload));
}

#[test]
fn encode_decode_round_trip_including_empty_payload() {
    for payload in [b"round trip".as_slice(), b"".as_slice()] {
        let encoded = encode_frame(0xFF01, 99, payload).unwrap();
        let (header, decoded, consumed) = try_decode_frame(&encoded).unwrap().unwrap();
        assert_eq!(decoded, payload);
        assert_eq!(consumed, encoded.len());
        assert!(verify_crc(&header, &decoded));
    }
}

#[test]
fn frame_header_over_128_mib_is_rejected_before_payload_buffering() {
    assert_eq!(MAX_PAYLOAD_SIZE, 128 * 1024 * 1024);
    let declared_length = MAX_PAYLOAD_SIZE as u32 + 1;
    let mut header_only = [0_u8; FRAME_HEADER_LEN];
    header_only[0..4].copy_from_slice(&common::frame::FRAME_MAGIC.to_be_bytes());
    header_only[4..8].copy_from_slice(&0x0502_u32.to_be_bytes());
    header_only[8..12].copy_from_slice(&7_u32.to_be_bytes());
    header_only[12..16].copy_from_slice(&declared_length.to_be_bytes());

    let error = try_decode_frame(&header_only).expect_err("oversized header must be rejected");

    assert!(error.to_string().contains("128 MiB"));
}
