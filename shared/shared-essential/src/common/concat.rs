/// Конкатенация const-срезов в один `&[T]`.
///
/// Применимо только к `T: Copy` — макрос индексирует исходные срезы оператором
/// `[i]`, который копирует элемент. Для типов без `Copy` будет ошибка компиляции
/// при индексации; ниже SAFETY полагается на корректность этой копии.
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

        // Защитная проверка: если суммарный размер не совпал с LEN, что-то
        // пошло не так в самом макросе (несогласованные lengths). В таком случае
        // паника предпочтительнее UB в transmute ниже.
        if base != LEN { panic!("invalid length"); }
        &{
            // SAFETY:
            // 1. `arr` имеет тип `[MaybeUninit<T>; LEN]`, целевой тип — `[T; LEN]`;
            //    layout идентичен по гарантиям std (`MaybeUninit` имеет тот же
            //    размер и выравнивание, что `T`).
            // 2. Все LEN позиций массива записаны через `MaybeUninit::new` в циклах
            //    выше: каждый элемент исходных срезов положен в `arr[base + i]`,
            //    а проверка `base == LEN` гарантирует, что заполнен весь массив.
            // 3. `T: Copy` (требуется для индексации `[i]` в const-контексте),
            //    поэтому копия не создаёт двойного владения.
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
