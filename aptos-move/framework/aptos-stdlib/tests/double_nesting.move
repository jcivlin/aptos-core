#[test_only]
module 0x42::Dnest1 {
    public inline fun fun1(a: u64, b: u64, c: u64): u64 {
        (((2 * (a as u128)) + (3 * (b as u128))  + (5 * (c as u128))) as u64)
    }
}

#[test_only]
module 0x42::Dnest2 {
    public inline fun fun2(a: u64, b: u64, c: u64): u64 {
        (((7 * (a as u128)) + (11 * (b as u128))  + (13 * (c as u128))) as u64)
    }
}

#[test_only]
module 0x42::DoubleNestingTest {
    use 0x42::Dnest1;
    use 0x42::Dnest2;

    #[test]
    fun test_nested_fun1() {
        let a = Dnest1::fun1(2, Dnest1::fun1(3, Dnest2::fun2(4, 5, 6), 7),
                        Dnest2::fun2(8, 9, Dnest1::fun1(10, Dnest2::fun2(11, 12, 13), 14)));
        assert!(a == 81911, 0);
    }
}
