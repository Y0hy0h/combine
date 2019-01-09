extern crate combine;

use combine::parser::char::{digit, letter, string};
use combine::parser::choice::{choice, optional};
use combine::parser::combinator::{attempt, no_partial, not_followed_by};
use combine::parser::error::unexpected;
use combine::parser::item::{any, eof, position, token, value, Token};
use combine::parser::range::{self, range};
use combine::parser::repeat::{count, count_min_max, many, sep_by, sep_end_by1, skip_until, take_until};
use combine::Parser;

#[test]
fn choice_empty() {
    let mut parser = choice::<&mut [Token<&str>]>(&mut []);
    let result_err = parser.parse("a");
    assert!(result_err.is_err());
}

#[test]
fn tuple() {
    let mut parser = (digit(), token(','), digit(), token(','), letter());
    assert_eq!(parser.parse("1,2,z"), Ok((('1', ',', '2', ',', 'z'), "")));
}

#[test]
fn issue_99() {
    let result = any().map(|_| ()).or(eof()).parse("");
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn not_followed_by_does_not_consume_any_input() {
    let mut parser = not_followed_by(range("a")).map(|_| "").or(range("a"));

    assert_eq!(parser.parse("a"), Ok(("a", "")));

    let mut parser = range("a").skip(not_followed_by(range("aa")));

    assert_eq!(parser.parse("aa"), Ok(("a", "a")));
    assert!(parser.parse("aaa").is_err());
}

#[cfg(feature = "std")]
mod tests_std {
    use super::*;
    use combine::parser::byte::{alpha_num, bytes};
    use combine::parser::byte::num::be_u32;
    use combine::parser::char::{char, digit, letter};
    use combine::stream::easy::{self, Error, Errors};
    use combine::stream::state::{SourcePosition, State};
    use combine::Parser;

    #[derive(Clone, PartialEq, Debug)]
    struct CloneOnly {
        s: String,
    }

    #[test]
    fn token_clone_but_not_copy() {
        // Verify we can use token() with a StreamSlice with an item type that is Clone but not
        // Copy.
        let input = &[
            CloneOnly { s: "x".to_string() },
            CloneOnly { s: "y".to_string() },
        ][..];
        let result = token(CloneOnly { s: "x".to_string() }).easy_parse(input);
        assert_eq!(
            result,
            Ok((
                CloneOnly { s: "x".to_string() },
                &[CloneOnly { s: "y".to_string() }][..]
            ))
        );
    }

    #[test]
    fn sep_by_consumed_error() {
        let mut parser2 = sep_by((letter(), letter()), token(','));
        let result_err: Result<(Vec<(char, char)>, &str), easy::ParseError<&str>> =
            parser2.easy_parse("a,bc");
        assert!(result_err.is_err());
    }

    /// The expected combinator should retain only errors that are not `Expected`
    #[test]
    fn expected_retain_errors() {
        let mut parser = digit()
            .message("message")
            .expected("N/A")
            .expected("my expected digit");
        assert_eq!(
            parser.easy_parse(State::new("a")),
            Err(Errors {
                position: SourcePosition::default(),
                errors: vec![
                    Error::Unexpected('a'.into()),
                    Error::Message("message".into()),
                    Error::Expected("my expected digit".into()),
                ],
            })
        );
    }

    #[test]
    fn tuple_parse_error() {
        let mut parser = (digit(), digit());
        let result = parser.easy_parse(State::new("a"));
        assert_eq!(
            result,
            Err(Errors {
                position: SourcePosition::default(),
                errors: vec![
                    Error::Unexpected('a'.into()),
                    Error::Expected("digit".into()),
                ],
            })
        );
    }

    #[test]
    fn message_tests() {
        // Ensure message adds to both consumed and empty errors, interacting with parse_lazy and
        // parse_stream correctly on either side
        let input = "hi";

        let mut ok = char('h').message("not expected");
        let mut empty0 = char('o').message("expected message");
        let mut empty1 = char('o').message("expected message").map(|x| x);
        let mut empty2 = char('o').map(|x| x).message("expected message");
        let mut consumed0 = char('h').with(char('o')).message("expected message");
        let mut consumed1 = char('h')
            .with(char('o'))
            .message("expected message")
            .map(|x| x);
        let mut consumed2 = char('h')
            .with(char('o'))
            .map(|x| x)
            .message("expected message");

        assert!(ok.easy_parse(State::new(input)).is_ok());

        let empty_expected = Err(Errors {
            position: SourcePosition { line: 1, column: 1 },
            errors: vec![
                Error::Unexpected('h'.into()),
                Error::Expected('o'.into()),
                Error::Message("expected message".into()),
            ],
        });

        let consumed_expected = Err(Errors {
            position: SourcePosition { line: 1, column: 2 },
            errors: vec![
                Error::Unexpected('i'.into()),
                Error::Expected('o'.into()),
                Error::Message("expected message".into()),
            ],
        });

        assert_eq!(empty0.easy_parse(State::new(input)), empty_expected);
        assert_eq!(empty1.easy_parse(State::new(input)), empty_expected);
        assert_eq!(empty2.easy_parse(State::new(input)), empty_expected);

        assert_eq!(consumed0.easy_parse(State::new(input)), consumed_expected);
        assert_eq!(consumed1.easy_parse(State::new(input)), consumed_expected);
        assert_eq!(consumed2.easy_parse(State::new(input)), consumed_expected);
    }

    #[test]
    fn expected_tests() {
        // Ensure `expected` replaces only empty errors, interacting with parse_lazy and
        // parse_stream correctly on either side
        let input = "hi";

        let mut ok = char('h').expected("not expected");
        let mut empty0 = char('o').expected("expected message");
        let mut empty1 = char('o').expected("expected message").map(|x| x);
        let mut empty2 = char('o').map(|x| x).expected("expected message");
        let mut consumed0 = char('h').with(char('o')).expected("expected message");
        let mut consumed1 = char('h')
            .with(char('o'))
            .expected("expected message")
            .map(|x| x);
        let mut consumed2 = char('h')
            .with(char('o'))
            .map(|x| x)
            .expected("expected message");

        assert!(ok.easy_parse(State::new(input)).is_ok());

        let empty_expected = Err(Errors {
            position: SourcePosition { line: 1, column: 1 },
            errors: vec![
                Error::Unexpected('h'.into()),
                Error::Expected("expected message".into()),
            ],
        });

        let consumed_expected = Err(Errors {
            position: SourcePosition { line: 1, column: 2 },
            errors: vec![Error::Unexpected('i'.into()), Error::Expected('o'.into())],
        });

        assert_eq!(empty0.easy_parse(State::new(input)), empty_expected);
        assert_eq!(empty1.easy_parse(State::new(input)), empty_expected);
        assert_eq!(empty2.easy_parse(State::new(input)), empty_expected);

        assert_eq!(consumed0.easy_parse(State::new(input)), consumed_expected);
        assert_eq!(consumed1.easy_parse(State::new(input)), consumed_expected);
        assert_eq!(consumed2.easy_parse(State::new(input)), consumed_expected);
    }

    #[test]
    fn try_tests() {
        // Ensure attempt adds error messages exactly once
        assert_eq!(
            attempt(unexpected("test")).easy_parse(State::new("hi")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 1 },
                errors: vec![
                    Error::Unexpected('h'.into()),
                    Error::Unexpected("test".into()),
                ],
            })
        );
        assert_eq!(
            attempt(char('h').with(unexpected("test"))).easy_parse(State::new("hi")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 2 },
                errors: vec![
                    Error::Unexpected('i'.into()),
                    Error::Unexpected("test".into()),
                ],
            })
        );
    }

    #[test]
    fn sequence_error() {
        let mut parser = (char('a'), char('b'), char('c'));

        assert_eq!(
            parser.easy_parse(State::new("c")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 1 },
                errors: vec![Error::Unexpected('c'.into()), Error::Expected('a'.into())],
            })
        );

        assert_eq!(
            parser.easy_parse(State::new("ac")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 2 },
                errors: vec![Error::Unexpected('c'.into()), Error::Expected('b'.into())],
            })
        );
    }

    #[test]
    fn optional_empty_ok_then_error() {
        let mut parser = (optional(char('a')), char('b'));

        assert_eq!(
            parser.easy_parse(State::new("c")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 1 },
                errors: vec![
                    Error::Unexpected('c'.into()),
                    Error::Expected('a'.into()),
                    Error::Expected('b'.into()),
                ],
            })
        );
    }

    #[test]
    fn nested_optional_empty_ok_then_error() {
        let mut parser = ((optional(char('a')), char('b')), char('c'));

        assert_eq!(
            parser.easy_parse(State::new("c")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 1 },
                errors: vec![
                    Error::Unexpected('c'.into()),
                    Error::Expected('a'.into()),
                    Error::Expected('b'.into()),
                ],
            })
        );
    }

    #[test]
    fn consumed_then_optional_empty_ok_then_error() {
        let mut parser = (char('b'), optional(char('a')), char('b'));

        assert_eq!(
            parser.easy_parse(State::new("bc")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 2 },
                errors: vec![
                    Error::Unexpected('c'.into()),
                    Error::Expected('a'.into()),
                    Error::Expected('b'.into()),
                ],
            })
        );
    }

    #[test]
    fn sequence_in_choice_parser_empty_err() {
        let mut parser = choice((
            (optional(char('a')), char('1')),
            (optional(char('b')), char('2')).skip(char('d')),
        ));

        assert_eq!(
            parser.easy_parse(State::new("c")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 1 },
                errors: vec![
                    Error::Expected('a'.into()),
                    Error::Expected('1'.into()),
                    Error::Expected('b'.into()),
                    Error::Expected('2'.into()),
                    Error::Unexpected('c'.into()),
                ],
            })
        );
    }

    #[test]
    fn sequence_in_choice_array_parser_empty_err() {
        let mut parser = choice([
            (optional(char('a')), char('1')),
            (optional(char('b')), char('2')),
        ]);

        assert_eq!(
            parser.easy_parse(State::new("c")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 1 },
                errors: vec![
                    Error::Expected('a'.into()),
                    Error::Expected('1'.into()),
                    Error::Expected('b'.into()),
                    Error::Expected('2'.into()),
                    Error::Unexpected('c'.into()),
                ],
            })
        );
    }

    #[test]
    fn sequence_in_choice_array_parser_empty_err_where_first_parser_delay_errors() {
        let mut p1 = char('1');
        let mut p2 = no_partial((optional(char('b')), char('2')).map(|t| t.1));
        let mut parser =
            choice::<[&mut Parser<Input = _, Output = _, PartialState = _>; 2]>([&mut p1, &mut p2]);

        assert_eq!(
            parser.easy_parse(State::new("c")),
            Err(Errors {
                position: SourcePosition { line: 1, column: 1 },
                errors: vec![
                    Error::Expected('1'.into()),
                    Error::Expected('b'.into()),
                    Error::Expected('2'.into()),
                    Error::Unexpected('c'.into()),
                ],
            })
        );
    }

    #[test]
    fn sep_end_by1_dont_eat_separator_twice() {
        let mut parser = sep_end_by1(digit(), token(';'));
        assert_eq!(parser.parse("1;;"), Ok((vec!['1'], ";")));
    }

    #[test]
    fn count_min_max_empty_error() {
        assert_eq!(
            count_min_max(1, 1, char('a')).or(value(vec![])).parse("b"),
            Ok((vec![], "b"))
        );
    }

    #[test]
    fn sequence_parser_resets_partial_state_issue_168() {
        assert_eq!(
            take_until::<String, _>(attempt((char('a'), char('b')))).parse("aaab"),
            Ok((String::from("aa"), "ab"))
        );
    }

    #[test]
    fn parser_macro_must_impl_parse_mode_issue_168() {
        assert_eq!(
            skip_until(attempt((char('a'), char('b')))).parse("aaab"),
            Ok(((), "ab"))
        );
    }

    #[test]
    fn recognize_parser_issue_168() {
        assert_eq!(
            range::recognize(skip_until(attempt((char('a'), char('b'))))).parse("aaab"),
            Ok(("aa", "ab"))
        );
    }

    #[test]
    fn sequence_in_optional_report_delayed_error() {
        assert_eq!(
            optional(position().with(char('a')))
                .skip(char('}'))
                .easy_parse("b")
                .map_err(|e| e.errors),
            Err(vec![
                Error::Unexpected('b'.into()),
                Error::Expected('a'.into()),
                Error::Expected('}'.into()),
            ]),
        );
    }

    #[test]
    fn sequence_in_optional_nested_report_delayed_error() {
        assert_eq!(
            optional(position().with(char('a')))
                .skip(optional(position().with(char('c'))))
                .skip(char('}'))
                .easy_parse("b")
                .map_err(|e| e.errors),
            Err(vec![
                Error::Unexpected('b'.into()),
                Error::Expected('a'.into()),
                Error::Expected('c'.into()),
                Error::Expected('}'.into()),
            ]),
        );
    }

    #[test]
    fn sequence_in_optional_nested_2_report_delayed_error() {
        assert_eq!(
            (
                char('{'),
                optional(position().with(char('a')))
                    .skip(optional(position().with(char('c'))))
                    .skip(char('}'))
            )
                .easy_parse("{b")
                .map_err(|e| e.errors),
            Err(vec![
                Error::Unexpected('b'.into()),
                Error::Expected('a'.into()),
                Error::Expected('c'.into()),
                Error::Expected('}'.into()),
            ]),
        );
    }

    macro_rules! sequence_many_test {
        ($many:expr, $seq:expr) => {
            let mut parser = $seq($many(position().with(char('a'))), char('}'));
            let expected_error = Err(vec![
                Error::Unexpected('b'.into()),
                Error::Expected('a'.into()),
                Error::Expected('}'.into()),
            ]);
            assert_eq!(
                parser.easy_parse("ab").map_err(|e| e.errors),
                expected_error,
            );
        };
    }

    #[test]
    fn sequence_in_many_report_delayed_error() {
        use combine::parser::{repeat, sequence};

        sequence_many_test!(repeat::many::<Vec<_>, _>, sequence::skip);
        sequence_many_test!(repeat::many1::<Vec<_>, _>, sequence::skip);
        sequence_many_test!(repeat::many::<Vec<_>, _>, sequence::with);
        sequence_many_test!(repeat::many1::<Vec<_>, _>, sequence::with);
        sequence_many_test!(repeat::many::<Vec<_>, _>, |l, x| sequence::between(
            l,
            char('|'),
            x,
        ));
        sequence_many_test!(repeat::many1::<Vec<_>, _>, |l, x| sequence::between(
            l,
            char('|'),
            x,
        ));
    }

    macro_rules! sequence_sep_by_test {
        ($many:expr, $seq:expr) => {
            let mut parser = $seq($many(position().with(char('a')), char(',')), char('}'));
            let expected_error = Err(vec![
                Error::Unexpected('b'.into()),
                Error::Expected(','.into()),
                Error::Expected('}'.into()),
            ]);
            assert_eq!(
                parser.easy_parse("a,ab").map_err(|e| e.errors),
                expected_error,
            );
        };
    }

    #[test]
    fn sequence_in_sep_by_report_delayed_error() {
        use combine::parser::{repeat, sequence};

        sequence_sep_by_test!(repeat::sep_by::<Vec<_>, _, _>, sequence::skip);
        sequence_sep_by_test!(repeat::sep_by1::<Vec<_>, _, _>, sequence::skip);
        sequence_sep_by_test!(repeat::sep_by::<Vec<_>, _, _>, sequence::with);
        sequence_sep_by_test!(repeat::sep_by1::<Vec<_>, _, _>, sequence::with);
    }

    #[test]
    fn choice_compose_on_error() {
        let ident = |s| attempt(string(s));
        let mut parser = choice((ident("aa").skip(string(";")), choice((ident("cc"),))));

        assert_eq!(
            parser.easy_parse("c").map_err(|err| err.errors),
            Err(vec![
                Error::Unexpected('c'.into()),
                Error::Expected("aa".into()),
                Error::Unexpected("end of input".into()),
                Error::Expected("cc".into()),
            ]),
        );
    }

    #[test]
    fn choice_compose_issue_175() {
        let ident = |s| attempt(string(s));
        let mut parser = many::<Vec<_>, _>(position().and(choice((
            ident("aa").skip(string(";")),
            choice((ident("bb"), ident("cc"))),
        ))))
        .skip(string("."));

        assert_eq!(
            parser.easy_parse("c").map_err(|err| err.errors),
            Err(vec![
                Error::Unexpected('c'.into()),
                Error::Expected("aa".into()),
                Error::Expected("bb".into()),
                Error::Expected("cc".into()),
            ]),
        );
    }

    #[test]
    fn test() {
        let mut parser = (digit(), letter());

        assert_eq!(
            parser.easy_parse("11").map_err(|err| err.errors),
            Err(vec![
                Error::Unexpected('1'.into()),
                Error::Expected("letter".into()),
            ]),
        );
    }

    #[test]
    fn test_nested_count_overflow() {
        let key = || count::<Vec<_>, _>(64, alpha_num());
        let value_bytes = || be_u32()
            .then_partial(|&mut size| count::<Vec<_>, _>(size as usize, any()));
        let value_messages = (be_u32(), be_u32())
            .then_partial(|&mut (_body_size, message_count)| {
                count::<Vec<_>, _>(message_count as usize, value_bytes())
            });
        let put = (bytes(b"PUT"), key())
            .map(|(_, key)| key)
            .and(value_messages);

        let parser = || put.map(|(_, messages)| messages);

        let command = &b"PUTkey\x00\x00\x00\x12\x00\x00\x00\x02\x00\x00\x00\x04\xDE\xAD\xBE\xEF\x00\x00\x00\x02\xBE\xEF"[..];
        let result = parser().parse(command).unwrap();
        assert_eq!(2, result.0.len());
    }
}
