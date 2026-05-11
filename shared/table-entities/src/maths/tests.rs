#![allow(clippy::inconsistent_digit_grouping)]
use super::*;

#[test]
fn test_rem() {
    assert_eq!(roundn(1000, 1000), 1000);
    assert_eq!(roundn(1345, 1000), 1000);
    assert_eq!(roundn(1499, 1000), 1000);
    assert_eq!(roundn(1500, 1000), 2000);
    assert_eq!(roundn(2000, 1000), 2000);

    assert_eq!(roundn(1000, 100), 1000);
    assert_eq!(roundn(1345, 100), 1300);
    assert_eq!(roundn(1499, 100), 1500);
    assert_eq!(roundn(1500, 100), 1500);
    assert_eq!(roundn(2000, 100), 2000);
}

#[test]
fn test_sum_x_quant() {
    assert_eq!(sum_x_quant(2000, 1000), 2000);
    assert_eq!(sum_x_quant(2000, 10_000), 20_000);
    assert_eq!(sum_x_quant(2000, 100), 200);
    assert_eq!(sum_x_quant(2345, 100), 235);
    assert_eq!(sum_x_quant(2344, 100), 234);
    assert_eq!(sum_x_quant(2345, 200), 469);
    assert_eq!(sum_x_quant(2342, 200), 468);
}

#[test]
fn test_basic_quantity() {
    let q: Quantity = 50i64.into();
    let q2: Quantity = 50.345f64.into();
    let q3: Quantity = 50.545f64.into();

    assert_eq!(q, Quantity(50_000));
    assert_eq!(q2, Quantity(50_345));
    assert_eq!(q3, Quantity(50_545));

    let v: i64 = q.into();
    let v2: i64 = q2.into();
    let v3: i64 = q3.into();

    assert_eq!(v, 50);
    assert_eq!(v2, 50);
    assert_eq!(v3, 51);
}

#[test]
fn test_basic_currency() {
    let q: CurrencyValue = 50i64.into();
    let q2: CurrencyValue = 50.35f64.into();
    let q3: CurrencyValue = 50.55f64.into();

    assert_eq!(q, CurrencyValue(50_00));
    assert_eq!(q2, CurrencyValue(50_35));
    assert_eq!(q3, CurrencyValue(50_55));

    let v: i64 = q.into();
    let v2: i64 = q2.into();
    let v3: i64 = q3.into();

    assert_eq!(v, 50);
    assert_eq!(v2, 50);
    assert_eq!(v3, 51);
}

#[test]
fn test_basic_rate() {
    let q: CurrencyRate = 50i64.into();
    let q2: CurrencyRate = 50.34567f64.into();
    let q3: CurrencyRate = 50.54567f64.into();

    assert_eq!(q, CurrencyRate(50_00000));
    assert_eq!(q2, CurrencyRate(50_34567));
    assert_eq!(q3, CurrencyRate(50_54567));

    let v: i64 = q.into();
    let v2: i64 = q2.into();
    let v3: i64 = q3.into();

    assert_eq!(v, 50);
    assert_eq!(v2, 50);
    assert_eq!(v3, 51);
}

#[test]
fn convert_value() {
    let sum: CurrencyValue = 100_000i64.into();
    let rate1: CurrencyRate = 51.2f64.into();
    let rate2: CurrencyRate = 0.04567f64.into();

    let converted_sum1 = rate1.convert_value(sum);
    let converted_sum2 = rate2.convert_value(sum);

    assert_eq!(converted_sum1, CurrencyValue(5_120_000_00));
    assert_eq!(converted_sum2, CurrencyValue(4_567_00));

    let v: i64 = converted_sum1.into();
    let v2: i64 = converted_sum2.into();

    assert_eq!(v, 5_120_000);
    assert_eq!(v2, 4_567);
}

#[test]
fn sum_value() {
    let sum: CurrencyValue = 100_000i64.into();
    let q1: Quantity = 1234i64.into();
    let q2: Quantity = 1.345f64.into();

    let total1 = q1.sum_value(sum);
    let total2 = q2.sum_value(sum);

    assert_eq!(total1, CurrencyValue(123_400_000_00));
    assert_eq!(total2, CurrencyValue(134_500_00));

    let v: i64 = total1.into();
    let v2: i64 = total2.into();

    assert_eq!(v, 123_400_000);
    assert_eq!(v2, 134_500);
}

#[test]
fn display_currency_value() {
    let a = format!("{}", CurrencyValue(12345678_90));
    let b = format!("{}", CurrencyValue(432234_00));
    let c = format!("{}", CurrencyValue(4));
    let d = format!("{}", CurrencyValue(-34_56));
    let e = format!("{}", CurrencyValue(-4));
    assert_eq!(a, "12345678.90");
    assert_eq!(b, "432234.00");
    assert_eq!(c, "0.04");
    assert_eq!(d, "-34.56");
    assert_eq!(e, "-0.04");
}

#[test]
fn display_currency_rate() {
    let a = format!("{}", CurrencyRate(12345_67890));
    let b = format!("{}", CurrencyRate(432_23400));
    let c = format!("{}", CurrencyRate(4));
    let d = format!("{}", CurrencyRate(-1_23456));
    let e = format!("{}", CurrencyRate(-4));
    assert_eq!(a, "12345.67890");
    assert_eq!(b, "432.23400");
    assert_eq!(c, "0.00004");
    assert_eq!(d, "-1.23456");
    assert_eq!(e, "-0.00004");
}

#[test]
fn display_quantity() {
    let a = format!("{}", Quantity(1234567_890));
    let b = format!("{}", Quantity(43223_400));
    let c = format!("{}", Quantity(4));
    let d = format!("{}", Quantity(-3_456));
    let e = format!("{}", Quantity(-4));
    assert_eq!(a, "1234567.890");
    assert_eq!(b, "43223.400");
    assert_eq!(c, "0.004");
    assert_eq!(d, "-3.456");
    assert_eq!(e, "-0.004");
}

#[test]
fn vat_ids() {
    use crate::maths::VatId;

    let q = CurrencyValue(200_00);

    assert_eq!(VatId::NoVat.vat(q), 0.into());
    assert_eq!(VatId::R0.vat(q), 0.into());
    assert_eq!(VatId::R10.vat(q), 20.into());
    assert_eq!(VatId::R18.vat(q), 36.into());
    assert_eq!(VatId::R20.vat(q), 40.into());
    assert_eq!(VatId::R12.vat(q), 24.into());
    assert_eq!(VatId::R21.vat(q), 42.into());
    assert_eq!(VatId::R13.vat(q), 26.into());
    assert_eq!(VatId::R25.vat(q), 50.into());
    assert_eq!(VatId::R15.vat(q), 30.into());
    assert_eq!(VatId::R11.vat(q), 22.into());
}

#[test]
fn test_parse_vat_id() {
    for i in 0_i16..=12 {
        let s = i.to_string();
        let r = s.parse::<VatId>().expect("should be parsable");
        assert_eq!(r, i.into());
    }
}

#[test]
fn try_from_int_tests() {
    let big_num = i64::MAX;

    let v = CurrencyValue::from_i64(big_num);
    let v_ok = CurrencyValue::from_i64(big_num / CURRENCY_VALUE_RATIO);

    let r = CurrencyRate::from_i64(big_num);
    let r_ok = CurrencyRate::from_i64(big_num / CURRENCY_RATE_RATIO);

    let q = Quantity::from_i64(big_num);
    let q_ok = Quantity::from_i64(big_num / QUANTITY_RATIO);

    assert_eq!(
        &v.unwrap_err().to_string(),
        "'9223372036854775807' слишком велико чтобы задействовать в валютах."
    );
    assert_eq!(
        &r.unwrap_err().to_string(),
        "'9223372036854775807' слишком велико чтобы задействовать в валютах."
    );
    assert_eq!(
        &q.unwrap_err().to_string(),
        "'9223372036854775807' слишком велико чтобы задействовать в валютах."
    );
    assert_eq!(v_ok.unwrap(), CurrencyValue(9223372036854775800));
    assert_eq!(r_ok.unwrap(), CurrencyRate(9223372036854700000));
    assert_eq!(q_ok.unwrap(), Quantity(9223372036854775000));
}

#[test]
fn test_cmp() {
    let a = CurrencyValue(666);
    let b = CurrencyValue(777);
    let mut v = vec![a, b];
    let exp = vec![b, a];

    v.sort_by(|a, b| b.cmp(a));
    assert_eq!(v, exp);
}

mod currency_types {
    use super::*;
    use asez2_shared_db::test_setup::run_db_test;
    use asez2_shared_db::DbItem;

    const CREATE: &str = "(
        id BIGINT PRIMARY KEY,
        weight_kg BIGINT NOT NULL,
        price_total BIGINT NOT NULL,
        price_horns BIGINT NOT NULL,
        price_meat_kg BIGINT NOT NULL,
        price_hide BIGINT NOT NULL,
        local_rate BIGINT NOT NULL
    )";
    #[derive(DbItem, PartialEq, Debug, Clone)]
    #[item_table = "moose"]
    struct Moose {
        #[item_field_pkey]
        id: i64,
        weight_kg: Quantity,
        price_total: CurrencyValue,
        price_horns: CurrencyValue,
        price_meat_kg: CurrencyValue,
        price_hide: CurrencyValue,
        local_rate: CurrencyRate,
    }

    #[tokio::test]
    async fn test_currency() {
        run_db_test(Moose::TABLE, CREATE, None, |mut pool| async move {
            let mut new = Moose {
                id: 1,
                weight_kg: 523.45.into(),
                price_total: CurrencyValue::from(-1_i64),
                price_horns: 200.into(),
                price_meat_kg: 20.into(),
                price_hide: 2000.into(),
                local_rate: 99.98765.into(),
            };
            let ret = new.insert_returning(&mut pool).await.unwrap();

            assert_eq!(ret, new);

            let total = new.weight_kg.sum_value(new.price_meat_kg)
                + new.price_horns
                + new.price_hide;
            assert_eq!(i64::from(total), (20.0 * 523.45 + 2000. + 200.) as i64);

            new.price_total = total;
            let ret = new
                .update_returning::<_, &str>(None, None, &mut pool)
                .await
                .unwrap();

            assert_eq!(ret, new);
        })
        .await
    }
}
