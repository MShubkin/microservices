mod budget_item;
mod category;
mod okpd2;
mod plan_reasons_cancel;
mod scheduler;

#[macro_export]
macro_rules! assert_search_result {
    ($term:expr, $count:expr, $result:expr, $all:expr) => {
        let term = $term.to_lowercase();
        let matches = |s: &str| s.to_lowercase().contains(&term);
        let expected = $all
            .iter()
            .filter(|x| matches(&x.code) || matches(&x.text))
            .map(|x| (x.id, x))
            .collect::<ahash::AHashMap<_, _>>();
        let found = $result.iter().map(|x| x.id).collect::<ahash::AHashSet<_>>();
        if found.len() < $count {
            assert_eq!(expected.len(), $result.len(), "all items should be found");
        }
        $result.iter().for_each(|x| assert_eq!(expected.get(&x.id), Some(&x)));
    };
}

#[macro_export]
macro_rules! assert_found {
    ($found:expr, $id:expr, $code:expr, $text:expr) => {
        assert!($found
            .iter()
            .any(|x| x.id == $id && &x.code == $code && &x.text == $text));
    };
}
