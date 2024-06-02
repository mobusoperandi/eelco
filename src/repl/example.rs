use anyhow::bail;

use crate::{app::state::repl_state::ExpectedResult, example_id::ExampleId};

use super::driver::ReplQuery;

#[derive(Debug, Clone)]
pub(crate) struct ReplExample {
    pub(crate) id: ExampleId,
    pub(crate) entries: ReplExampleEntries,
}

impl ReplExample {
    pub(crate) fn try_new(id: ExampleId, contents: String) -> anyhow::Result<Self> {
        Ok(Self {
            id,
            entries: contents.parse()?,
        })
    }
}

#[derive(Debug, Clone, derive_more::IntoIterator)]
pub(crate) struct ReplExampleEntries(Vec<ReplEntry>);

impl std::str::FromStr for ReplExampleEntries {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(rest) = s.strip_prefix("nix-repl> ") else {
            bail!("repl example must start with a prompt")
        };

        let entries = rest
            .split("\nnix-repl> ")
            .map(|pair| {
                let Some((query, expected)) = pair.split_once('\n') else {
                    bail!("query must be followed by a line feed")
                };

                Ok(ReplEntry::new(
                    ReplQuery::new(format!("{query}\n").try_into().unwrap()),
                    ExpectedResult(expected.trim_end().to_owned()),
                ))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self(entries))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplEntry {
    pub(crate) query: ReplQuery,
    pub(crate) expected_result: ExpectedResult,
}

impl ReplEntry {
    pub(crate) fn new(query: ReplQuery, expected_result: ExpectedResult) -> Self {
        Self {
            query,
            expected_result,
        }
    }
}

pub(crate) const NIX_REPL_LANG_TAG: &str = "nix-repl";

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::repl::example::ReplExampleEntries;

    use super::{ExpectedResult, ReplEntry, ReplQuery};

    #[derive(PartialEq, Eq, Debug)]
    struct Case {
        input: &'static str,
        expected_output: Vec<ReplEntry>,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Failure {
        case: Case,
        actual_output: Vec<ReplEntry>,
    }

    macro_rules! parse_success {
        ($(
            {
                input: $input:expr,
                expected_output: [$(
                    {
                        query: $query:expr,
                        expected_result: $expected_result:expr,
                    },
                )*],
            },
        )*) => {
            #[test]
            fn parse_success() {
                let cases = vec![$(
                    Case {
                        input: $input,
                        expected_output: vec![$(
                            ReplEntry::new(
                                ReplQuery::new($query.parse().unwrap()),
                                ExpectedResult($expected_result.to_owned()),
                            ),
                        )*],
                    },
                )*];

                test_parse_success_cases(cases);
            }
        };
    }

    fn test_parse_success_cases(cases: Vec<Case>) {
        let failures: Vec<Failure> = cases
            .into_iter()
            .filter_map(|case| {
                let actual: ReplExampleEntries = case.input.parse().unwrap();
                let actual_output = actual.0;
                if actual_output == case.expected_output {
                    None
                } else {
                    Some(Failure {
                        case,
                        actual_output,
                    })
                }
            })
            .collect();
        assert_eq!(failures, vec![]);
    }

    parse_success! [
        {
            input: indoc! {"
                nix-repl> 1 + 1
                2

            "},
            expected_output: [
                {
                    query: "1 + 1\n",
                    expected_result: "2",
                },
            ],
        },
        {
            input: indoc! {"
                nix-repl> a = 1

            "},
            expected_output: [
                {
                    query: "a = 1\n",
                    expected_result: "",
                },
            ],
        },
        {
            input: indoc! {r#"
                nix-repl> 1 + 1
                2

                nix-repl> "a" + "b"
                "ab"

            "#},
            expected_output: [
                {
                    query: "1 + 1\n",
                    expected_result: "2",
                },
                {
                    query: "\"a\" + \"b\"\n",
                    expected_result: "\"ab\"",
                },
            ],
        },
        {
            input: indoc! {r#"
                nix-repl> b = "b"

                nix-repl> 1
                1

            "#},
            expected_output: [
                {
                    query: "b = \"b\"\n",
                    expected_result: "",
                },
                {
                    query: "1\n",
                    expected_result: "1",
                },
            ],
        },
        {
            input: indoc! {r#"
                nix-repl> { a=1; b=2; }
                {
                  a = 1
                  b = 2
                }

            "#},
            expected_output: [
                {
                    query: "{ a=1; b=2; }\n",
                    expected_result: indoc! {"
                        {
                          a = 1
                          b = 2
                        }\
                    "},
                },
            ],
        },
    ];
}
