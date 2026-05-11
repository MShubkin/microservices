use crate::presentation::dto::processing;

use processing::CalculatedPlanRep;

/// Tests whether the code in Calculated works as intended.
/// This is necessary as the calculation closure has not been used in anger
/// and we should test whether it is viable (lifetimes, etc.)
#[tokio::test]
async fn test_set_process_count_with() {
    let tooth_count = 99;
    let tooth_name = "Fang";
    let calculated = CalculatedPlanRep::default()
        .set_pricing_process_count_with(
            &["pricing_process_count", "approvers"],
            move || async move {
                let x = tooth_count / tooth_name.chars().count();
                Ok(x as u16)
            },
        )
        .await
        .unwrap();

    assert_eq!(calculated.calculated.pricing_process_count, Some(24));
}

#[test]
fn test_set_process_count() {
    let tooth_count = 99;
    let tooth_name = "Fang";
    let x = tooth_count / tooth_name.chars().count();

    let calculated = CalculatedPlanRep::default().set_pricing_process_count(
        &["pricing_process_count", "approvers"],
        x as u16,
    );

    assert_eq!(calculated.calculated.pricing_process_count, Some(24));
}
