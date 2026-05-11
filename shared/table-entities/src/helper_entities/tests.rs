use super::*;

use asez2_shared_db::test_setup::run_db_test;
use asez2_shared_db::DbItem;

#[test]
fn test_string_to_sap_id() {
    let ok_a = "abcdefghij0123456789".to_string();
    let err_a = "bcdefghij0123456789".to_string();
    let err_b = "0abcdefghij0123456789".to_string();

    let id = SapID::try_from(ok_a).unwrap();
    let err_a = SapID::try_from(err_a).unwrap_err();
    let err_b = SapID::try_from(err_b).unwrap_err();

    let exp_id = SapID {
        id: [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', '0', '1', '2', '3',
            '4', '5', '6', '7', '8', '9',
        ],
    };
    let err_a_exp = "SAP ID 'bcdefghij0123456789' format is incorrect";
    let err_b_exp = "SAP ID '0abcdefghij0123456789' format is incorrect";

    assert_eq!(id, exp_id);
    assert_eq!(err_a, err_a_exp);
    assert_eq!(err_b, err_b_exp);
}

#[test]
fn test_string_to_hex_color() {
    let ok_a = "00af99".to_string();
    let err_a = "00aa992".to_string();
    let err_b = "0aa99".to_string();
    let err_c = "00ga99".to_string();

    let hex = ColorCode::try_from(ok_a).unwrap();
    let err_a = ColorCode::try_from(err_a).unwrap_err();
    let err_b = ColorCode::try_from(err_b).unwrap_err();
    let err_c = ColorCode::try_from(err_c).unwrap_err();

    let exp_c = ColorCode {
        r: 0,
        g: 175,
        b: 153,
    };

    assert_eq!(hex, exp_c);
    assert_eq!(
        err_a,
        ColorCodeError::InvalidLength {
            expected: 6,
            found: 7
        }
    );
    assert_eq!(
        err_b,
        ColorCodeError::InvalidLength {
            expected: 6,
            found: 5
        }
    );
    assert_eq!(
        err_c,
        ColorCodeError::InvalidFormat {
            msg: String::from("код `00ga99` содержит невалидные hex символы")
        }
    );
}

#[test]
fn test_hex_to_string() {
    let inp = ColorCode {
        r: 0,
        g: 175,
        b: 153,
    };
    let exp = "00AF99";
    assert_eq!(&String::from(inp), &exp);
}

#[test]
fn test_sap_id_to_string() {
    let inp = SapID {
        id: [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', '0', '1', '2', '3',
            '4', '5', '6', '7', '8', '9',
        ],
    };
    let exp = "abcdefghij0123456789";
    assert_eq!(&String::from(inp), exp);
}

#[tokio::test]
async fn test_db_sap_hex() {
    #[derive(Default, Debug, Clone, DbItem, PartialEq)]
    #[item_table = "saphex"]
    struct SapHex {
        #[item_field_pkey]
        id: i32,
        hex: ColorCode,
        sap_id: SapID,
    }

    run_db_test(
        "saphex",
        "(id INTEGER PRIMARY KEY, hex CHAR(6) NOT NULL, sap_id CHAR(20) NOT NULL)",
        Some(
            "(id, hex, sap_id) VALUES\
            (1, '00AF99', 'abcdefghij0123456789')",
        ),
        |mut pool| async move {
            let mut items = SapHex::select_all(&mut pool).await.unwrap();
            assert_eq!(items.len(), 1);

            let item = items.pop().unwrap();
            assert_eq!(
                item,
                SapHex {
                    id: 1,
                    hex: ColorCode::try_from("00AF99").unwrap(),
                    sap_id: SapID::try_from("abcdefghij0123456789").unwrap(),
                }
            );

            let mut new = SapHex {
                id: 2,
                hex: ColorCode::try_from("18BD09").unwrap(),
                sap_id: SapID::try_from("ABCDEFGHIJ0123456789").unwrap(),
            };
            let ret = new.insert_returning(&mut pool).await.unwrap();
            assert_eq!(new, ret);
        },
    )
    .await
}
