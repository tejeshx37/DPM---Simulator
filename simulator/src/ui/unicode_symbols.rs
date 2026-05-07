macro_rules! symbol {
    ($name:ident, $unicode:literal) => {
        pub const $name: &'static str = $unicode;
    };
}

symbol!(PLAY, "\u{23F5}");
symbol!(PAUSE, "\u{23F8}");
symbol!(REFRESH, "\u{27F3}");
symbol!(FILE, "\u{1F5C0}");
symbol!(CROSS, "\u{2716}");
symbol!(TRASH_CAN, "\u{1F5D1}");
symbol!(WARNING, "\u{26A0}");
symbol!(SUN, "\u{2600}");
symbol!(MOON, "\u{1F319}");
symbol!(ELLIPSIS, "\u{2026}");
symbol!(GEAR, "\u{2699}");
symbol!(BULLET, "\u{2022}");
