pub fn override_lifetime<'a, 'b, T>(x: &'a T) -> &'b T {
    unsafe { std::mem::transmute(x) }
}
