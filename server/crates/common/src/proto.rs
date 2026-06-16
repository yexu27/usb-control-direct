//! Protobuf 生成类型。由 `build.rs` 通过 `prost-build` 在 `OUT_DIR`
//! 生成；本模块 `include!` 暴露。
//!
//! 业务模块通过 `common::proto::CmdLogin` 等路径访问。除本模块外，禁止
//! 在其他 crate 中再次手写 prost::Message 同名类型。

#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/usb_control.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn cmd_login_encode_decode_round_trip() {
        let req = CmdLogin {
            username: "admin".to_string(),
            password: "admin@123".to_string(),
        };
        let bytes = req.encode_to_vec();
        let decoded = CmdLogin::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.username, "admin");
        assert_eq!(decoded.password, "admin@123");
    }

    #[test]
    fn rsp_common_default_success_is_false() {
        let rsp = RspCommon::default();
        assert!(!rsp.success);
        assert_eq!(rsp.result_code, 0);
    }
}
