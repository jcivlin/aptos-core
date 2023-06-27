#[test_only]
module 0x42::mathtest0 {
    public inline fun mul_div(a: u64, b: u64, c: u64): u64 {
        (((a as u128) * (b as u128) / (c as u128)) as u64)
    }
}

#[test_only]
module 0x42::mathtest2 {
    use 0x42::mathtest0;
    public inline fun mul_div2(a: u64, b: u64, c: u64): u64 {
        mathtest0::mul_div(b, a, c)
    }
}

#[test_only]
module 0x42::mathtest3 {
    use 0x42::mathtest2;
    public inline fun mul_div3(a: u64, b: u64, c: u64): u64 {
        mathtest2::mul_div2(b, a, c)
    }
}

#[test_only]
module 0x42::TestRecursiveNesting {
    use 0x42::mathtest0;
    use 0x42::mathtest2;
    use 0x42::mathtest3;
    #[test]
    fun test_nested_mul_div() {
        let a = mathtest0::mul_div(
                    mathtest3::mul_div3(1, 1, 1),
                    mathtest0::mul_div(1, 1, 1),
                    mathtest2::mul_div2(1, 1, 1));
        assert!(a == 1, 0);
    }
}
