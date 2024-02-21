use crate::example_id::ExampleId;
use crate::expression::ExpressionExample;
use crate::repl::example::ReplExample;
use crate::repl::example::NIX_REPL_LANG_TAG;
use anyhow::Context;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub(crate) enum Example {
    Repl(ReplExample),
    Expression(ExpressionExample),
}

pub(crate) fn obtain(glob: &str) -> anyhow::Result<Vec<Example>> {
    glob::glob(glob)?
        .map(|path| {
            let path = camino::Utf8PathBuf::try_from(path?)?;
            let contents = std::fs::read_to_string(path.clone())?;
            anyhow::Ok((path, contents))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|(path, contents)| {
            let arena = comrak::Arena::new();
            let ast = comrak::parse_document(&arena, &contents, &comrak::ComrakOptions::default());
            ast.traverse()
                .filter_map(move |node_edge| match node_edge {
                    comrak::arena_tree::NodeEdge::Start(node) => {
                        let ast = node.data.borrow().clone();
                        Some((path.clone(), ast))
                    }
                    comrak::arena_tree::NodeEdge::End(_) => None,
                })
                .collect::<Vec<_>>()
        })
        .filter_map(|(path, ast)| {
            if let comrak::nodes::NodeValue::CodeBlock(code_block) = ast.value {
                let comrak::nodes::NodeCodeBlock { info, literal, .. } = code_block;
                let line = ast.sourcepos.start.line;
                let id = ExampleId::new(path, line);

                let maybe_result = match info.split_ascii_whitespace().next() {
                    Some(NIX_REPL_LANG_TAG) => {
                        let repl_example =
                            ReplExample::try_new(id.clone(), literal.clone()).map(Example::Repl);
                        Some(repl_example)
                    }
                    Some("nix") => {
                        let expression_example =
                            ExpressionExample::new(id.clone(), literal.clone());
                        Some(Ok(Example::Expression(expression_example)))
                    }
                    _ => None,
                };

                maybe_result.map(|result| result.context(format!("{id}")))
            } else {
                None
            }
        })
        .try_collect()
}
