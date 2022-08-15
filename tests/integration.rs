extern crate assert_cmd;

mod integration {
    use assert_cmd::Command;

    static BIN: &'static str = "gluon-mesh-vpn-key-translate";
    static FASTD_SECRET: &'static str =
        "68c2cc80f95054b6edd70bb355142adc54d902c697fe1cedc898df71b79e4f76";
    static FASTD_PUBLIC: &'static str =
        "12760ee9088d7b737a11c21c587ab73b64be3c2834eaef6185ec66c3c05c1d78";
    static WG_SECRET: &'static str = "aMLMgPlQVLbt1wuzVRQq3FTZAsaX/hztyJjfcbeeT3Y=";
    static WG_PUBLIC: &'static str = "Gluon+LYDa2ib6EDnWlRELwrst1s1Ut7WKNR5LMI83c=";

    trait NlExt {
        fn nl(&self) -> String;
    }

    impl NlExt for str {
        fn nl(&self) -> String {
            return self.to_owned() + "\n";
        }
    }

    #[test]
    fn calling_help() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.arg("-h").assert();
        assert.success();
    }

    #[test]
    fn translate_public_key_from_argument() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.arg(FASTD_PUBLIC).assert();
        assert.success().stdout(WG_PUBLIC.nl());
    }

    #[test]
    fn translate_public_key_from_stdin() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.write_stdin(FASTD_PUBLIC).assert();
        assert.success().stdout(WG_PUBLIC.nl());
    }

    #[test]
    fn translate_private_key_from_stdin() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.arg("--private").write_stdin(FASTD_SECRET).assert();
        assert.success().stdout(WG_SECRET.nl());
    }

    #[test]
    fn translate_private_key_from_stdin_using_alias_secret() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.arg("--secret").write_stdin(FASTD_SECRET).assert();
        assert.success().stdout(WG_SECRET.nl());
    }

    #[test]
    fn translate_public_key_from_file() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.args(&["--if", "./tests/assets/fastd_public"]).assert();
        assert.success().stdout(WG_PUBLIC.nl());
    }

    #[test]
    fn translate_private_key_from_file() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd
            .args(&["--private", "--if", "./tests/assets/fastd_secret"])
            .assert();
        assert.success().stdout(WG_SECRET.nl());
    }

    // test error cases

    #[test]
    fn translate_private_key_from_argument() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.args(&["--private", FASTD_SECRET]).assert();
        assert.failure();
    }

    #[test]
    fn translate_public_key_from_argument_invalid_hex() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.arg("anything_but_hex").assert();
        assert.failure();
    }

    #[test]
    fn translate_public_key_from_argument_invalid_point_on_curve() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd.write_stdin(FASTD_SECRET).assert();
        assert.failure();
    }

    #[test]
    fn translate_public_key_from_file_invalid_path() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        let assert = cmd
            .args(&["--if", "./tests/assets/does_not_exist"])
            .assert();
        assert.failure();
    }
}
