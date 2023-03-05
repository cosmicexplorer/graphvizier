/*
 * Description: A dot file generator.
 *
 * Copyright (C) 2023 Danny McClanahan <dmcC2@hypnicjerk.ai>
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! A [dot file][^dot-lang] generator.
//!
//! [^dot-lang]: https://www.graphviz.org/doc/info/lang.html

/* These clippy lint descriptions are purely non-functional and do not affect the functionality
 * or correctness of the code.
 * TODO: rustfmt breaks multiline comments when used one on top of another! (each with its own
 * pair of delimiters)
 * Note: run clippy with: rustup run nightly cargo-clippy! */
/* TODO: add docs! */
/* #![warn(missing_docs)] */
/* There should be no need to use unsafe code here! */
#![deny(unsafe_code)]
/* Ensure any doctest warnings fails the doctest! */
#![doc(test(attr(deny(warnings))))]
/* Enable all clippy lints except for many of the pedantic ones. It's a shame this needs to be
 * copied and pasted across crates, but there doesn't appear to be a way to include inner
 * attributes from a common source. */
#![deny(
  clippy::all,
  clippy::default_trait_access,
  clippy::expl_impl_clone_on_copy,
  clippy::if_not_else,
  clippy::needless_continue,
  clippy::single_match_else,
  clippy::unseparated_literal_suffix,
  clippy::used_underscore_binding
)]
/* It is often more clear to show that nothing is being moved. */
#![allow(clippy::match_ref_pats)]
/* Subjective style. */
#![allow(
  clippy::derive_hash_xor_eq,
  clippy::len_without_is_empty,
  clippy::redundant_field_names,
  clippy::too_many_arguments
)]
/* Default isn't as big a deal as people seem to think it is. */
#![allow(clippy::new_without_default, clippy::new_ret_no_self)]
/* Arc<Mutex> can be more clear than needing to grok Orderings. */
#![allow(clippy::mutex_atomic)]

pub mod entities {
  #[derive(Debug, Hash, PartialEq, Eq, Clone)]
  pub struct Id(String);

  impl Id {
    pub fn into_string(self) -> String {
      self.0
    }

    pub fn validate(s: String) -> Self {
      use lazy_static::lazy_static;
      use regex::Regex;

      lazy_static! {
        static ref VALID_GRAPHVIZ_ID: Regex = Regex::new("^[a-zA-Z0-9_-]*$").unwrap();
      }

      if !VALID_GRAPHVIZ_ID.is_match(&s) {
        let rx: &Regex = &VALID_GRAPHVIZ_ID;
        panic!("invalid id '{}' provided: must match /{}/", s, rx);
      }

      Self(s)
    }
  }

  #[derive(Debug, Clone)]
  pub struct Label(pub String);

  #[derive(Debug, Clone)]
  pub struct Color(pub String);

  #[derive(Debug, Clone)]
  pub struct Vertex {
    pub id: Id,
    pub label: Option<Label>,
    pub color: Option<Color>,
    pub fontcolor: Option<Color>,
  }

  impl Default for Vertex {
    fn default() -> Self {
      use uuid::Uuid;

      let id = Id::validate(Uuid::new_v4().to_string());
      Self {
        id,
        label: None,
        color: None,
        fontcolor: None,
      }
    }
  }

  #[derive(Debug, Clone, Default)]
  pub struct NodeDefaults {
    pub color: Option<Color>,
    pub fontcolor: Option<Color>,
  }

  #[derive(Debug, Clone)]
  pub enum Entity {
    Subgraph(Subgraph),
    Vertex(Vertex),
    Edge(Edge),
  }

  #[derive(Debug, Clone)]
  pub struct Subgraph {
    pub id: Id,
    pub label: Option<Label>,
    pub color: Option<Color>,
    pub fontcolor: Option<Color>,
    pub node_defaults: Option<NodeDefaults>,
    pub entities: Vec<Entity>,
  }

  impl Default for Subgraph {
    fn default() -> Self {
      use uuid::Uuid;

      /* TODO: make this a utility method! */
      let id = Id::validate(Uuid::new_v4().to_string());
      Self {
        id,
        label: None,
        color: None,
        fontcolor: None,
        node_defaults: None,
        entities: Vec::new(),
      }
    }
  }

  #[derive(Debug, Clone)]
  pub struct Edge {
    pub source: Id,
    pub target: Id,
    pub label: Option<Label>,
    pub color: Option<Color>,
    pub fontcolor: Option<Color>,
  }

  impl Default for Edge {
    fn default() -> Self {
      Self {
        source: Id::validate("".to_string()),
        target: Id::validate("".to_string()),
        label: None,
        color: None,
        fontcolor: None,
      }
    }
  }
}

pub mod generator {
  use super::entities::*;

  #[derive(Debug, Hash, PartialEq, Eq, Clone)]
  pub struct DotOutput(pub String);

  pub struct GraphBuilder {
    entities: Vec<Entity>,
  }

  impl GraphBuilder {
    pub fn new() -> Self {
      Self {
        entities: Vec::new(),
      }
    }

    pub fn accept_entity(&mut self, e: Entity) {
      self.entities.push(e);
    }

    fn newline(output: &mut String) {
      output.push('\n');
    }

    fn newline_indent(output: &mut String, indent: usize) {
      Self::newline(output);
      for _ in 0..indent {
        output.push(' ');
      }
    }

    fn bump_indent(indent: &mut usize) {
      *indent += 2;
    }

    fn unbump_indent(indent: &mut usize) {
      assert!(*indent >= 2);
      *indent -= 2;
    }

    fn print_entity(entity: Entity, mut indent: usize) -> String {
      match entity {
        Entity::Vertex(Vertex {
          id,
          label,
          color,
          fontcolor,
        }) => {
          let mut output = id.into_string();

          let mut modifiers: Vec<String> = Vec::new();
          if let Some(Label(label)) = label {
            modifiers.push(format!("label=\"{}\"", label));
          }
          if let Some(Color(color)) = color {
            modifiers.push(format!("color=\"{}\"", color));
          }
          if let Some(Color(fontcolor)) = fontcolor {
            modifiers.push(format!("fontcolor=\"{}\"", fontcolor));
          }

          if !modifiers.is_empty() {
            output.push('[');

            for m in modifiers.into_iter() {
              output.push_str(format!("{}, ", m).as_str());
            }

            output.push(']');
          }

          output.push(';');

          output
        },
        Entity::Edge(Edge {
          source,
          target,
          label,
          color,
          fontcolor,
        }) => {
          let mut output = format!("{} -> {}", source.into_string(), target.into_string());

          let mut modifiers: Vec<String> = Vec::new();
          if let Some(Label(label)) = label {
            modifiers.push(format!("label=\"{}\"", label));
          }
          if let Some(Color(color)) = color {
            modifiers.push(format!("color=\"{}\"", color));
          }
          if let Some(Color(fontcolor)) = fontcolor {
            modifiers.push(format!("fontcolor=\"{}\"", fontcolor));
          }

          if !modifiers.is_empty() {
            output.push('[');

            for m in modifiers.into_iter() {
              output.push_str(format!("{}, ", m).as_str());
            }

            output.push(']');
          }

          output.push(';');

          output
        },
        Entity::Subgraph(Subgraph {
          id,
          label,
          color,
          fontcolor,
          node_defaults,
          entities,
        }) => {
          let mut output = format!("subgraph {} {{", id.into_string());
          Self::bump_indent(&mut indent);

          Self::newline_indent(&mut output, indent);
          if let Some(Label(label)) = label {
            output.push_str(format!("label = \"{}\";", label).as_str());
            Self::newline_indent(&mut output, indent);
          }
          output.push_str("cluster = true;");
          Self::newline_indent(&mut output, indent);
          output.push_str("rank = same;");
          Self::newline(&mut output);

          if let Some(Color(color)) = color {
            Self::newline_indent(&mut output, indent);
            output.push_str(format!("color = \"{}\";", color).as_str());
          }
          if let Some(Color(fontcolor)) = fontcolor {
            Self::newline_indent(&mut output, indent);
            output.push_str(format!("fontcolor = \"{}\";", fontcolor).as_str());
          }
          if let Some(NodeDefaults { color, fontcolor }) = node_defaults {
            let mut modifiers: Vec<String> = Vec::new();
            if let Some(Color(color)) = color {
              modifiers.push(format!("color=\"{}\"", color));
            }
            if let Some(Color(fontcolor)) = fontcolor {
              modifiers.push(format!("fontcolor=\"{}\"", fontcolor));
            }
            if !modifiers.is_empty() {
              Self::newline_indent(&mut output, indent);
              output.push_str("node [");
              for m in modifiers.into_iter() {
                output.push_str(format!("{}, ", m).as_str());
              }
              output.push_str("];")
            }
          }
          Self::newline(&mut output);

          for e in entities.into_iter() {
            Self::newline_indent(&mut output, indent);
            let expr = Self::print_entity(e, indent);
            output.push_str(expr.as_str());
          }

          Self::unbump_indent(&mut indent);
          Self::newline_indent(&mut output, indent);
          output.push('}');

          output
        },
      }
    }

    pub fn build(self, graph_name: Id) -> DotOutput {
      let mut output: String = String::new();
      let mut indent: usize = 0;

      output.push_str(format!("digraph {} {{", graph_name.into_string()).as_str());
      Self::bump_indent(&mut indent);

      Self::newline_indent(&mut output, indent);
      output.push_str("compound = true;");

      for entity in self.entities.into_iter() {
        Self::newline(&mut output);
        Self::newline_indent(&mut output, indent);

        let expr = Self::print_entity(entity, indent);
        output.push_str(expr.as_str());
      }

      Self::unbump_indent(&mut indent);
      assert_eq!(indent, 0);
      Self::newline_indent(&mut output, indent);
      output.push('}');
      Self::newline(&mut output);

      DotOutput(output)
    }
  }

  #[cfg(test)]
  mod test {
    use super::*;

    fn numeric_vertex(index: usize) -> Vertex {
      let key = format!("node_{}", index);
      Vertex {
        id: Id::validate(key.clone()),
        label: Some(Label(key)),
        color: None,
        fontcolor: None,
      }
    }

    #[test]
    fn render_single_vertex() {
      let mut gb = GraphBuilder::new();
      gb.accept_entity(Entity::Vertex(numeric_vertex(0)));
      let DotOutput(output) = gb.build(Id::validate("test_graph".to_string()));

      assert_eq!(
        output,
        "digraph test_graph {\n  \
             compound = true;\n\n  \
             node_0[label=\"node_0\", ];\n\
           }\n"
      );
    }

    #[test]
    fn render_single_edge() {
      let mut gb = GraphBuilder::new();
      gb.accept_entity(Entity::Vertex(numeric_vertex(0)));
      gb.accept_entity(Entity::Vertex(numeric_vertex(1)));
      gb.accept_entity(Entity::Edge(Edge {
        source: numeric_vertex(0).id,
        target: numeric_vertex(1).id,
        label: Some(Label("asdf".to_string())),
        ..Default::default()
      }));

      let DotOutput(output) = gb.build(Id::validate("test_graph".to_string()));

      assert_eq!(
        output,
        "digraph test_graph {\n  \
             compound = true;\n\n  \
             node_0[label=\"node_0\", ];\n\n  \
             node_1[label=\"node_1\", ];\n\n  \
             node_0 -> node_1[label=\"asdf\", ];\n\
           }\n"
      );
    }
  }
}

/// Implement this trait to expose a graphviz implementation of your type.
pub trait Graphable {
  /// This impl will be somewhat complex, and different for each type!
  fn build_graph(self) -> generator::GraphBuilder;
}
