use crate::repl::example::ReplExample;
use crate::repl::example::NIX_REPL_LANG_TAG;
use itertools::Itertools;

pub(crate) fn obtain(glob: &str) -> anyhow::Result<Vec<ReplExample>> {
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
                if let Some(NIX_REPL_LANG_TAG) = info.split_ascii_whitespace().next() {
                    Some((path, ast.sourcepos.start.line, literal.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|(path, line, contents)| ReplExample::try_new(path, line, contents))
        .try_collect()
}
