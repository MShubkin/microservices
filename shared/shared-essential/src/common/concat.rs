/// Конкатенация конст массивов в &[T]
/// Первым элементом передается дефолтное значение для T, дальше
/// идут массивы
///
/// ```ignore
/// const A: &[&str] = &["1", "2"];
/// const B: &[&str] = &["3", "4"];
/// const C: &[&str] = concat_slice!(A, B);
/// assert_eq!(C, ["1", "2", "3", "4"])
/// ```
#[macro_export]
macro_rules! concat_slice {
    ($($s:expr),+) => {{
        use std::mem::{MaybeUninit, transmute};
        const LEN: usize = $( $s.len() + )* 0;

        let mut arr = [MaybeUninit::uninit(); LEN];
        let mut base: usize = 0;
        $({
            let mut i = 0;
            while i < $s.len() {
                arr[base + i] = MaybeUninit::new($s[i]);
                i += 1;
            }
            base += $s.len();
        })*

        if base != LEN { panic!("invalid length"); }
        &{
            // SAFETY: Проверкой выше мы убедились, что все
            // элементы были проинициализированы
            unsafe { transmute::<_, [_; LEN]>(arr) }
        }
    }}
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_usage() {
        const A: &[&str] = &["1", "2"];
        const B: &[&str] = &["3", "4"];
        const C: &[&str] = concat_slice!(A, B);

        assert_eq!(C, ["1", "2", "3", "4"])
    }
}
