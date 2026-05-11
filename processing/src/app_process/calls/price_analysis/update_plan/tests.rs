#[test]
fn update_vat_id() {
    use asez2_tables::maths::VatId::{self, *};
    for (vals, exp) in [
        (&[] as &[VatId], None),
        (&[NoVat], Some(NoVat)),
        (&[R20, R20, R20], Some(R20)),
        (&[NoVat, R0], Some(Compound)),
    ] {
        let mut act = None;
        for v in vals {
            super::update_vat_id(&mut act, *v);
        }
        assert_eq!(act, exp);
    }
}
