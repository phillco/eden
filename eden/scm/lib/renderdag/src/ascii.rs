/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use itertools::Itertools;

use crate::render::{Ancestor, GraphRowRenderer, LinkLine, NodeLine, PadLine, Renderer};

pub struct AsciiRenderer<N> {
    inner: GraphRowRenderer<N>,
    min_height: usize,
    extra_pad_line: Option<String>,
}

impl<N> AsciiRenderer<N> {
    pub(crate) fn new(inner: GraphRowRenderer<N>, min_height: usize) -> Self {
        AsciiRenderer {
            inner,
            min_height,
            extra_pad_line: None,
        }
    }
}

impl<N> Renderer<N> for AsciiRenderer<N>
where
    N: Clone + Eq,
{
    type Output = String;

    fn width(&self, node: Option<&N>, parents: Option<&Vec<Ancestor<N>>>) -> u64 {
        self.inner
            .width(node, parents)
            .saturating_mul(2)
            .saturating_add(1)
    }

    fn reserve(&mut self, node: N) {
        self.inner.reserve(node);
    }

    fn next_row(
        &mut self,
        node: N,
        parents: Vec<Ancestor<N>>,
        glyph: String,
        message: String,
    ) -> String {
        let line = self.inner.next_row(node, parents, glyph, message);
        let mut out = String::new();
        let mut message_lines = line.message.lines().pad_using(self.min_height, |_| "");
        let mut need_extra_pad_line = false;

        // Render the previous extra pad line
        if let Some(extra_pad_line) = self.extra_pad_line.take() {
            out.push_str(extra_pad_line.trim_end());
            out.push_str("\n");
        }

        // Render the nodeline
        let mut node_line = String::new();
        for entry in line.node_line.iter() {
            match entry {
                NodeLine::Node => {
                    node_line.push_str(&line.glyph);
                    node_line.push_str(" ");
                }
                NodeLine::Parent => node_line.push_str("| "),
                NodeLine::Ancestor => node_line.push_str(": "),
                NodeLine::Blank => node_line.push_str("  "),
            }
        }
        if let Some(msg) = message_lines.next() {
            node_line.push_str(" ");
            node_line.push_str(msg);
        }
        out.push_str(node_line.trim_end());
        out.push_str("\n");

        // Render the link line
        if let Some(link_row) = line.link_line {
            let mut link_line = String::new();
            for (prev, cur, next) in Some(LinkLine::empty())
                .iter()
                .chain(link_row.iter())
                .chain(Some(LinkLine::empty()).iter())
                .tuple_windows()
            {
                let adj = prev.clone() | next.clone();

                // Draw the parent/ancestor line.
                if cur.contains(LinkLine::HORIZONTAL) {
                    if cur.intersects(LinkLine::CHILD | LinkLine::ANY_FORK_OR_MERGE) {
                        link_line.push_str("+");
                    } else {
                        link_line.push_str("-");
                    }
                } else if cur.intersects(LinkLine::PARENT | LinkLine::ANCESTOR) {
                    if cur.intersects(LinkLine::ANY_FORK_OR_MERGE)
                        && adj.contains(LinkLine::HORIZONTAL)
                    {
                        link_line.push_str("+");
                    } else if cur.contains(LinkLine::ANCESTOR) {
                        link_line.push_str(":");
                    } else {
                        link_line.push_str("|");
                    }
                } else if cur.intersects(LinkLine::ANY_MERGE) && adj.contains(LinkLine::HORIZONTAL)
                {
                    link_line.push_str("'");
                } else if cur.intersects(LinkLine::ANY_FORK) && adj.contains(LinkLine::HORIZONTAL) {
                    link_line.push_str(".");
                } else {
                    link_line.push_str(" ");
                }

                // Draw the connecting line.
                if cur.contains(LinkLine::HORIZONTAL) {
                    link_line.push_str("-");
                } else if cur.contains(LinkLine::RIGHT_MERGE) {
                    if next.contains(LinkLine::LEFT_FORK) && !next.contains(LinkLine::HORIZONTAL) {
                        link_line.push_str("\\");
                    } else {
                        link_line.push_str("-");
                    }
                } else if cur.contains(LinkLine::RIGHT_FORK) {
                    if next.contains(LinkLine::LEFT_MERGE) && !next.contains(LinkLine::HORIZONTAL) {
                        link_line.push_str("/");
                    } else {
                        link_line.push_str("-");
                    }
                } else {
                    link_line.push_str(" ");
                }
            }
            if let Some(msg) = message_lines.next() {
                link_line.push_str(" ");
                link_line.push_str(msg);
            }
            out.push_str(link_line.trim_end());
            out.push_str("\n");
        }

        // Render the term line
        if let Some(term_row) = line.term_line {
            let term_strs = ["| ", "~ "];
            for term_str in term_strs.iter() {
                let mut term_line = String::new();
                for (i, term) in term_row.iter().enumerate() {
                    if *term {
                        term_line.push_str(term_str);
                    } else {
                        term_line.push_str(match line.pad_lines[i] {
                            PadLine::Parent => "| ",
                            PadLine::Ancestor => ": ",
                            PadLine::Blank => "  ",
                        });
                    }
                }
                if let Some(msg) = message_lines.next() {
                    term_line.push_str(" ");
                    term_line.push_str(msg);
                }
                out.push_str(term_line.trim_end());
                out.push_str("\n");
            }
            need_extra_pad_line = true;
        }

        let mut base_pad_line = String::new();
        for entry in line.pad_lines.iter() {
            base_pad_line.push_str(match entry {
                PadLine::Parent => "| ",
                PadLine::Ancestor => ": ",
                PadLine::Blank => "  ",
            });
        }

        // Render any pad lines
        for msg in message_lines {
            let mut pad_line = base_pad_line.clone();
            pad_line.push_str(" ");
            pad_line.push_str(msg);
            out.push_str(pad_line.trim_end());
            out.push_str("\n");
            need_extra_pad_line = false;
        }

        if need_extra_pad_line {
            self.extra_pad_line = Some(base_pad_line);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use crate::render::GraphRowRenderer;
    use crate::test_utils::render_string;

    fn render(
        dag: &str,
        heads: &[&str],
        reserve: &[&str],
        ancestors: &[(&str, &str)],
        missing: &[&str],
    ) -> String {
        render_string(
            dag,
            heads,
            reserve,
            ancestors,
            missing,
            &mut GraphRowRenderer::new().ascii(2),
        )
    }

    #[test]
    fn basic() {
        assert_eq!(
            render("A-B-C", &["C"], &[], &[], &[]),
            r#"
            o  C
            |
            o  B
            |
            o  A"#
        );
    }

    #[test]
    fn branches_and_merges() {
        assert_eq!(
            render(
                r#"
                      T /---------------N--O---\           T
                     / /                        \           \
               /----E-F-\    /-------L--M--------P--\     S--U---\
            A-B-C-D------G--H--I--J--K---------------Q--R---------V--W
                                   \--N
            "#,
                &["W"],
                &[],
                &[],
                &[],
            ),
            r#"
            o  W
            |
            o    V
            |\
            | o    U
            | |\
            | | o  T
            | | |
            | o |  S
            |   |
            o   |  R
            |   |
            o   |  Q
            |\  |
            | o |    P
            | +---.
            | | | o  O
            | | | |
            | | | o    N
            | | | |\
            | o | | |  M
            | | | | |
            | o | | |  L
            | | | | |
            o | | | |  K
            +-------'
            o | | |  J
            | | | |
            o | | |  I
            |/  | |
            o   | |  H
            |   | |
            o   | |  G
            +-----+
            |   | o  F
            |   |/
            |   o  E
            |   |
            o   |  D
            |   |
            o   |  C
            +---'
            o  B
            |
            o  A"#
        );
    }

    #[test]
    fn octopus_branch_and_merge() {
        assert_eq!(
            render(
                r#"
                        /-----\
                       /       \
                      D /--C--\ I
                     / /---D---\ \
                    A-B----E----H-J
                       \---F---/ /
                        \--G--/ F
                "#,
                &["J"],
                &[],
                &[],
                &[],
            ),
            r#"
            o      J
            +-+-.
            | | o  I
            | | |
            | o |      H
            +-+-+-+-.
            | | | | o  G
            | | | | |
            | | | o |  E
            | | | |/
            | | o |  D
            | | |\|
            | o | |  C
            | +---'
            o | |  F
            |/  |
            o   |  B
            +---'
            o  A"#
        );
    }

    #[test]
    fn reserved_column() {
        assert_eq!(
            render(
                r#"
                   A-B-C-F-G----\
                    D-E-/   \-W  \-X-Y-Z
                "#,
                &["W", "Z"],
                &["G"],
                &[],
                &[],
            ),
            r#"
              o  Z
              |
              o  Y
              |
              o  X
             /
            | o  W
            |/
            o  G
            |
            o    F
            |\
            | o  E
            | |
            | o  D
            |
            o  C
            |
            o  B
            |
            o  A"#
        );
    }

    #[test]
    fn ancestors() {
        assert_eq!(
            render(
                r#"
                   A----B-D-----E----------F-\
                    \-C--/       \-W  \-X     \-Y-Z
                "#,
                &["W", "X", "Z"],
                &["F"],
                &[("C", "A"), ("D", "C"), ("E", "D"), ("F", "E")],
                &[],
            ),
            r#"
              o  Z
              |
              o  Y
             /
            o  F
            :
            : o  X
            :/
            | o  W
            |/
            o  E
            :
            o    D
            |\
            | o  C
            | :
            o :  B
            |/
            o  A"#
        );
    }

    #[test]
    fn split_parents() {
        assert_eq!(
            render(
                r#"
                    /-B-\     A-\
                   A     D-E  B--E
                    \-C-/     C-/
                "#,
                &["E"],
                &["B", "D", "C"],
                &[("E", "A"), ("E", "B")],
                &[],
            ),
            r#"
                  o  E
            .-+-+-+
            : o | :  D
            :/ \| :
            |   o :  C
            |   |/
            o   |  B
            +---'
            o  A"#
        );
    }

    #[test]
    fn terminations() {
        assert_eq!(
            render(
                r#"
                   A-B-C  D-E-\
                            F---I--J
                        X-D-H-/  \-K
                "#,
                &["C", "J", "K"],
                &["E"],
                &[("B", "A")],
                &["A", "F", "X"],
            ),
            r#"
              o  K
              |
              | o  J
              |/
              o    I
             /|\
            | | |
            | ~ |
            |   |
            |   o  H
            |   |
            o   |  E
            +---'
            o  D
            |
            ~
            
            o  C
            |
            o  B
            |
            ~"#
        );
    }
}