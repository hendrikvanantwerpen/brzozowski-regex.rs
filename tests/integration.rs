#[test]
fn can_build() {
    use brzozowski_regex::Regex;

    let x = Regex::symbol(42);
    let y = Regex::empty_set();
    let z = x | y;

    assert!(z.is_match(vec![42]));
}

#[test]
fn can_use_ops() {
    use brzozowski_regex::ops::*; // required to use `.r`/`.s`/`.c` methods
    use brzozowski_regex::Regex;

    type R = Regex<usize>;

    let x: R = 42.s().c();
    let y: R = ().r();
    let z = x | y;

    assert!(z.is_match(vec![42, 42]));
}

#[test]
fn can_test_nullability() {
    use brzozowski_regex::ops::*; // required to use `.r`/`.s`/`.c` methods
    use brzozowski_regex::Regex;

    type R = Regex<usize>;

    let x: R = 42.s().c();
    let y: R = ().r();
    let z = x | y;

    assert!(z.is_nullable());
}

#[test]
fn can_display() {
    use brzozowski_regex::ops::*; // required to use `.r`/`.s`/`.c` methods
    use brzozowski_regex::Regex;

    type R = Regex<usize>;

    let x: R = 42.s().c();

    assert_eq!("42*", x.to_string());
}

#[test]
fn can_derive() {
    use brzozowski_regex::ops::*; // required to use `.r`/`.s`/`.c` methods
    use brzozowski_regex::Regex;

    type R = Regex<usize>;

    let x: R = 42.s().c();
    let y: R = ().r();
    let z = x | y;

    assert!(z.is_nullable());
}

#[test]
fn test_automaton() {
    use brzozowski_regex::ops::*; // required to use `.r`/`.s`/`.c` methods
    use brzozowski_regex::Regex;

    type R = Regex<usize>;

    let x: R = 42.s().c();

    let mut m = x.to_automaton().into_matcher();
    assert!(m.next_iter([42, 42]));
    assert_eq!(&x, m.regex());
}
