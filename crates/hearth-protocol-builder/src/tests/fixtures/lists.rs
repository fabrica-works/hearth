pub fn only<T>(items: &[T]) -> &T {
    let [item] = items else { panic!("one item") };
    item
}
