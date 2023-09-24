use crate::{base_trait::ToRange, MutableStringSequence, SharedStringSequence, StringSequence};

#[test]
fn basics() {
    let hello_world = ["Hello,", " World!", "asd", " ahgteaw", "adsgads", "dsagkd"];

    assert_eq!(
        dbg!(MutableStringSequence::from_iter(hello_world)),
        StringSequence::from_iter(hello_world)
    );

    assert_eq!(
        MutableStringSequence::from_iter(hello_world),
        SharedStringSequence::from_iter(hello_world)
    );
}

#[test]
fn mutation() {
    let mut seq = MutableStringSequence::new();

    seq.push_back("hello");
    assert!(seq.iter().eq(["hello"]));
    assert!(seq.clone().into_string_sequence().iter().eq(["hello"]));
    assert_eq!(seq.text(), "hello");

    seq.push_back("world");
    assert!(seq.iter().eq(["hello", "world"]));
    assert!(seq.clone().into_string_sequence().iter().eq(["hello", "world"]));
    assert_eq!(seq.text(), "helloworld");

    seq.insert(0, "!");
    assert!(dbg!(&seq).iter().eq(["!", "hello", "world"]));
    assert!(dbg!(&seq.clone().into_string_sequence()).iter().eq(["!", "hello", "world"]));
    assert_eq!(seq.text(), "!helloworld");

    seq.insert(0, "howdy");
    assert!(seq.iter().eq(["howdy", "!", "hello", "world"]));
    assert!(seq.clone().into_string_sequence().iter().eq(["howdy", "!", "hello", "world"]));
    assert_eq!(seq.text(), "howdy!helloworld");

    assert!(seq.drain(1..3).eq(["!", "hello"]));
    assert!(seq.iter().eq(["howdy", "world"]));
    assert!(seq.clone().into_string_sequence().iter().eq(["howdy", "world"]));
    assert_eq!(seq.text(), "howdyworld");

    assert_eq!(seq.drain(0..0).count(), 0);
    assert!(seq.iter().eq(["howdy", "world"]));

    seq.extend(seq.clone().drain(..).chain(seq.clone().drain(..)));
    assert!(seq.iter().eq(["howdy", "world", "howdy", "world", "howdy", "world"]));
}

macro_rules! generate_view_test {
    ($func_name:ident, $type_name:ty) => {
        fn $func_name(view: $type_name, expected: &[&str]) {
            assert!(view.iter().eq(expected.iter().copied()));
            assert_eq!(view.len(), expected.len());
            assert_eq!(view.text(), expected.join(""));
            assert_eq!(view.first(), expected.first().copied());
            assert_eq!(view.last(), expected.last().copied());

            let array_len = expected.len();
            let ranges = [
                ToRange::to_range(.., array_len),
                ToRange::to_range(..0, array_len),
                ToRange::to_range(0.., array_len),
                ToRange::to_range(..array_len / 2, array_len),
                ToRange::to_range(0..array_len, array_len),
                ToRange::to_range(0..array_len / 2, array_len),
                ToRange::to_range(array_len / 2..array_len, array_len),
            ];

            assert!(view.starts_with(&expected[..]));
            assert!(view.starts_with(&expected[..array_len / 2]));
            assert!(view.ends_with(&expected[array_len / 2..array_len]));

            for range in ranges.iter() {
                assert!(view.slice(range.clone()).eq(expected[range.clone()].iter().copied()));
                assert!(view.contains(&expected[range.clone()]));
            }
        }
    };
}

generate_view_test!(test_view_seq, StringSequence);
generate_view_test!(test_view_mut, MutableStringSequence);
generate_view_test!(test_view_share, SharedStringSequence);

#[test]
#[cfg(feature = "serde")]
fn view() {
    let vars: &[&[&str]] = &[
        &["dsagdsaf", "ewarsdag", "adsgsdag", "dfd0k99", "llas0px;;;"], // only ascii
        &["dsagdsaf", "ewarsdag", "adsgsdag", "ㅇㄴ미ㅠ채ㅑㅁㄷ", "ㅇㄴ미ㅏㅊ"], // with unicode
        &["dsagdsaf", "ashg", "asdglkjic090a", "ㅇㄴ미ㅠ채ㅑㅁㄷ", "ㅇㄴ미ㅏㅊ"],
        &["ㅐㄴㅇ0", "ㄹㅇㄴ02.,", " ㅇㄴ마🤣🤣🤣", "ㅇㄴㅁ000"], // with emoji (4B)
        &["asdlk0f99"],
        &[],
        &["--9dsc0", "0as-=-ㄴㅁ0", "ㅊ,ㅍ0009", "ㄴ00ㅏㅏ0-ㅔ;"],
    ];

    for var in vars.iter().copied() {
        let view = MutableStringSequence::from_slice(var);
        test_view_seq(view.clone().into(), var);

        let shared = SharedStringSequence::from(&view);
        test_view_share(shared.clone(), var);

        let range_begin = rand::random::<u8>() % 8;
        let range_size = rand::random::<u8>() % 8;
        let begin = (range_begin as usize).min(var.len());
        let end = (begin + range_size as usize).min(var.len());

        test_view_share(shared.subsequence(begin..end), &var[begin..end]);

        test_view_mut(view, var);
    }

    let var = ["dsagdsaf", "ewarsdag", "adsgsdag", "ㅇㄴ미ㅠ채ㅑㅁㄷ", "ㅇㄴ미ㅏㅊ"];
    let ser = serde_json::to_string(&var).unwrap();
    let de: MutableStringSequence = serde_json::from_str(&ser).unwrap();
    let de_ser = serde_json::to_string(&de).unwrap();
    assert_eq!(ser, de_ser);

    test_view_seq(de.clone().into(), &var);
    test_view_share(de.clone().into(), &var);
    test_view_mut(de, &var);
}

#[test]
fn stability() {
    for _ in 0..5 {
        let var: Vec<_> = (0..5000)
            .map(|_| String::from_iter((0..rand::random::<u8>()).map(|_| rand::random::<char>())))
            .collect();

        let var = Vec::from_iter(var.iter().map(|x| x.as_str()));
        let var = &var[..];
        let view = MutableStringSequence::from_slice(&var);
        test_view_seq(view.clone().into(), var);
        test_view_share(view.clone().into(), var);
        test_view_mut(view, var);
    }
}
